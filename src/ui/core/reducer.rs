pub trait Reducer<S, E> {
    fn reduce(&self, state: &mut S, event: E);
}
