//! Creates an iterator that skips the first n elements, but only if n != None

pub trait OptionalSkip<I> where I: Iterator {
    fn skip_by_option(self, n: Option<usize>) -> SkipByOption<I>;
}

impl <I> OptionalSkip<I> for I where I: Iterator {
    fn skip_by_option(self, n: Option<usize>) -> SkipByOption<I> {
        SkipByOption::new(self, n)
    }
}

pub struct SkipByOption<I> {
    iter: I,
    n: Option<usize>
}

impl<I> SkipByOption<I> {
    pub fn new(iter: I, n: Option<usize>) -> SkipByOption<I> {
        SkipByOption { iter, n }
    }
}

impl<I> Iterator for SkipByOption<I> where I: Iterator {
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<I::Item> {
        match self.n {
            None | Some(0) => self.iter.next(),
            Some(n) => {
                self.n = None;
                self.iter.nth(n)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OptionalSkip;

    #[test]
    fn should_not_skips_elements_for_none() {
        assert_eq!(
            vec![1, 2, 3],
            vec![1, 2, 3].into_iter().skip_by_option(None).collect::<Vec<_>>()
        )
    }

    #[test]
    fn should_skips_elements_for_some() {
        assert_eq!(
            vec![3],
            vec![1, 2, 3].into_iter().skip_by_option(Some(2)).collect::<Vec<_>>()
        )
    }
}