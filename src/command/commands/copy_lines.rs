use std::any::Any;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange};
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
        let dest = editor.get_line_number_from(&self.address);

        let mut lines: Vec<String> = Vec::new();
        for i in start..=end {
            if i < editor.buffer.lines.len() {
                lines.push(editor.buffer.lines[i].clone());
            }
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
