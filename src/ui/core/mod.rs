mod component;
mod dispatcher;
mod reducer;

pub use self::component::*;
pub use self::dispatcher::*;
pub use self::reducer::*;

use termion::event::{Event as InputEvent, Key};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use std::thread;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::io::{Stdout, stdout, stdin};
use std::io::Write;
use std::sync::{Arc, RwLock};
use ui::state::State;
use std::marker::PhantomData;

pub struct UiCore<C, S, E> {
    component_type: PhantomData<C>,
    state_type: PhantomData<S>,
    event_type: PhantomData<E>
}

impl <C, S, E> UiCore<C, S, E>
    where C: Component<S, E> + Send + Sync + 'static,
          S: Send + Sync + 'static,
          E: Send + Sync + 'static
{

    pub fn start(
        component: C,
        reducer: impl Reducer<S, E>,
        state: S
    ) {
        let component_rc = Arc::new(component);
        let state_rc = Arc::new(RwLock::new(state));

        let (tx, rx) = channel();

        UiCore::start_input_listening(component_rc.clone(), tx, state_rc.clone());
        UiCore::start_rendering(component_rc, reducer, rx, state_rc);
    }

    fn start_rendering(
        component: Arc<C>,
        reducer: impl Reducer<S, E>,
        rx: Receiver<E>,
        state: Arc<RwLock<S>>
    ) {
        info!("Starting event loop...");
        let mut stdout = AlternateScreen::from(stdout().into_raw_mode().unwrap());

        {
            let local_state = state.read()
                .expect("Cannot acquire read lock on state");
            component.render(&local_state, &mut stdout);
        }

        loop {
            let event = rx.recv().unwrap();
            let mut local_state = state.write()
                .expect("Cannot acquire write lock on state");;
            reducer.reduce(&mut local_state, event);
            component.render(&local_state, &mut stdout);
        }
    }

    fn start_input_listening(
        component: Arc<C>,
        tx: Sender<E>,
        state: Arc<RwLock<S>>
    ) {
        info!("Starting input listening thread...");
        thread::spawn(move || {
            let dispatcher = ChannelDispatcher::new(tx);
            let stdin = stdin();
            for c in stdin.events() {
                match c {
                    Ok(event) => {
                        info!("Input event {:?} has been received", event);
                        match event {
                            InputEvent::Key(Key::Ctrl('c')) => {
                                std::process::exit(0);
                            },
                            _ => {
                                let local_state = state.read()
                                    .expect("Cannot acquire read lock on state");
                                component.dispatch_input(&local_state, &event, &dispatcher);
                            }
                        }
                    },
                    Err(err) => {
                        error!("Cannot read input from stdin: {}", err);
                        panic!(err);
                    }
                }
            }
        });
    }
}