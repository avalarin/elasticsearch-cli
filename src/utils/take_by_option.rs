//! Creates an iterator that yields its first n elements, but only if n != None

pub trait OptionalTake<I> where I: Iterator {
    fn take_by_option(self, n: Option<usize>) -> TakeByOption<I>;
}

impl <I> OptionalTake<I> for I where I: Iterator {
    fn take_by_option(self, n: Option<usize>) -> TakeByOption<I> {
        TakeByOption::new(self, n)
    }
}

pub struct TakeByOption<I> {
    iter: I,
    n: Option<usize>
}

impl<I> TakeByOption<I> {
    pub fn new(iter: I, n: Option<usize>) -> TakeByOption<I> {
        TakeByOption { iter, n }
    }
}

impl<I> Iterator for TakeByOption<I> where I: Iterator {
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<<I as Iterator>::Item> {
        match self.n {
            None => self.iter.next(),
            Some(0) => None,
            Some(n) => {
                self.n = Some(n - 1);
                self.iter.next()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OptionalTake;

    #[test]
    fn should_not_affect_for_none() {
        assert_eq!(
            vec![1, 2, 3],
            vec![1, 2, 3].into_iter().take_by_option(None).collect::<Vec<_>>()
        )
    }

    #[test]
    fn should_takes_some_elements_for_some() {
        assert_eq!(
            vec![1, 2],
            vec![1, 2, 3].into_iter().take_by_option(Some(2)).collect::<Vec<_>>()
        )
    }
}