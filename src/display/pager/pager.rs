use crate::client::Collector;
use crate::display::Formatter;
use super::ScrollMode;

use serde_json::Value;
use termion::event::{Key, Event, MouseEvent, MouseButton};
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use std::cmp::max;
use std::io::{Write, stdout, stdin, Stdout};
use std::sync::Arc;
use display::pager::collector::{LinesCollector, CollectedLines};

pub struct Pager {
    formatter: Arc<Formatter>,
    collector: Collector<Value>,
    stdout: AlternateScreen<RawTerminal<Stdout>>,
    top_index: usize,
    bottom_index: usize,
    scroll_mode: ScrollMode,
    has_cropped_item: bool
}

impl Pager {
    pub fn new(collector: Collector<Value>, formatter: Arc<Formatter>) -> Self {
        Pager {
            formatter: formatter.clone(),
            collector,
            stdout: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
            top_index: 0,
            bottom_index: 0,
            scroll_mode: ScrollMode::ScrollUp,
            has_cropped_item: false
        }
    }

    pub fn start(mut self) {
        let stdin = stdin();

        self.display();
        for c in stdin.events() {
            match c.unwrap() {
                Event::Key(Key::Char('q')) |
                Event::Key(Key::Ctrl('c')) => break,
                Event::Key(Key::Up) |
                Event::Key(Key::PageUp) |
                Event::Mouse(MouseEvent::Press(MouseButton::WheelUp, _, _)) => {
                    if (self.scroll_mode == ScrollMode::ScrollUp) || !self.has_cropped_item {
                        self.top_index = max(self.top_index, 1) - 1;
                    }
                    self.scroll_mode = ScrollMode::ScrollUp;
                },
                Event::Key(Key::Down) |
                Event::Key(Key::PageDown) |
                Event::Mouse(MouseEvent::Press(MouseButton::WheelDown, _, _)) => {
                    if (self.scroll_mode == ScrollMode::ScrollDown) || !self.has_cropped_item {
                        self.bottom_index = self.bottom_index + 1;
                    }
                    self.scroll_mode = ScrollMode::ScrollDown;
                },
                _ => {}
            }
            self.display();
        }
    }

    fn display(&mut self) {
        let (_, height) = termion::terminal_size().unwrap();
        let working_height = (height - 2) as usize;

        let lines = self.get_lines(working_height);

        self.has_cropped_item = lines.has_cropped_items;
        match self.scroll_mode {
            ScrollMode::ScrollUp => {
                self.bottom_index = self.top_index + lines.items_count;
            },
            ScrollMode::ScrollDown => {
                self.top_index = self.bottom_index - lines.items_count;
            }
        }

        self.clear();
        self.print_lines(lines.lines);

        write!(self.stdout, "{}Loaded {} from {}, displayed {}-{} ({} items)",
               termion::cursor::Goto(1, height - 1),
               self.collector.from,
               self.collector.total,
               self.top_index,
               self.bottom_index,
               lines.items_count
        ).unwrap();

        write!(self.stdout, "{}Press q to exit, ↑/↓ to navigate",
               termion::cursor::Goto(1, height)
        ).unwrap();

        self.stdout.flush().unwrap();
    }

    fn clear(&mut self) {
        write!(self.stdout,
               "{}{}{}",
               termion::clear::All,
               termion::cursor::Goto(1, 1),
               termion::cursor::Hide
        ).unwrap();
    }

    fn get_lines(&mut self, limit: usize) -> CollectedLines {
        match self.scroll_mode {
            ScrollMode::ScrollUp => {
                LinesCollector::new(self.formatter.clone())
                    .skip_items(self.top_index)
                    .take_lines(limit)
            },
            ScrollMode::ScrollDown => {
                LinesCollector::new(self.formatter.clone())
                    .take_items(self.bottom_index)
                    .take_last_lines(limit)
            }
        }.collect(self.collector.iter())
    }

    fn print_lines(&mut self, lines: Vec<String>) {
        lines.iter().enumerate().for_each(|(index, line)| {
            write!(self.stdout, "{}{}",
                   termion::cursor::Goto(1, (index + 1) as u16),
                   line
            ).unwrap();
        });
    }

}