use unicode_width::UnicodeWidthChar;

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
        let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
        if editor.cursor_position_in_buffer.col < line.len() {
            let c = line
                .chars()
                .nth(editor.cursor_position_in_buffer.col)
                .unwrap();
            let char_width = UnicodeWidthChar::width(c).unwrap_or(0) as u16;

            editor.cursor_position_on_screen.col += char_width;
            editor.cursor_position_in_buffer.col += 1;

            // If the cursor goes beyond the end of the line, move to the next line
            if editor.cursor_position_in_buffer.col >= line.len()
                && editor.cursor_position_in_buffer.row < editor.buffer.lines.len() - 1
            {
                editor.cursor_position_on_screen.col = 0;
                editor.cursor_position_in_buffer.col += 1;
                if editor.cursor_position_on_screen.row < editor.terminal_size.height - 1 {
                    editor.cursor_position_on_screen.row += 1;
                } else {
                    editor.window_position_in_buffer.row += 1;
                }
            }
        }
    }
}

pub struct LeftOneChar;
impl Command for LeftOneChar {
    fn execute(&mut self, editor: &mut Editor) {
        if editor.cursor_position_in_buffer.col > 0 {
            editor.cursor_position_in_buffer.col -= 1;

            let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
            let c = line
                .chars()
                .nth(editor.cursor_position_in_buffer.col)
                .unwrap();
            let char_width = UnicodeWidthChar::width(c).unwrap_or(0) as u16;

            if editor.cursor_position_on_screen.col >= char_width {
                editor.cursor_position_on_screen.col -= char_width;
            } else {
                // When displaying long line wrapping, when the character cursor returns to the left from the wrapped line, the cursor will move to the line above the wrapped line.
                if editor.cursor_position_on_screen.row > 0 {
                    editor.cursor_position_on_screen.row -= 1;
                } else if editor.window_position_in_buffer.row > 0 {
                    editor.window_position_in_buffer.row -= 1;
                }
            }
        }
    }
}
