use std::any::Any;

use crate::command::base::Command;
use crate::editor::{Editor, EditorCursorData};
use crate::generic_error::GenericResult;
use crate::util::split_line;
use crate::buffer::CursorPositionInBuffer;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Paste {
    pub before: bool,
    pub editor_cursor_data: Option<EditorCursorData>,
    pub text: Option<String>,
}

impl Default for Paste {
    fn default() -> Self {
        Self {
            before: false,
            editor_cursor_data: None,
            text: None,
        }
    }
}

fn next_position(start: CursorPositionInBuffer, text: &str) -> CursorPositionInBuffer {
    let lines = split_line(text);
    if lines.len() == 1 {
        CursorPositionInBuffer {
            row: start.row,
            col: start.col + lines[0].len(),
        }
    } else {
        CursorPositionInBuffer {
            row: start.row + lines.len() - 1,
            col: lines.last().unwrap().len(),
        }
    }
}

impl Command for Paste {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let text = editor.unnamed_register.clone();
        if text.is_empty() {
            return Ok(());
        }
        self.text = Some(text.clone());
        self.editor_cursor_data = Some(editor.snapshot_cursor_data());
        if editor.unnamed_register_linewise {
            let row = if self.before {
                editor.cursor_position_in_buffer.row
            } else {
                editor.cursor_position_in_buffer.row + 1
            };
            if row >= editor.buffer.lines.len() {
                editor.buffer.lines.push(String::new());
            }
            editor.buffer.insert(row, 0, &text)?;
            editor.move_cursor_to(row, 0)?;
        } else {
            let row = editor.cursor_position_in_buffer.row;
            let col = if self.before {
                editor.cursor_position_in_buffer.col
            } else {
                editor.cursor_position_in_buffer.col + 1
            };
            editor.buffer.insert(row, col, &text)?;
            let end = next_position(CursorPositionInBuffer { row, col }, &text);
            if self.before {
                editor.move_cursor_to(row, col)?;
            } else {
                let end_pos = if end.col == 0 { end } else { CursorPositionInBuffer { row: end.row, col: end.col - 1 } };
                editor.move_cursor_to(end_pos.row, end_pos.col)?;
            }
        }
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let (Some(orig), Some(text)) = (self.editor_cursor_data, &self.text) {
            if editor.unnamed_register_linewise {
                let row = if self.before {
                    orig.cursor_position_in_buffer.row
                } else {
                    orig.cursor_position_in_buffer.row + 1
                };
                let start = CursorPositionInBuffer { row, col: 0 };
                let end = next_position(start, text);
                editor.buffer.delete(start, end)?;
            } else {
                let row = orig.cursor_position_in_buffer.row;
                let col = if self.before {
                    orig.cursor_position_in_buffer.col
                } else {
                    orig.cursor_position_in_buffer.col + 1
                };
                let start = CursorPositionInBuffer { row, col };
                let end = next_position(start, text);
                editor.buffer.delete(start, end)?;
            }
            editor.restore_cursor_data(orig);
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let mut new_cmd = Box::new(Paste {
            before: self.before,
            editor_cursor_data: None,
            text: self.text.clone(),
        });
        new_cmd.execute(editor)?;
        Ok(Some(new_cmd))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
