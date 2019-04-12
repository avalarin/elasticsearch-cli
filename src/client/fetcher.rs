use std::collections::linked_list::LinkedList;

#[derive(Debug, Fail)]
pub enum FetcherError {
    #[fail(display = "request error: {}", inner)]
    RequestError { inner: String }
}

pub trait Fetcher<T> {
    fn fetch_next(&self, from: usize) -> Result<(usize, Vec<T>), FetcherError>;
}

pub struct Collector<T> {
    fetcher: Box<Fetcher<T>>,
    buffer: LinkedList<T>,
    from: usize,
    total: usize,
}

impl <T> Collector<T> {
    pub fn create(fetcher: impl Fetcher<T> + 'static) -> Result<Self, FetcherError> {
        let mut collector = Collector {
            fetcher: Box::new(fetcher),
            buffer: LinkedList::new(),
            from: 0,
            total: 0
        };

        collector.fetch_next()
            .map(|_| collector)
    }

    fn fetch_next(&mut self) -> Result<usize, FetcherError> {
        let (total, results) = self.fetcher.fetch_next(self.from).map_err(|err| {
            error!("Cannot fetch next items: {}", err);
            err
        })?;
        if results.len() == 0 {
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

impl <T> Iterator for Collector<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.len() == 0 {
            if self.from >= self.total {
                return None
            }
            if self.fetch_next().unwrap_or(0) == 0 {
                return None
            }
        }
        self.buffer.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FnFetcher<T>(pub Box<Fn(usize) -> Result<(usize, Vec<T>), FetcherError>>);
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
        let result: Vec<i32> = Collector::create(fetcher).unwrap().collect();
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
        let result: Vec<i32> = Collector::create(fetcher).unwrap().collect();
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
            6...20 => Ok((5, vec![1, 2])),
            _ => Ok((5, vec![]))
        }));
        let result: Vec<i32> = Collector::create(fetcher).unwrap().collect();
        assert_eq!(vec![99, 98, 97, 96, 95, 94], result);
    }

    #[test]
    fn it_should_not_fetch_more_items_than_requested() {
        let fetcher = FnFetcher(Box::new(|from| match from {
            0 => Ok((5, vec![99])),
            1 => Ok((5, vec![98, 97])),
            _ => panic!("should not happen"),
        }));
        let result: Vec<i32> = Collector::create(fetcher).unwrap().take(3).collect();
        assert_eq!(vec![99, 98, 97], result);
    }

}