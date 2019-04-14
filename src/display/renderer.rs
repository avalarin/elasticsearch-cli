use super::{OutputFormat, JSONExtractor};

use serde_json::Value;
use strfmt::strfmt;
use colored::*;

use std::collections::HashMap;
use std::io::Write;

pub struct Renderer {
    format: OutputFormat,
    extractor: JSONExtractor
}

impl Renderer {
    pub fn create(format: OutputFormat, extractor: JSONExtractor) -> Self {
        Renderer { format, extractor }
    }

    pub fn render(&self, w: &mut Write, item: &Value, index: usize) {
        match &self.format {
            OutputFormat::Pretty => {
                if index > 0 {
                    let _ = writeln!(w, "{}", "-".repeat(4).blue().bold());
                }
                let map = self.extractor.extract(item);
                for (key, value) in map {
                    let _ = writeln!(w, "{}: {}", key.green().bold(), format_string(&value));
                }
            },
            OutputFormat::JSON => {
                let _ = writeln!(w, "{}", item);
            },
            OutputFormat::Custom(format) => {
                let map: &HashMap<String, String> = &self.extractor.extract(item).into_iter().collect();
                let _ = writeln!(w, "{}", strfmt(format, map).unwrap_or_else(|_| "Cannot format item".to_owned()));
            }
        };
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
    use display::renderer::Renderer;
    use display::format::OutputFormat;

    use colored::*;

    struct Writer {
        pub data: String
    }

    impl Writer {
        fn new() -> Self {
            Writer { data: "".to_string() }
        }
    }

    impl std::io::Write for Writer {
        fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
            self.data.push_str(std::str::from_utf8(buf).unwrap());
            Ok(buf.len())
        }

        fn flush(&mut self) -> Result<(), std::io::Error> {
            Ok(())
        }
    }

    #[test]
    fn renderer_pretty_format_should_render_first_row_without_delimiter() {
        let mut writer = Writer::new();

        Renderer::create(OutputFormat::Pretty, JSONExtractor::default())
            .render(
                &mut writer,
                &json!({
                    "root": {
                         "obj": {
                              "strKey": "str1"
                         },
                    }
                }),
                0);

        assert_eq!(format!("{}: {}\n", "root.obj.strKey".green().bold(), "str1"), writer.data);
    }

    #[test]
    fn renderer_pretty_format_should_render_not_first_row_with_delimiter() {
        let mut writer = Writer::new();

        Renderer::create(OutputFormat::Pretty, JSONExtractor::default())
            .render(
                &mut writer,
                &json!({
                    "root": {
                         "obj": {
                              "strKey": "str1"
                         },
                    }
                }),
                5);

        assert_eq!(format!("{}\n{}: {}\n", "----".blue().bold(), "root.obj.strKey".green().bold(), "str1"), writer.data);
    }

    #[test]
    fn renderer_pretty_format_should_render_several_fields() {
        let mut writer = Writer::new();

        Renderer::create(OutputFormat::Pretty, JSONExtractor::default())
            .render(
                &mut writer,
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
            writer.data
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