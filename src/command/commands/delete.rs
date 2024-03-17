use crate::command::base::Command;
use crate::command::factory::command_factory;
use crate::editor::Editor;
use crate::generic_error::GenericResult;
use crate::util::split_line;

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
    fn is_reusable(&self) -> bool {
        false
    }

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

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let mut new_delete = Box::new(DeleteChar::default());
        new_delete.execute(editor)?;
        Ok(Some(new_delete))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Delete {
    pub editor_cursor_data: Option<crate::editor::EditorCursorData>,
    pub text: Option<String>,
    pub jump_command_data_opt: Option<crate::command::base::JumpCommandData>,
}

impl Default for Delete {
    fn default() -> Self {
        Self {
            editor_cursor_data: None,
            text: None,
            jump_command_data_opt: None,
        }
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
        let start_cursor_data = editor.snapshot_cursor_data();
        if let Some(jump_command_data) = self.jump_command_data_opt {
            let command_data: crate::command::base::CommandData = jump_command_data.into();
            for _ in 0..command_data.count {
                let mut jump_command = command_factory(&command_data);
                jump_command.execute(editor)?;
            }
            let end_cursor_data = editor.snapshot_cursor_data();
            if let Ok(deleted) = editor.buffer.delete(
                start_cursor_data.cursor_position_in_buffer,
                end_cursor_data.cursor_position_in_buffer,
            ) {
                self.text = Some(deleted);
                if start_cursor_data
                    .cursor_position_in_buffer
                    .cmp(&end_cursor_data.cursor_position_in_buffer)
                    == std::cmp::Ordering::Greater
                {
                    editor.restore_cursor_data(end_cursor_data);
                    self.editor_cursor_data = Some(end_cursor_data);
                } else {
                    editor.restore_cursor_data(start_cursor_data);
                    self.editor_cursor_data = Some(start_cursor_data);
                }
            }
        }

        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(text) = &self.text {
            if let Some(cursor_data) = &self.editor_cursor_data {
                let lines = split_line(text);
                let mut row = cursor_data.cursor_position_in_buffer.row;
                let mut col = cursor_data.cursor_position_in_buffer.col;
                for line in lines {
                    let line_length = line.chars().count();
                    if line_length > 0 {
                        let mut new_line = editor.buffer.lines[row].clone();
                        new_line.insert_str(col, line);
                        editor.buffer.lines[row] = new_line;
                        col += line_length;
                    } else {
                        editor.buffer.lines.insert(row, line.to_string());
                    }
                    row += 1;
                }
                editor.restore_cursor_data(*cursor_data);
            }
        }
        Ok(())
    }
}
