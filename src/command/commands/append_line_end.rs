use std::any::Any;

use crate::command::base::Command;
use crate::command::commands::move_cursor::MoveEndOfLine;
use crate::editor::{Editor, EditorCursorData};
use crate::generic_error::GenericResult;
use crate::util::{get_char_width, split_line};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct AppendLineEnd {
    pub insert_cursor_data: Option<EditorCursorData>,
    pub original_cursor_data: Option<EditorCursorData>,
    pub text: Option<String>,
}

impl Default for AppendLineEnd {
    fn default() -> Self {
        Self {
            insert_cursor_data: None,
            original_cursor_data: None,
            text: None,
        }
    }
}

impl Command for AppendLineEnd {
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
        if !editor.is_insert_mode() {
            self.original_cursor_data = Some(editor.snapshot_cursor_data());
            let mut move_eol = MoveEndOfLine {};
            move_eol.execute(editor)?;
            let width = editor
                .get_current_char()
                .map(get_char_width)
                .unwrap_or(0);
            editor.cursor_position_in_buffer.col += 1;
            editor.cursor_position_on_screen.col += width;
            if editor.cursor_position_on_screen.col >= editor.terminal_size.width {
                editor.cursor_position_on_screen.col = 0;
                if editor.cursor_position_on_screen.row < editor.max_content_row_index() {
                    editor.cursor_position_on_screen.row += 1;
                } else {
                    editor.window_position_in_buffer.row += 1;
                }
            }
            self.insert_cursor_data = Some(editor.snapshot_cursor_data());
            editor.set_insert_mode();
        }
        Ok(())
    }

    fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let (Some(insert_cd), Some(original_cd)) =
            (self.insert_cursor_data, self.original_cursor_data)
        {
            if let Some(text) = &self.text {
                if text.is_empty() {
                    editor.restore_cursor_data(original_cd);
                    return Ok(());
                }
                let row = insert_cd.cursor_position_in_buffer.row;
                let col = insert_cd.cursor_position_in_buffer.col;
                let input_text_lines: Vec<&str> = split_line(text);
                if input_text_lines.is_empty() {
                    editor.restore_cursor_data(original_cd);
                    return Ok(());
                }
                if input_text_lines.len() == 1 {
                    let line = &editor.buffer.lines[row];
                    let new_line: String = line
                        .chars()
                        .take(col)
                        .chain(line.chars().skip(col + text.chars().count()))
                        .collect();
                    editor.buffer.lines[row] = new_line;
                } else {
                    let last_input_line = input_text_lines[input_text_lines.len() - 1];
                    let first_line = editor.buffer.lines[row].clone();
                    let last_line = editor.buffer.lines[row + input_text_lines.len() - 1].clone();
                    let new_first_line: String = first_line.chars().take(col).collect();
                    let new_last_line: String =
                        last_line.chars().skip(last_input_line.chars().count()).collect();
                    editor.buffer.lines[row] = new_first_line + new_last_line.as_str();
                    for _ in 0..input_text_lines.len() - 1 {
                        editor.buffer.lines.remove(row + 1);
                    }
                }
            }
            editor.restore_cursor_data(original_cd);
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let new_cmd = Box::new(AppendLineEnd {
            insert_cursor_data: self.insert_cursor_data,
            original_cursor_data: self.original_cursor_data,
            text: self.text.clone(),
        });

        let mut move_eol = MoveEndOfLine {};
        move_eol.execute(editor)?;
        let width = editor
            .get_current_char()
            .map(get_char_width)
            .unwrap_or(0);
        editor.cursor_position_in_buffer.col += 1;
        editor.cursor_position_on_screen.col += width;
        if editor.cursor_position_on_screen.col >= editor.terminal_size.width {
            editor.cursor_position_on_screen.col = 0;
            if editor.cursor_position_on_screen.row < editor.max_content_row_index() {
                editor.cursor_position_on_screen.row += 1;
            } else {
                editor.window_position_in_buffer.row += 1;
            }
        }

        if let Some(input_text) = &self.text {
            if input_text.is_empty() {
                return Ok(Some(new_cmd));
            }
            for c in input_text.chars() {
                if c == '\n' {
                    editor.append_new_line()?;
                } else {
                    editor.insert_char(c)?;
                }
            }
        }

        Ok(Some(new_cmd))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor::Editor;

    #[test]
    fn append_line_end() {
        let mut editor = Editor::new();
        editor.terminal_size = crate::editor::TerminalSize { width: 80, height: 24 };
        editor.buffer.lines = vec!["hello".to_string()];
        editor.cursor_position_in_buffer.col = 1;

        let mut cmd = AppendLineEnd::default();
        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 5);
        editor.insert_char('X').unwrap();
        editor.set_command_mode();
        cmd.set_text(editor.last_input_string.clone());
        assert_eq!(editor.buffer.lines[0], "helloX");
        cmd.undo(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines[0], "hello");
        assert_eq!(editor.cursor_position_in_buffer.col, 1);
    }

    #[test]
    fn append_line_end_empty_string() {
        let mut editor = Editor::new();
        editor.terminal_size = crate::editor::TerminalSize { width: 80, height: 24 };
        editor.buffer.lines = vec!["hello".to_string()];
        let mut cmd = AppendLineEnd::default();
        cmd.execute(&mut editor).unwrap();
        editor.set_command_mode();
        cmd.set_text(String::new());
        let before = editor.buffer.lines.clone();
        cmd.redo(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines, before);
        cmd.undo(&mut editor).unwrap();
        assert_eq!(editor.buffer.lines, before);
    }
}

