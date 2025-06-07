use std::any::Any;

use crate::command::base::{Command, JumpCommandData};
use crate::command::region::get_region;
use crate::editor::{Editor, EditorCursorData};
use crate::generic_error::GenericResult;
use crate::util::split_line;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Change {
    pub editor_cursor_data: Option<EditorCursorData>,
    pub deleted_text: Option<String>,
    pub inserted_text: Option<String>,
    pub jump_command_data_opt: Option<JumpCommandData>,
}

fn next_position(start: crate::buffer::CursorPositionInBuffer, text: &str) -> crate::buffer::CursorPositionInBuffer {
    let lines = split_line(text);
    if lines.len() == 1 {
        crate::buffer::CursorPositionInBuffer {
            row: start.row,
            col: start.col + lines[0].len(),
        }
    } else {
        crate::buffer::CursorPositionInBuffer {
            row: start.row + lines.len() - 1,
            col: lines.last().unwrap().len(),
        }
    }
}

impl Default for Change {
    fn default() -> Self {
        Self {
            editor_cursor_data: None,
            deleted_text: None,
            inserted_text: None,
            jump_command_data_opt: None,
        }
    }
}

impl Command for Change {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_modeful(&self) -> bool {
        true
    }

    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.is_dirty = true;
        if let Some(jump_command_data) = self.jump_command_data_opt {
            let region = get_region(editor, jump_command_data)?;
            let start = region.start.cursor_position_in_buffer;
            let end = region.end.cursor_position_in_buffer;
            let deleted = editor.buffer.delete(start, end)?;
            self.deleted_text = Some(deleted);
            editor.restore_cursor_data(region.start);
            self.editor_cursor_data = Some(region.start);
        }
        editor.set_insert_mode();
        Ok(())
    }

    fn set_text(&mut self, text: String) {
        self.inserted_text = Some(text);
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(cursor) = self.editor_cursor_data {
            if let Some(inserted) = &self.inserted_text {
                let start = cursor.cursor_position_in_buffer;
                let end = next_position(start, inserted);
                editor.buffer.delete(start, end)?;
            }
            if let Some(deleted) = &self.deleted_text {
                let row = cursor.cursor_position_in_buffer.row;
                let col = cursor.cursor_position_in_buffer.col;
                editor.buffer.insert(row, col, deleted)?;
            }
            editor.restore_cursor_data(cursor);
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let new_change = Box::new(Change {
            editor_cursor_data: self.editor_cursor_data,
            deleted_text: self.deleted_text.clone(),
            inserted_text: self.inserted_text.clone(),
            jump_command_data_opt: self.jump_command_data_opt,
        });
        // perform delete again
        if let Some(jump_command_data) = self.jump_command_data_opt {
            let region = get_region(editor, jump_command_data)?;
            let start = region.start.cursor_position_in_buffer;
            let end = region.end.cursor_position_in_buffer;
            editor.buffer.delete(start, end)?;
            editor.restore_cursor_data(region.start);
        }
        if let Some(cursor) = self.editor_cursor_data {
            if let Some(inserted) = &self.inserted_text {
                let row = cursor.cursor_position_in_buffer.row;
                let col = cursor.cursor_position_in_buffer.col;
                editor.buffer.insert(row, col, inserted)?;
            }
            if let Some(deleted) = &self.deleted_text {
                editor.buffer.insert(cursor.cursor_position_in_buffer.row, cursor.cursor_position_in_buffer.col, deleted)?;
            }
            editor.restore_cursor_data(cursor);
        }
        Ok(Some(new_change))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::Editor;
    use crossterm::event::KeyCode;

    #[test]
    fn change_word() {
        let mut editor = Editor::new();
        editor.terminal_size = crate::editor::TerminalSize { width: 80, height: 24 };
        editor.buffer.lines = vec!["hello world".to_string()];
        let mut cmd = Change {
            jump_command_data_opt: Some(JumpCommandData { count: 1, key_code: KeyCode::Char('w'), modifiers: crossterm::event::KeyModifiers::NONE }),
            ..Default::default()
        };
        cmd.execute(&mut editor).unwrap();
        // simulate typing text and leaving insert mode
        editor.insert_char('X').unwrap();
        editor.set_command_mode();
        cmd.set_text(editor.last_input_string.clone());
        assert_eq!(editor.buffer.lines[0], "Xworld");
        // undo using command
        cmd.undo(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines[0], "hello world");
    }
}
