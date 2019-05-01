use super::{JSONExtractor};

use serde_json::Value;
use strfmt::strfmt;
use colored::*;

use std::fmt::Write;
use std::collections::HashMap;

pub enum OutputFormat {
    JSON,
    Pretty,
    Custom(String)
}


pub struct Formatter {
    format: OutputFormat,
    extractor: JSONExtractor
}

impl Formatter {
    pub fn new(format: OutputFormat, extractor: JSONExtractor) -> Self {
        Self {
            format, extractor
        }
    }

    pub fn format(&self, item: &Value, index: usize) -> String {
        let mut str = String::new();
        match &self.format {
            OutputFormat::Pretty => {
                if index > 0 {
                    let _ = writeln!(str, "{}", "-".repeat(4).blue().bold());
                }
                let map = self.extractor.extract(item);
                for (key, value) in map {
                    let _ = writeln!(str, "{}: {}", key.green().bold(), format_string(&value));
                }
            },
            OutputFormat::JSON => {
                let _ = writeln!(str, "{}", item);
            },
            OutputFormat::Custom(format) => {
                let map: &HashMap<String, String> = &self.extractor.extract(item).into_iter().collect();
                let _ = writeln!(str, "{}", strfmt(format, map).unwrap_or_else(|_| "Cannot format item".to_owned()));
            }
        };
        str
    }
}

// TODO refactor this
fn format_string(value: &str) -> String {
    Some(value)
        .map(|s| str::replace(s, "\\n", "\n"))
        .map(|s| str::replace(&s, "\\t", "\t"))
        .unwrap()
}


#[cfg(test)]
mod tests {
    use super::{ JSONExtractor, format_string };
    use display::format::{ Formatter, OutputFormat };
    use colored::*;

    #[test]
    fn renderer_pretty_format_should_render_first_row_without_delimiter() {
        let str = Formatter::new(OutputFormat::Pretty, JSONExtractor::default())
            .format(
                &json!({
                    "root": {
                         "obj": {
                              "strKey": "str1"
                         },
                    }
                }),
                0);

        assert_eq!(format!("{}: {}\n", "root.obj.strKey".green().bold(), "str1"), str);
    }

    #[test]
    fn renderer_pretty_format_should_render_not_first_row_with_delimiter() {
        let str = Formatter::new(OutputFormat::Pretty, JSONExtractor::default())
            .format(
                &json!({
                    "root": {
                         "obj": {
                              "strKey": "str1"
                         },
                    }
                }),
                5);

        assert_eq!(format!("{}\n{}: {}\n", "----".blue().bold(), "root.obj.strKey".green().bold(), "str1"), str);
    }

    #[test]
    fn renderer_pretty_format_should_render_several_fields() {
        let str = Formatter::new(OutputFormat::Pretty, JSONExtractor::default())
            .format(
                &json!({
                    "root": {
                         "obj": {
                              "strKey": "str1"
                         },
                         "arr": [
                            { "value": 1 }
                         ]
                    }
                }),
                0);

        assert_eq!(
            format!(
                "{}: {}\n{}: {}\n",
                "root.arr.0.value".green().bold(),
                "1",
                "root.obj.strKey".green().bold(),
                "str1"
            ),
            str
        );
    }

    #[test]
    fn format_string_should_replaces_new_line_and_tab_placeholders_to_real_symbols() {
        assert_eq!(
            "abc\tabc\nabc - abc\tabc\nabc",
            format_string("abc\\tabc\\nabc - abc\\tabc\\nabc")
        )
    }

}