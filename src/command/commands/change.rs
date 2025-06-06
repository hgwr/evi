use std::any::Any;

use crate::command::base::Command;
use crate::command::commands::delete::Delete;
use crate::command::commands::insert::Insert;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Change {
    pub delete: Delete,
    pub insert: Insert,
    pub line_wise: bool,
}

impl Default for Change {
    fn default() -> Self {
        Self {
            delete: Delete::default(),
            insert: Insert::default(),
            line_wise: false,
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
        self.delete.execute(editor)?;
        if self.line_wise {
            editor
                .buffer
                .lines
                .insert(editor.cursor_position_in_buffer.row, String::new());
        }
        self.insert.execute(editor)?;
        Ok(())
    }

    fn set_text(&mut self, text: String) {
        self.insert.set_text(text);
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        self.insert.undo(editor)?;
        if self.line_wise {
            if let Some(cursor) = self.delete.editor_cursor_data {
                editor.buffer.lines.remove(cursor.cursor_position_in_buffer.row);
            }
        }
        self.delete.undo(editor)?;
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let mut new_change = Change {
            delete: self.delete.clone(),
            insert: self.insert.clone(),
            line_wise: self.line_wise,
        };
        new_change.delete.execute(editor)?;
        if new_change.line_wise {
            editor
                .buffer
                .lines
                .insert(editor.cursor_position_in_buffer.row, String::new());
        }
        if let Some(input_text) = &new_change.insert.text {
            for c in input_text.chars() {
                if c == '\n' {
                    editor.append_new_line()?;
                } else {
                    editor.insert_char(c)?;
                }
            }
        }
        Ok(Some(Box::new(new_change)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct RepeatLastCommand;

impl Command for RepeatLastCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.repeat_last_command()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
