use std::any::Any;
use std::cmp;

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
        let len = editor.buffer.lines.len();
        let mut start = editor.get_line_number_from(&self.line_range.start);
        let mut end = editor.get_line_number_from(&self.line_range.end);

        if len > 0 {
            start = cmp::min(start, len - 1);
            end = cmp::min(end, len - 1);
        } else {
            start = 0;
            end = 0;
        }

        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
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
