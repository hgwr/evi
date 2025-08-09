use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

// File
// Historical versions of the ex editor file command displayed a current line and number of lines in the edit buffer of 0 when the file was empty, while the vi <control>-G command displayed a current line and number of lines in the edit buffer of 1 in the same situation. POSIX.1-2017 does not permit this discrepancy, instead requiring that a message be displayed indicating that the file is empty.
pub struct DisplayFile;
impl Command for DisplayFile {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let file_name = editor.current_file_name().unwrap_or("No Name".to_string());

        if editor.buffer.lines.is_empty() {
            editor.status_line = format!("\"{}\" -- No lines in buffer --", file_name);
        } else {
            // "file_name" line n of m --p%-- col c char d
            editor.status_line = format!(
                "\"{}\" line {} of {} --{}%-- col {} char {}",
                file_name,
                editor.cursor.row + 1,
                editor.buffer.lines.len(),
                (editor.cursor.row + 1) * 100 / editor.buffer.lines.len(),
                editor.cursor.col + 1,
                editor.buffer.lines[editor.cursor.row]
                    .chars()
                    .nth(editor.cursor.col)
                    .unwrap_or(' ')
            );
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_file_empty_buffer() {
        let mut editor = Editor::new();
        let mut cmd = DisplayFile;
        cmd.execute(&mut editor).unwrap();
        assert_eq!(editor.status_line, "\"No Name\" -- No lines in buffer --");
    }

    #[test]
    fn display_file_with_content() {
        let mut editor = Editor::new();
    editor.buffer.lines = vec!["abc".to_string()];
    editor.cursor.col = 1;
    editor.sync_old_from_new();
        let mut cmd = DisplayFile;
        cmd.execute(&mut editor).unwrap();
        assert_eq!(
            editor.status_line,
            "\"No Name\" line 1 of 1 --100%-- col 2 char b"
        );
    }
}
