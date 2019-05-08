use crate::display::format::Formatter;
use crate::utils::{OptionalSkip, OptionalTake};

use serde_json::Value;

use std::cmp::max;
use std::sync::Arc;

pub struct LinesCollector {
    formatter: Arc<Formatter>,

    skip_items: Option<usize>,
    take_items: Option<usize>,
    take_lines: Option<usize>,
    take_last_lines: Option<usize>
}

pub struct CollectedLines {
    pub lines: Vec<String>,
    pub items_count: usize,
    pub has_cropped_items: bool
}

impl LinesCollector {
    pub fn new(
        formatter: Arc<Formatter>,
    ) -> Self {
        Self {
            formatter,
            skip_items: None,
            take_items: None,
            take_lines: None,
            take_last_lines: None
        }
    }

    pub fn skip_items(mut self, n: usize) -> Self {
        self.skip_items = Some(n);
        self
    }

    pub fn take_items(mut self, n: usize) -> Self {
        self.take_items = Some(n);
        self
    }

    pub fn take_lines(mut self, n: usize) -> Self {
        self.take_lines = Some(n);
        self
    }

    pub fn take_last_lines(mut self, n: usize) -> Self {
        self.take_last_lines = Some(n);
        self
    }

    pub fn collect(self, source: impl Iterator<Item=Value>) -> CollectedLines {
        let mut real_lines_count: usize = 0;
        let mut items_count: usize = 0;
        let mut items_sizes: Vec<usize> = vec![];

        let mut lines: Vec<String> = source
            .skip_by_option(self.skip_items)
            .take_by_option(self.take_items)
            .enumerate()
            .map(|(index, item)| self.formatter.format(&item, index))
            .map(|text| {
                text.lines()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            })
            .filter(|item_lines| {
                items_count += 1;
                real_lines_count += item_lines.len();
                items_sizes.push(item_lines.len());
                true
            })
            .flat_map(|item_lines| item_lines)
            .take_by_option(self.take_lines)
            .collect();

        if let Some(take_last_lines) = self.take_last_lines {
            let lines_to_skip = max(lines.len(), take_last_lines) - take_last_lines;
            let mut total = 0;
            items_count = items_sizes.iter()
                .skip_while(|c| {
                    total += *c;
                    real_lines_count -= *c;
                    total < lines_to_skip
                }).count();
            lines.drain(..lines_to_skip);
        }

        let lines_count = lines.len();

        CollectedLines {
            lines,
            items_count,
            has_cropped_items: real_lines_count != lines_count
        }
    }
}
