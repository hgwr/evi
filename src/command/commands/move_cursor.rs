use crate::command::base::Command;
use crate::editor::Editor;

pub struct DownOneLine;
impl Command for DownOneLine {
    fn execute(&mut self, editor: &mut Editor) {
        let next_row = editor.cursor_position_in_buffer.row + 1;
        if next_row < editor.buffer.lines.len() {
            editor.cursor_position_in_buffer.row = next_row;

            if editor.cursor_position_on_screen.row < editor.terminal_size.height - 1 {
                editor.cursor_position_on_screen.row += 1;
            } else {
                editor.window_position_in_buffer.row += 1;
            }
        }
    }
}

pub struct UpOneLine;
impl Command for UpOneLine {
    fn execute(&mut self, editor: &mut Editor) {
        if editor.cursor_position_in_buffer.row > 0 {
            editor.cursor_position_in_buffer.row -= 1;

            if editor.cursor_position_on_screen.row > 0 {
                editor.cursor_position_on_screen.row -= 1;
            } else if editor.window_position_in_buffer.row > 0 {
                editor.window_position_in_buffer.row -= 1;
            }
        }
    }
}

pub struct RightOneChar;
impl Command for RightOneChar {
    fn execute(&mut self, editor: &mut Editor) {
        editor.cursor_position_on_screen.col += 1;
    }
}

pub struct LeftOneChar;
impl Command for LeftOneChar {
    fn execute(&mut self, editor: &mut Editor) {
        if editor.cursor_position_on_screen.col == 0 {
            return;
        }
        editor.cursor_position_on_screen.col -= 1;
    }
}
