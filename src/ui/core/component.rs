use super::dispatcher::Dispatcher;

use termion::event::Event as InputEvent;

use std::io::Write;

pub trait Component<S, E> {

    fn render(&self, state: &S, out: &mut Write);

    fn dispatch_input(&self, state: &S, input: &InputEvent, dispatcher: &Dispatcher<E>);
}
