use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;
use crate::util::split_line;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OpenLine {
    pub editor_cursor_data: Option<crate::editor::EditorCursorData>,
    pub text: Option<String>,
    pub above: bool,
}

impl Default for OpenLine {
    fn default() -> Self {
        Self {
            editor_cursor_data: None,
            text: None,
            above: false,
        }
    }
}

impl Command for OpenLine {
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
        if editor.is_insert_mode() {
            // do nothing
        } else {
            self.editor_cursor_data = Some(editor.snapshot_cursor_data());
            if self.above {
                editor
                    .buffer
                    .lines
                    .insert(editor.cursor_position_in_buffer.row, String::new());
                let mut previous_line = crate::command::commands::move_cursor::PreviousLine {};
                previous_line.execute(editor)?;
            } else {
                editor
                    .buffer
                    .lines
                    .insert(editor.cursor_position_in_buffer.row + 1, String::new());
                let mut next_line = crate::command::commands::move_cursor::NextLine {};
                next_line.execute(editor)?;
            }
            let mut move_beginning = crate::command::commands::move_cursor::MoveBeginningOfLine {};
            move_beginning.execute(editor)?;
            editor.set_insert_mode();
        }
        Ok(())
    }

    fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(original_cursor_data) = self.editor_cursor_data {
            if let Some(text) = &self.text {
                let start_row = if self.above {
                    original_cursor_data.cursor_position_in_buffer.row
                } else {
                    original_cursor_data.cursor_position_in_buffer.row + 1
                };
                let lines = split_line(text);
                for _ in 0..lines.len() {
                    editor.buffer.lines.remove(start_row);
                }
            }
            editor.restore_cursor_data(original_cursor_data);
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let cursor_before = editor.snapshot_cursor_data();
        let new_open = Box::new(OpenLine {
            editor_cursor_data: Some(cursor_before),
            text: self.text.clone(),
            above: self.above,
        });

        if self.above {
            editor
                .buffer
                .lines
                .insert(editor.cursor_position_in_buffer.row, String::new());
            let mut previous_line = crate::command::commands::move_cursor::PreviousLine {};
            previous_line.execute(editor)?;
        } else {
            editor
                .buffer
                .lines
                .insert(editor.cursor_position_in_buffer.row + 1, String::new());
            let mut next_line = crate::command::commands::move_cursor::NextLine {};
            next_line.execute(editor)?;
        }
        let mut move_beginning = crate::command::commands::move_cursor::MoveBeginningOfLine {};
        move_beginning.execute(editor)?;

        if let Some(input_text) = &self.text {
            for c in input_text.chars() {
                if c == '\n' {
                    editor.append_new_line()?;
                } else {
                    editor.insert_char(c)?;
                }
            }
        }

        Ok(Some(new_open))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
