use core::num;

use unicode_width::UnicodeWidthChar;

use crate::command::base::Command;
use crate::editor::Editor;

pub struct ForwardChar;
impl Command for ForwardChar {
    fn execute(&mut self, editor: &mut Editor) {
        let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
        let num_of_chars = line.chars().count();
        if editor.cursor_position_in_buffer.col < num_of_chars {
            editor.cursor_position_in_buffer.col += 1;

            let c = line
                .chars()
                .nth(editor.cursor_position_in_buffer.col)
                .unwrap();
            let char_width = UnicodeWidthChar::width(c).unwrap_or(0) as u16;
            editor.cursor_position_on_screen.col += char_width;

            if editor.cursor_position_on_screen.col >= editor.terminal_size.width {
                editor.cursor_position_on_screen.col = 0;
                if editor.cursor_position_on_screen.row < editor.terminal_size.height {
                    editor.cursor_position_on_screen.row += 1;
                } else {
                    editor.window_position_in_buffer.row += 1;
                }
            }
        }
    }
}

pub struct BackwardChar;
impl Command for BackwardChar {
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
                if editor.cursor_position_on_screen.row > 0 {
                    editor.cursor_position_on_screen.row -= 1;
                } else if editor.window_position_in_buffer.row > 0 {
                    editor.window_position_in_buffer.row -= 1;
                }
            }
        }
    }
}

pub struct NextLine;
impl Command for NextLine {
    fn execute(&mut self, editor: &mut Editor) {
        let next_row = editor.cursor_position_in_buffer.row + 1;
        if next_row < editor.buffer.lines.len() {
            editor.cursor_position_in_buffer.row = next_row;

            if editor.cursor_position_on_screen.row < editor.terminal_size.height - 1 {
                editor.cursor_position_on_screen.row += 1;
            } else {
                editor.window_position_in_buffer.row += 1;
            }

            let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
            let num_of_chars = line.chars().count();
            if editor.cursor_position_in_buffer.col > num_of_chars {
                editor.cursor_position_in_buffer.col = num_of_chars;
            }
            if editor.cursor_position_in_buffer.col >= editor.terminal_size.width as usize {
                editor.cursor_position_on_screen.col = editor.terminal_size.width;
                editor.cursor_position_in_buffer.col = editor.terminal_size.width as usize;
            }
        }
    }
}

pub struct PreviousLine;
impl Command for PreviousLine {
    fn execute(&mut self, editor: &mut Editor) {
        if editor.cursor_position_in_buffer.row > 0 {
            editor.cursor_position_in_buffer.row -= 1;

            if editor.cursor_position_on_screen.row > 0 {
                editor.cursor_position_on_screen.row -= 1;
            } else if editor.window_position_in_buffer.row > 0 {
                editor.window_position_in_buffer.row -= 1;
            }

            let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
            let num_of_chars = line.chars().count();
            if editor.cursor_position_in_buffer.col > num_of_chars {
                editor.cursor_position_in_buffer.col = num_of_chars;
            }
            if editor.cursor_position_in_buffer.col >= editor.terminal_size.width as usize {
                editor.cursor_position_on_screen.col = editor.terminal_size.width;
                editor.cursor_position_in_buffer.col = editor.terminal_size.width as usize;
            }
        }
    }
}

pub struct MoveBeginningOfLine;
impl Command for MoveBeginningOfLine {
    fn execute(&mut self, editor: &mut Editor) {
        let mut backward_char = BackwardChar {};
        while editor.cursor_position_in_buffer.col > 0 {
            backward_char.execute(editor);
        }
    }
}

pub struct MoveEndOfLine;
impl Command for MoveEndOfLine {
    fn execute(&mut self, editor: &mut Editor) {
        let mut forward_char = ForwardChar {};
        let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
        let num_of_chars = line.chars().count();
        while editor.cursor_position_in_buffer.col < num_of_chars {
            forward_char.execute(editor);
        }
    }
}
