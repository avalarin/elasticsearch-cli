use std::sync::mpsc::Sender;

pub trait Dispatcher<E> {
    fn dispatch(&self, event: E);
}

pub struct ChannelDispatcher<E> {
    sender: Sender<E>
}

impl <E> ChannelDispatcher<E> {
    pub fn new(sender: Sender<E>) -> Self {
        Self { sender }
    }
}

impl <E> Dispatcher<E> for ChannelDispatcher<E> {
    fn dispatch(&self, event: E) {
        self.sender.send(event);
    }
}