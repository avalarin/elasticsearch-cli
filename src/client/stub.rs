use client::{Client, SearchRequest, Collector, ClientError, FetcherError};
use serde_json::Value;
use client::fetcher::Fetcher;

pub struct StubClient {
    buffer_size: usize
}

pub struct StubFetcher {
    buffer_size: usize,
    total_count: usize
}

impl StubClient {
    pub fn new(
        buffer_size: usize
    ) -> Self {
        Self { buffer_size }
    }
}

impl Client for StubClient {
    fn execute(&self, _: &SearchRequest) -> Result<Collector<Value>, ClientError> {
        Collector::create(StubFetcher::new(self.buffer_size, 1000))
            .map_err(From::from)
    }
}

impl StubFetcher {
    fn new(
        buffer_size: usize,
        total_count: usize
    ) -> Self {
        Self {
            buffer_size,
            total_count
        }
    }
}

impl Fetcher<Value> for StubFetcher {

    fn fetch_next(&self, from: usize) -> Result<(usize, Vec<Value>), FetcherError> {
        let to = if from + self.buffer_size < self.total_count {
            from + self.buffer_size
        } else {
            self.total_count
        };
        let count = to - from;
        Ok((count, (from..to).map(|i| json!({"index": i})).collect()))
    }
}