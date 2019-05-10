use super::core::Reducer;

use super::events::{Event, PagerEvent};
use super::state::{State, View};

pub struct RootReducer { }

impl RootReducer {
    pub fn new() -> Self {
        Self { }
    }
}

impl Reducer<State, Event> for RootReducer {
    fn reduce(&self, state: &mut State, event: Event) {
        match event {
            Event::SwitchView => {
                state.view = match state.view {
                    View::Pager => View::Help,
                    View::Help => View::Pager
                }
            },
            Event::Pager(pagerEvent) => match pagerEvent {
                PagerEvent::ScrollUp => {
                    state.pager.top_index = std::cmp::max(state.pager.top_index, 1) - 1;
                },
                PagerEvent::ScrollDown => {
                    state.pager.top_index += 1;
                }
            }
        };
    }
}