use std::any::Any;
use std::io::{stdout, Write};

use crate::command::base::Command;
use crate::data::LineRange;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct PrintCommand {
    #[cfg_attr(not(test), allow(dead_code))]
    pub line_range: LineRange,
}

impl Command for PrintCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.buffer.lines.is_empty() {
            return Ok(());
        }

        let mut start = editor.get_line_number_from(&self.line_range.start);
        let mut end = editor.get_line_number_from(&self.line_range.end);
        let last = editor.buffer.lines.len().saturating_sub(1);

        if start > last {
            start = last;
        }
        if end > last {
            end = last;
        }
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }

        let mut out = stdout();
        let mut collected = Vec::new();
        for idx in start..=end {
            if let Some(line) = editor.buffer.lines.get(idx) {
                writeln!(out, "{}", line)?;
                collected.push(line.clone());
            }
        }
        out.flush()?;

        editor.status_line = collected.join("\n");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{LineAddressType, LineRange, SimpleLineAddressType};

    #[test]
    fn print_single_line() {
        let mut editor = Editor::new();
        editor.buffer.lines = vec!["one".into(), "two".into()];

        let mut cmd = PrintCommand {
            line_range: LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1)),
                end: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1)),
            },
        };

        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.status_line, "one");
    }

    #[test]
    fn print_range() {
        let mut editor = Editor::new();
        editor.buffer.lines = vec!["a".into(), "b".into(), "c".into(), "d".into()];

        let mut cmd = PrintCommand {
            line_range: LineRange {
                start: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1)),
                end: LineAddressType::Absolute(SimpleLineAddressType::LineNumber(3)),
            },
        };

        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.status_line, "a\nb\nc");
    }
}
