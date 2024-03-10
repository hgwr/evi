use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Insert {
    pub editor_cursor_data: Option<crate::editor::EditorCursorData>,
    pub text: Option<String>,
}

impl Default for Insert {
    fn default() -> Self {
        Self {
            editor_cursor_data: None,
            text: None,
        }
    }
}

impl Command for Insert {
    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.is_dirty = true;
        if editor.is_insert_mode() {
            // do nothing
        } else {
            self.editor_cursor_data = Some(editor.snapshot_cursor_data());
            editor.set_insert_mode();
        }
        Ok(())
    }

    fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(editor_cursor_data) = self.editor_cursor_data {
            let row = editor_cursor_data.cursor_position_in_buffer.row;
            let col = editor_cursor_data.cursor_position_in_buffer.col;
            if let Some(text) = &self.text {
                let line = &editor.buffer.lines[row];
                let new_line: String = line
                    .chars()
                    .take(col)
                    .chain(line.chars().skip(col + text.len()))
                    .collect();
                editor.buffer.lines[row] = new_line;
            }
            editor.restore_cursor_data(editor_cursor_data);
        }
        Ok(())
    }
}
