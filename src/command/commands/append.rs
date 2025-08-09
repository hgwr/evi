use log::info;
use std::any::Any;

use crate::command::base::Command;
use crate::editor::{Editor, NewCursorSnapshot};
use crate::generic_error::GenericResult;
use crate::util::split_line;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Append {
    pub snapshot: Option<NewCursorSnapshot>,
    pub text: Option<String>,
}

impl Default for Append {
    fn default() -> Self {
    Self { snapshot: None, text: None }
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
            // sync old->new first (while transitional)
            editor.sync_new_from_old();
            if let Some(_c) = editor.get_current_char() {
                editor.cursor.col += 1; // move after current char
                editor.ensure_cursor_visible();
                editor.sync_old_from_new();
                self.snapshot = Some(editor.snapshot_new_cursor());
                editor.set_insert_mode();
            } else { return Err("Failed to get current char".into()); }
        }
        Ok(())
    }

    fn set_text(&mut self, text: String) {
        self.text = Some(text);
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
    if let Some(snap) = self.snapshot {
            if let Some(text) = &self.text {
        let row = snap.cursor.row;
        let col = snap.cursor.col;
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
            editor.restore_new_cursor(snap);
            let mut backward_char = crate::command::commands::move_cursor::BackwardChar {};
            backward_char.execute(editor)?;
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        editor.is_dirty = true;
    let new_insert = Box::new(Append { snapshot: self.snapshot, text: self.text.clone() });

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
