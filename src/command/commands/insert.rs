use log::info;

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
        if let Some(original_cursor_data) = self.editor_cursor_data {
            if let Some(text) = &self.text {
                let row = original_cursor_data.cursor_position_in_buffer.row;
                let col = original_cursor_data.cursor_position_in_buffer.col;
                let mut input_text_lines: Vec<&str> = text.lines().collect();
                if input_text_lines.len() == 0 {
                    panic!("input_text_lines.len() == 0, text: '{:?}'", text);
                }
                info!("input_text_lines: {:?}", input_text_lines);
                info!("input_text_lines.len(): {:?}", input_text_lines.len());
                // TODO: fix bug
                if text == "\n" {
                    input_text_lines = vec!["", ""];
                }
                if input_text_lines.len() == 1 {
                    let line = &editor.buffer.lines[row];
                    let new_line: String = line
                        .chars()
                        .take(col)
                        .chain(line.chars().skip(col + text.len()))
                        .collect();
                    editor.buffer.lines[row] = new_line;
                } else if input_text_lines.len() >= 2 {
                    let first_line = editor.buffer.lines[row].clone();
                    let last_line = editor.buffer.lines[row + input_text_lines.len() - 1].clone();
                    let new_first_line: String = first_line
                        .chars()
                        .take(col)
                        .chain(first_line.chars().skip(col + input_text_lines[0].len()))
                        .collect();
                    let new_last_line: String = last_line
                        .chars()
                        .take(input_text_lines[input_text_lines.len() - 1].len())
                        .chain(
                            last_line
                                .chars()
                                .skip(input_text_lines[input_text_lines.len() - 1].len()),
                        )
                        .collect();
                    editor.buffer.lines[row] = new_first_line + new_last_line.as_str();
                    for _ in 0..input_text_lines.len() - 1 {
                        editor.buffer.lines.remove(row + 1);
                    }
                }
            }
            editor.restore_cursor_data(original_cursor_data);
        }
        Ok(())
    }
}
