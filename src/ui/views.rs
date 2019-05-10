use super::core::{Component, Dispatcher};

use super::state::{ State, View };
use super::events::Event;

use termion::event::{ Event as InputEvent, Key };

use std::io::Write;
use ui::events::PagerEvent;

pub struct RootView {
    pager_view: PagerView,
    help_view: HelpView
}

impl RootView {
    pub fn new() -> Self {
        Self {
            pager_view: PagerView::new(),
            help_view: HelpView::new()
        }
    }
}

impl Component<State, Event> for RootView {
    fn render(&self, state: &State, out: &mut Write) {
        match state.view {
            View::Pager => self.pager_view.render(state, out),
            View::Help => self.help_view.render(state, out)
        };
        out.flush().unwrap();
    }

    fn dispatch_input(&self, state: &State, event: &InputEvent, dispatcher: &Dispatcher<Event>) {
        match event {
            InputEvent::Key(Key::Char('s')) => dispatcher.dispatch(Event::SwitchView),
            _ => match state.view {
                View::Pager => self.pager_view.dispatch_input(state, event, dispatcher),
                View::Help => self.help_view.dispatch_input(state, event, dispatcher)
            }
        }
    }
}

pub struct PagerView {

}

impl PagerView {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component<State, Event> for PagerView {
    fn render(&self, state: &State, out: &mut Write) {
        write!(out, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
        write!(out, "Pager, page #{}", state.pager.top_index).unwrap();
    }

    fn dispatch_input(&self, _state: &State, event: &InputEvent, dispatcher: &Dispatcher<Event>) {
        match event {
            InputEvent::Key(Key::Up) => dispatcher.dispatch(Event::Pager(PagerEvent::ScrollUp)),
            InputEvent::Key(Key::Down) => dispatcher.dispatch(Event::Pager(PagerEvent::ScrollDown)),
            _ => {}
        }
    }
}

pub struct HelpView {

}

impl HelpView {
    pub fn new() -> Self {
        Self {}
    }
}

impl Component<State, Event> for HelpView {
    fn render(&self, _state: &State, out: &mut Write) {
        write!(out, "{}{}", termion::clear::All, termion::cursor::Goto(1, 1)).unwrap();
        write!(out, "Hello from Help").unwrap();
    }

    fn dispatch_input(&self, _state: &State, _event: &InputEvent, _dispatcher: &Dispatcher<Event>) {
    }
}