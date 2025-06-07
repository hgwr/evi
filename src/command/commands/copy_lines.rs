use std::any::Any;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange, SimpleLineAddressType};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct CopyLines {
    pub line_range: LineRange,
    pub address: LineAddressType,
}

impl Command for CopyLines {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let start = editor.get_line_number_from(&self.line_range.start);
        let end = editor.get_line_number_from(&self.line_range.end);
        let (start, end) = if start > end { (end, start) } else { (start, end) };
        let mut dest = editor.get_line_number_from(&self.address);

        if dest >= editor.buffer.lines.len() {
            dest = editor.buffer.lines.len().saturating_sub(1);
        }

        let base = if editor.buffer.lines.is_empty() {
            0
        } else if matches!(
            .buffer
            .lines
            .splice(base..base, lines.into_iter());
            .collect();

        let base = if matches!(
            self.address,
            LineAddressType::Absolute(SimpleLineAddressType::LineNumber(0))
        ) {
            dest
        } else {
            dest + 1
        };

        for (i, line) in lines.into_iter().enumerate() {
            editor.buffer.lines.insert(base + i, line);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
