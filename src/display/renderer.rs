use super::{OutputFormat, JSONExtractor, Pager};
use crate::client::Collector;
use crate::display::{Formatter};

use serde_json::Value;

use std::sync::Arc;

pub trait Renderer {
    fn render(&mut self, collector: Collector<Value>);
}

pub struct SimpleRenderer {
    formatter: Formatter
}

impl SimpleRenderer {
    pub fn new(format: OutputFormat, extractor: JSONExtractor) -> Self {
        Self {
            formatter: Formatter::new(format, extractor)
        }
    }
}

impl Renderer for SimpleRenderer {
    fn render(&mut self, mut collector: Collector<Value>) {
        collector.iter().enumerate()
            .for_each(|(index, item)| {
                print!("{}", self.formatter.format(&item, index));
            });
    }
}

pub struct PagedRenderer {
    formatter: Arc<Formatter>
}

impl PagedRenderer {
    pub fn new(format: OutputFormat, extractor: JSONExtractor) -> Self {
        Self {
            formatter: Arc::new(Formatter::new(format, extractor))
        }
    }
}

impl Renderer for PagedRenderer {
    fn render(&mut self, collector: Collector<Value>) {
        Pager::new(
            collector,
            self.formatter.clone()
        ).start()
    }
}