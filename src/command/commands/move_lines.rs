use std::any::Any;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange, SimpleLineAddressType};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct MoveLines {
    pub line_range: LineRange,
    pub address: LineAddressType,
}

impl Command for MoveLines {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let start = editor.get_line_number_from(&self.line_range.start);
        let end = editor.get_line_number_from(&self.line_range.end);
        let mut dest = editor.get_line_number_from(&self.address);

        let lines: Vec<String> = if start <= end && start < editor.buffer.lines.len() {
            editor.buffer.lines.drain(start..=end).collect()
        } else {
            Vec::new()
        };

        if dest > end {
            dest -= lines.len();
        }

        if dest >= editor.buffer.lines.len() {
            dest = editor.buffer.lines.len().saturating_sub(1);
        }

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
