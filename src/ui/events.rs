pub enum Event {
    SwitchView,
    Pager(PagerEvent)
}

pub enum PagerEvent {
    ScrollUp,
    ScrollDown
}