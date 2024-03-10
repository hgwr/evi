use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

// File
// Historical versions of the ex editor file command displayed a current line and number of lines in the edit buffer of 0 when the file was empty, while the vi <control>-G command displayed a current line and number of lines in the edit buffer of 1 in the same situation. POSIX.1-2017 does not permit this discrepancy, instead requiring that a message be displayed indicating that the file is empty.
pub struct DisplayFile;
impl Command for DisplayFile {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let file_name = editor.current_file_name().unwrap_or("No Name".to_string());

        // "file_name" line n of m --p%-- col c char d
        editor.status_line = format!(
            "\"{}\" line {} of {} --{}%-- col {} char {}",
            file_name,
            editor.cursor_position_in_buffer.row + 1,
            editor.buffer.lines.len(),
            (editor.cursor_position_in_buffer.row + 1) * 100 / editor.buffer.lines.len(),
            editor.cursor_position_in_buffer.col + 1,
            editor.buffer.lines[editor.cursor_position_in_buffer.row]
                .chars()
                .nth(editor.cursor_position_in_buffer.col)
                .unwrap_or(' ')
        );

        Ok(())
    }
}
