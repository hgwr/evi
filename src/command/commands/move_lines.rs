use std::any::Any;
use std::cmp;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange, SimpleLineAddressType};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct MoveLines {
    pub line_range: LineRange,
    pub address: LineAddressType,
    pub original_start_idx: Option<usize>,
    pub inserted_base: Option<usize>,
    pub drained_lines: Vec<String>,
}

impl Command for MoveLines {
    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let len = editor.buffer.lines.len();
        let mut start = editor.get_line_number_from(&self.line_range.start);
        let mut end = editor.get_line_number_from(&self.line_range.end);

        if len == 0 {
            return Ok(());
        }

        start = cmp::min(start, len - 1);
        end = cmp::min(end, len - 1);

        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        let mut dest = editor.get_line_number_from(&self.address);

        let lines: Vec<String> = editor.buffer.lines.drain(start..=end).collect();
        self.original_start_idx = Some(start);
        self.drained_lines = lines.clone();
        if dest > end {
            dest -= lines.len();
        }

        if dest >= editor.buffer.lines.len() {
            dest = editor.buffer.lines.len().saturating_sub(1);
        }

        let base = if editor.buffer.lines.is_empty() {
            0
        } else if matches!(
            self.address,
            LineAddressType::Absolute(SimpleLineAddressType::LineNumber(0))
        ) {
            dest
        } else {
            dest + 1
        };

        editor.buffer.lines.splice(base..base, lines.into_iter());
        self.inserted_base = Some(base);
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let (Some(base), Some(start_idx)) = (self.inserted_base, self.original_start_idx) {
            if base < editor.buffer.lines.len() {
                let end = (base + self.drained_lines.len()).min(editor.buffer.lines.len());
                editor.buffer.lines.drain(base..end);
            }
            let insert_idx = start_idx.min(editor.buffer.lines.len());
            editor.buffer.lines.splice(
                insert_idx..insert_idx,
                self.drained_lines.clone().into_iter(),
            );
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        let mut new_cmd = Box::new(MoveLines {
            line_range: self.line_range.clone(),
            address: self.address.clone(),
            original_start_idx: None,
            inserted_base: None,
            drained_lines: Vec::new(),
        });
        new_cmd.execute(editor)?;
        Ok(Some(new_cmd))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
