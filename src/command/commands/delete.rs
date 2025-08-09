use std::any::Any;

use crate::command::base::Command;
use crate::command::region::get_region;
use crate::editor::{Editor, NewCursorSnapshot};
use crate::generic_error::GenericResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeleteChar {
    pub snapshot: Option<NewCursorSnapshot>,
    pub char: Option<char>,
}

impl Default for DeleteChar {
    fn default() -> Self {
    Self { snapshot: None, char: None }
    }
}

impl Command for DeleteChar {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.is_dirty = true;
        editor.sync_new_from_old();
        let row = editor.cursor.row;
        let col = editor.cursor.col;
        self.snapshot = Some(editor.snapshot_new_cursor());
        if let Some(line) = editor.buffer.lines.get(row).cloned() {
            let num = line.chars().count();
            if col < num { // cursor sits on a valid char index
                let ch = line.chars().nth(col).unwrap();
                self.char = Some(ch);
                let new_line: String = line
                    .chars()
                    .take(col)
                    .chain(line.chars().skip(col + 1))
                    .collect();
                editor.buffer.lines[row] = new_line.clone();
                let new_len = new_line.chars().count();
                if col >= new_len {
                    editor.cursor.col = if new_len == 0 { 0 } else { new_len - 1 };
                }
            }
        }
        editor.ensure_cursor_visible();
        editor.sync_old_from_new();
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
    let snap = self.snapshot.unwrap();
    let row = snap.cursor.row;
    let col = snap.cursor.col;
        let char = self.char.unwrap();

        let line = &editor.buffer.lines[row];
        let new_line: String = line
            .chars()
            .take(col)
            .chain(std::iter::once(char))
            .chain(line.chars().skip(col))
            .collect();
        editor.buffer.lines[row] = new_line;
    editor.restore_new_cursor(snap);

        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let mut new_delete = Box::new(DeleteChar::default());
        new_delete.execute(editor)?;
        Ok(Some(new_delete))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Delete {
    pub snapshot: Option<NewCursorSnapshot>,
    pub text: Option<String>,
    pub jump_command_data_opt: Option<crate::command::base::JumpCommandData>,
}

impl Default for Delete {
    fn default() -> Self {
    Self { snapshot: None, text: None, jump_command_data_opt: None }
    }
}

impl Command for Delete {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(jump_command_data) = self.jump_command_data_opt {
            let region = get_region(editor, jump_command_data);
            if let Ok(region) = region {
                let start = region.start; // NewCursorSnapshot
                let end = region.end;
                use crate::buffer::CursorPositionInBuffer;
                let start_pos = CursorPositionInBuffer { row: start.cursor.row, col: start.cursor.col };
                let end_pos = CursorPositionInBuffer { row: end.cursor.row, col: end.cursor.col };
                if let Ok(deleted) = editor.buffer.delete(start_pos, end_pos) {
                    self.text = Some(deleted);
                    // determine min position
                    let (min_snap, _) = if (start.cursor.row, start.cursor.col) <= (end.cursor.row, end.cursor.col) { (start, end) } else { (end, start) };
                    editor.restore_new_cursor(min_snap);
                    self.snapshot = Some(min_snap);
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(text) = &self.text {
            if let Some(snap) = &self.snapshot {
                let row = snap.cursor.row;
                let col = snap.cursor.col;
                editor.buffer.insert(row, col, text)?;
                editor.restore_new_cursor(*snap);
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct DeleteLines {
    pub snapshot: Option<NewCursorSnapshot>,
    pub line_range: crate::data::LineRange,
    pub text: Option<String>,
}

impl Command for DeleteLines {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let start_address = self.line_range.start.clone();
        let end_address = self.line_range.end.clone();
        let start_row = editor.get_line_number_from(&start_address);
        let end_row = editor.get_line_number_from(&end_address);

        let start_cursor_data = crate::buffer::CursorPositionInBuffer {
            row: start_row,
            col: 0,
        };
        let end_cursor_data = crate::buffer::CursorPositionInBuffer {
            row: end_row,
            col: 0,
        };

    editor.sync_new_from_old();
    // snapshot before deletion but adjust cursor to start row
    let mut snap = editor.snapshot_new_cursor();
    snap.cursor.row = start_row;
    snap.cursor.col = 0;
    self.snapshot = Some(snap);

        if let Ok(deleted) = editor.buffer.delete(
            start_cursor_data,
            end_cursor_data,
        ) {
            self.text = Some(deleted);
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
    if let Some(snap) = &self.snapshot {
            if let Some(text) = &self.text {
        let row = snap.cursor.row;
        let col = snap.cursor.col;
                editor.buffer.insert(row, col, text)?;
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}