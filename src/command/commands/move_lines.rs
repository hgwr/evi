use std::any::Any;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange};
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

        let mut lines: Vec<String> = Vec::new();
        for _ in start..=end {
            if start < editor.buffer.lines.len() {
                lines.push(editor.buffer.lines.remove(start));
            }
        }

        if dest > end {
            dest -= lines.len();
        }

        for (i, line) in lines.into_iter().enumerate() {
            editor.buffer.lines.insert(dest + 1 + i, line);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
