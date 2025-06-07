use std::any::Any;
use std::cmp;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange, SimpleLineAddressType};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct CopyLines {
    pub line_range: LineRange,
    pub address: LineAddressType,
    pub insertion_idx: Option<usize>,
    pub copied_len: usize,
}

impl Command for CopyLines {
    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let buffer_len = editor.buffer.lines.len();

        // 1. Determine source range (0-indexed, inclusive start and end)
        let mut start_idx = editor.get_line_number_from(&self.line_range.start);
        let mut end_idx = editor.get_line_number_from(&self.line_range.end);

        if buffer_len == 0 {
            // Nothing to copy from an empty buffer.
            return Ok(());
        }

        // Clamp indices to be valid for slicing the buffer.
        start_idx = cmp::min(start_idx, buffer_len - 1);
        end_idx = cmp::min(end_idx, buffer_len - 1);

        // Ensure start_idx <= end_idx.
        if start_idx > end_idx {
            std::mem::swap(&mut start_idx, &mut end_idx);
        }

        // 2. Extract lines to be copied.
        let lines_to_copy: Vec<String> = editor.buffer.lines[start_idx..=end_idx]
            .iter()
            .cloned()
            .collect();

        // 3. Determine destination insertion index (0-indexed).
        let target_line_num_for_logic = editor.get_line_number_from(&self.address);

        let clamped_dest_line_idx = if buffer_len > 0 {
            cmp::min(target_line_num_for_logic, buffer_len - 1)
        } else {
            // If buffer_len is 0, target_line_num_for_logic (e.g. from '.') would be 0.
            0
        };

        let insertion_idx = if buffer_len == 0 {
            // If the buffer is currently empty, always insert at the beginning.
            0
        } else if matches!(
            self.address,
            LineAddressType::Absolute(SimpleLineAddressType::LineNumber(0))
        ) {
            // If the address explicitly targets line 0, insert at index 0.
            0
        } else {
            // For any other address, insert *after* the `clamped_dest_line_idx`.
            clamped_dest_line_idx + 1
        };

        // 4. Insert the copied lines.
        if !lines_to_copy.is_empty() {
            editor
                .buffer
                .lines
                .splice(insertion_idx..insertion_idx, lines_to_copy.into_iter());
            self.insertion_idx = Some(insertion_idx);
            self.copied_len = end_idx - start_idx + 1;
        } else {
            self.insertion_idx = None;
            self.copied_len = 0;
        }
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(idx) = self.insertion_idx {
            for _ in 0..self.copied_len {
                if idx < editor.buffer.lines.len() {
                    editor.buffer.lines.remove(idx);
                }
            }
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        let mut new_cmd = Box::new(CopyLines {
            line_range: self.line_range.clone(),
            address: self.address.clone(),
            insertion_idx: None,
            copied_len: 0,
        });
        new_cmd.execute(editor)?;
        Ok(Some(new_cmd))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
