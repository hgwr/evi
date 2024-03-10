use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeleteChar {
    pub editor_cursor_data: Option<crate::editor::EditorCursorData>,
    pub char: Option<char>,
}

impl Default for DeleteChar {
    fn default() -> Self {
        Self {
            editor_cursor_data: None,
            char: None,
        }
    }
}

impl Command for DeleteChar {
    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.is_dirty = true;
        let row = editor.cursor_position_in_buffer.row;
        let col = editor.cursor_position_in_buffer.col;
        self.editor_cursor_data = Some(editor.snapshot_cursor_data());
        let line = &editor.buffer.lines[row];
        let num_of_chars = line.chars().count();
        if col < num_of_chars {
            let char = line.chars().nth(col).unwrap();
            self.char = Some(char);
            let new_line: String = line
                .chars()
                .take(col)
                .chain(line.chars().skip(col + 1))
                .collect();
            let new_num_of_chars = new_line.chars().count();
            editor.buffer.lines[row] = new_line;
            if col >= new_num_of_chars && new_num_of_chars > 0 {
                editor.cursor_position_in_buffer.col = new_num_of_chars - 1;
                if editor.cursor_position_on_screen.col > 0 {
                    editor.cursor_position_on_screen.col -= 1;
                } else {
                    if editor.cursor_position_on_screen.row > 0 {
                        editor.cursor_position_on_screen.row -= 1;
                    } else if editor.window_position_in_buffer.row > 0 {
                        editor.window_position_in_buffer.row -= 1;
                    }
                    editor.cursor_position_on_screen.col = editor.terminal_size.width - 1;
                }
            }
        }
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let editor_cursor_data = self.editor_cursor_data.unwrap();
        let row = editor_cursor_data.cursor_position_in_buffer.row;
        let col = editor_cursor_data.cursor_position_in_buffer.col;
        let char = self.char.unwrap();

        let line = &editor.buffer.lines[row];
        let new_line: String = line
            .chars()
            .take(col)
            .chain(std::iter::once(char))
            .chain(line.chars().skip(col))
            .collect();
        editor.buffer.lines[row] = new_line;
        editor.restore_cursor_data(editor_cursor_data);

        Ok(())
    }
}