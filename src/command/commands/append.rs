use log::info;
use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;
use crate::util::{get_char_width, split_line};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Append {
    pub editor_cursor_data: Option<crate::editor::EditorCursorData>,
    pub text: Option<String>,
}

impl Default for Append {
    fn default() -> Self {
        Self {
            editor_cursor_data: None,
            text: None,
        }
    }
}

impl Command for Append {
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
            if let Some(c) = editor.get_current_char() {
                editor.cursor_position_in_buffer.col += 1;
                editor.cursor_position_on_screen.col += get_char_width(c);
                if editor.cursor_position_on_screen.col >= editor.terminal_size.width {
                    editor.cursor_position_on_screen.col = 0;
                    if editor.cursor_position_on_screen.row < editor.max_content_row_index() {
                        editor.cursor_position_on_screen.row += 1;
                    } else {
                        editor.window_position_in_buffer.row += 1;
                    }
                }
                self.editor_cursor_data = Some(editor.snapshot_cursor_data());
                editor.set_insert_mode();
            } else {
                return Err("Failed to get current char".into());
            }
        }
        Ok(())
    }

    fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(original_cursor_data) = self.editor_cursor_data {
            if let Some(text) = &self.text {
                let row = original_cursor_data.cursor_position_in_buffer.row;
                let col = original_cursor_data.cursor_position_in_buffer.col;
                let input_text_lines: Vec<&str> = split_line(text);
                if input_text_lines.len() == 0 {
                    panic!("input_text_lines.len() == 0, text: '{:?}'", text);
                }
                info!("input_text_lines: {:?}", input_text_lines);
                info!("input_text_lines.len(): {:?}", input_text_lines.len());
                if input_text_lines.len() == 1 {
                    let line = &editor.buffer.lines[row];
                    let new_line: String = line
                        .chars()
                        .take(col)
                        .chain(line.chars().skip(col + text.len()))
                        .collect();
                    editor.buffer.lines[row] = new_line;
                } else if input_text_lines.len() >= 2 {
                    let last_input_line = input_text_lines[input_text_lines.len() - 1];
                    let first_line = editor.buffer.lines[row].clone();
                    let last_line = editor.buffer.lines[row + input_text_lines.len() - 1].clone();
                    let new_first_line: String = first_line.chars().take(col).collect();
                    let new_last_line: String =
                        last_line.chars().skip(last_input_line.len()).collect();
                    editor.buffer.lines[row] = new_first_line + new_last_line.as_str();
                    for _ in 0..input_text_lines.len() - 1 {
                        editor.buffer.lines.remove(row + 1);
                    }
                }
            }
            editor.restore_cursor_data(original_cursor_data);
            let mut backward_char = crate::command::commands::move_cursor::BackwardChar {};
            backward_char.execute(editor)?;
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
        let new_insert = Box::new(Append {
            editor_cursor_data: self.editor_cursor_data,
            text: self.text.clone(),
        });

        if let Some(input_text) = &self.text {
            for c in input_text.chars() {
                if c == '\n' {
                    editor.append_new_line()?
                } else {
                    editor.insert_char(c)?;
                }
            }
        }

        Ok(Some(new_insert))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
