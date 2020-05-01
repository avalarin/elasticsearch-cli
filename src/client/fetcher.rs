#[derive(Debug, Fail)]
pub enum FetcherError {
    #[fail(display = "{}", inner)]
    RequestError { inner: String }
}

pub trait Fetcher<T> {
    fn fetch_next(&self, from: usize) -> Result<(usize, Vec<T>), FetcherError>;
}

pub struct Collector<T> where T: Clone {
    fetcher: Box<dyn Fetcher<T>>,
    buffer: Vec<T>,
    pub from: usize,
    pub total: usize,
}

impl <T> Collector<T> where T: Clone {
    pub fn create(fetcher: impl Fetcher<T> + 'static) -> Result<Self, FetcherError> {
        let mut collector = Collector {
            fetcher: Box::new(fetcher),
            buffer: Vec::new(),
            from: 0,
            total: 0
        };

        collector.fetch_next(true)
            .map(|_| collector)
    }

    pub fn iter(&mut self) -> CollectorIterator<'_, T> {
        CollectorIterator {
            collector: self,
            position: 0
        }
    }

    fn get(&mut self, index: usize) -> Option<&T> {
        // no more fetched items
        if self.from - index == 0 &&
            self.fetch_next(false).unwrap_or(0) == 0 {
            return None
        }

        self.buffer.get(index)
    }

    fn fetch_next(&mut self, first: bool) -> Result<usize, FetcherError> {
        if !first && self.from >= self.total {
            return Ok(0)
        }
        let (total, results) = self.fetcher.fetch_next(self.from)?;
        if results.is_empty() {
            return Ok(0)
        }
        let count = results.len();
        info!("Loaded {}/{} results", self.from + count, total);

        self.total = total;
        self.from += results.len();
        self.buffer.extend(results);

        Ok(count)
    }
}

pub struct CollectorIterator<'a, T> where T: Clone {
    collector: &'a mut Collector<T>,
    position: usize
}

impl <'a, T> Iterator for CollectorIterator<'a, T> where T: Clone {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        let result = self.collector.get(self.position);
        self.position += 1;
        result.map(Clone::clone)
    }
}

impl <'a, T> DoubleEndedIterator for CollectorIterator<'a, T> where T: Clone {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.position -= 1;
        self.collector.get(self.position).map(Clone::clone)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FnFetcher<T>(pub Box<dyn Fn(usize) -> Result<(usize, Vec<T>), FetcherError>>);
    impl <T> Fetcher<T> for FnFetcher<T> {
        fn fetch_next(&self, from: usize) -> Result<(usize, Vec<T>), FetcherError> {
            (self.0)(from)
        }
    }

    #[test]
    fn it_should_return_error_from_fetcher_on_creation() {
        let fetcher = FnFetcher::<i32>(Box::new(|_from| {
            Err(FetcherError::RequestError { inner: "fail".to_string() })
        }));
        match Collector::create(fetcher) {
            Ok(_) => assert!(false, "creation should be failed"),
            Err(_) => assert!(true),
        }
    }

    #[test]
    fn it_should_stop_fetching_when_collector_returned_error() {
        let fetcher = FnFetcher(Box::new(|from| match from {
            0 => Ok((5, vec![99, 98])),
            2 => Err(FetcherError::RequestError { inner: "fail".to_string() }),
            _ => Ok((5, vec![])),
        }));
        let result: Vec<i32> = Collector::create(fetcher).unwrap().iter().collect();
        assert_eq!(vec![99, 98], result);
    }

    #[test]
    fn it_should_not_fetch_more_items_than_returned_in_total_count() {
        let fetcher = FnFetcher(Box::new(|from| match from {
            0 => Ok((5, vec![99])),
            1 => Ok((5, vec![98, 97])),
            2 => Ok((5, vec![1, 2])),
            3 => Ok((5, vec![96, 95])),
            5 => Ok((5, vec![1, 2])),
            _ => Ok((5, vec![1, 2])),
        }));
        let result: Vec<i32> = Collector::create(fetcher).unwrap().iter().collect();
        assert_eq!(vec![99, 98, 97, 96, 95], result);
    }

    #[test]
    fn it_may_contain_more_items_than_total_count_if_fetcher_returns() {
        let fetcher = FnFetcher(Box::new(|from| match from {
            0 => Ok((5, vec![99])),
            1 => Ok((5, vec![98, 97])),
            2 => Ok((5, vec![1, 2])),
            3 => Ok((5, vec![96, 95, 94])),
            5 => Ok((5, vec![1, 2])),
            6..=20 => Ok((5, vec![1, 2])),
            _ => Ok((5, vec![]))
        }));
        let result: Vec<i32> = Collector::create(fetcher).unwrap().iter().collect();
        assert_eq!(vec![99, 98, 97, 96, 95, 94], result);
    }

    #[test]
    fn it_should_not_fetch_more_items_than_requested() {
        let fetcher = FnFetcher(Box::new(|from| match from {
            0 => Ok((5, vec![99])),
            1 => Ok((5, vec![98, 97])),
            _ => panic!("should not happen"),
        }));
        let result: Vec<i32> = Collector::create(fetcher).unwrap().iter().take(3).collect();
        assert_eq!(vec![99, 98, 97], result);
    }

    #[test]
    fn it_should_fetch_no_more_than_necessary() {
        let fetcher = FnFetcher(Box::new(|from| match from {
            0 => Ok((5, vec![99])),
            1 => Ok((5, vec![98, 97])),
            _ => panic!("should not happen"),
        }));

        let result: Vec<i32> = Collector::create(fetcher).unwrap().iter().skip(1).take(2).collect();
        assert_eq!(vec![98, 97], result);
    }

    #[test]
    fn it_should_fetch_no_less_than_necessary() {
        let fetcher = FnFetcher(Box::new(|from| match from {
            0 => Ok((5, vec![99])),
            1 => Ok((5, vec![98, 97])),
            _ => panic!("should not happen"),
        }));

        let mut collector =  Collector::create(fetcher).unwrap();

        let result: Vec<i32> = collector.iter().take(2).collect();
        assert_eq!(vec![99, 98], result);

        let result: Vec<i32> = collector.iter().skip(1).take(2).collect();
        assert_eq!(vec![98, 97], result);
    }
}