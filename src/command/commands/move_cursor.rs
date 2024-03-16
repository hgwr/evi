use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;
use crate::util::get_char_width;

pub struct ForwardChar;
impl Command for ForwardChar {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
        let num_of_chars = line.chars().count();
        if editor.cursor_position_in_buffer.col + 1 < num_of_chars {
            let c = line
                .chars()
                .nth(editor.cursor_position_in_buffer.col)
                .unwrap();
            let char_width = get_char_width(c);

            editor.cursor_position_in_buffer.col += 1;
            editor.cursor_position_on_screen.col += char_width;
            if editor.cursor_position_on_screen.col >= editor.terminal_size.width {
                editor.cursor_position_on_screen.col = 0;
                if editor.cursor_position_on_screen.row < editor.content_height() {
                    editor.cursor_position_on_screen.row += 1;
                } else {
                    editor.window_position_in_buffer.row += 1;
                }
            }
        }
        Ok(())
    }
}

pub struct BackwardChar;
impl Command for BackwardChar {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.cursor_position_in_buffer.col > 0 {
            editor.cursor_position_in_buffer.col -= 1;
            let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
            let c = line
                .chars()
                .nth(editor.cursor_position_in_buffer.col)
                .unwrap();
            let char_width = get_char_width(c);
            if editor.cursor_position_on_screen.col >= char_width {
                editor.cursor_position_on_screen.col -= char_width;
            } else {
                if editor.cursor_position_on_screen.row > 0 {
                    editor.cursor_position_on_screen.row -= 1;
                } else if editor.window_position_in_buffer.row > 0 {
                    editor.window_position_in_buffer.row -= 1;
                }
                editor.cursor_position_on_screen.col = editor.terminal_size.width - char_width;
            }
        }
        Ok(())
    }
}

pub struct MoveBeginningOfLine;
impl Command for MoveBeginningOfLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let mut backward_char = BackwardChar {};
        while editor.cursor_position_in_buffer.col > 0 {
            backward_char.execute(editor)?;
        }
        Ok(())
    }
}

pub struct MoveEndOfLine;
impl Command for MoveEndOfLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
        let num_of_chars = line.chars().count();
        let mut forward_char = ForwardChar {};
        while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
            forward_char.execute(editor)?;
        }
        Ok(())
    }
}

pub struct NextLine;
impl Command for NextLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let next_row = editor.cursor_position_in_buffer.row + 1;
        if next_row < editor.buffer.lines.len() {
            let current_cursor_col_in_buffer = editor.cursor_position_in_buffer.col;
            let mut move_end_of_line = MoveEndOfLine {};
            move_end_of_line.execute(editor)?;

            editor.cursor_position_in_buffer.row = next_row;
            if editor.cursor_position_on_screen.row < editor.content_height() {
                editor.cursor_position_on_screen.row += 1;
            } else {
                editor.window_position_in_buffer.row += 1;
            }

            let current_line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
            let num_of_chars_of_current_line = current_line.chars().count();
            let destination_col = if current_cursor_col_in_buffer > num_of_chars_of_current_line {
                num_of_chars_of_current_line
            } else {
                current_cursor_col_in_buffer
            };
            editor.cursor_position_in_buffer.col = 0;
            editor.cursor_position_on_screen.col = 0;
            editor.window_position_in_buffer.col = 0;
            let mut forward_char = ForwardChar {};
            for _ in 0..destination_col {
                forward_char.execute(editor)?;
            }
        }
        Ok(())
    }
}

pub struct PreviousLine;
impl Command for PreviousLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.cursor_position_in_buffer.row > 0 {
            let current_cursor_col_in_buffer = editor.cursor_position_in_buffer.col;
            if current_cursor_col_in_buffer > 0 {
                let mut move_beginning_of_line = MoveBeginningOfLine {};
                move_beginning_of_line.execute(editor)?;
            }

            editor.cursor_position_in_buffer.row -= 1;

            let line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
            let num_of_chars = line.chars().count();
            let num_of_lines_on_screen = if num_of_chars == 0 {
                1
            } else if num_of_chars % editor.terminal_size.width as usize == 0 {
                num_of_chars / editor.terminal_size.width as usize
            } else {
                num_of_chars / editor.terminal_size.width as usize + 1
            };

            if editor.cursor_position_on_screen.row >= num_of_lines_on_screen as u16 {
                editor.cursor_position_on_screen.row -= num_of_lines_on_screen as u16;
            } else if editor.window_position_in_buffer.row >= num_of_lines_on_screen {
                editor.window_position_in_buffer.row -= num_of_lines_on_screen;
            } else {
                editor.window_position_in_buffer.row = 0;
            }

            let mut forward_char = ForwardChar {};
            for _ in 0..current_cursor_col_in_buffer {
                forward_char.execute(editor)?;
            }
        }
        Ok(())
    }
}

pub struct ForwardWord;
impl Command for ForwardWord {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let mut forward_char = ForwardChar {};
        let mut num_of_chars = editor.num_of_current_line_chars();
        forward_char.execute(editor)?;
        'outer: loop {
            if editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.current_char().unwrap();
                    if c.is_whitespace() {
                        break;
                    }
                    forward_char.execute(editor)?;
                }
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.current_char().unwrap();
                    if !c.is_whitespace() {
                        break 'outer;
                    }
                    forward_char.execute(editor)?;
                }
            } else if editor.cursor_position_in_buffer.row + 1 < editor.buffer.lines.len() {
                let mut next_line = NextLine {};
                next_line.execute(editor)?;
                let mut move_beginning_of_line = MoveBeginningOfLine {};
                move_beginning_of_line.execute(editor)?;
                num_of_chars = editor.num_of_current_line_chars();
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.current_char().unwrap();
                    if !c.is_whitespace() {
                        break 'outer;
                    }
                    forward_char.execute(editor)?;
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}

pub struct BackwardWord;
impl Command for BackwardWord {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let mut backward_char = BackwardChar {};
        if editor.cursor_position_in_buffer.col > 0 {
            backward_char.execute(editor)?;
        }
        'outer: loop {
            if editor.cursor_position_in_buffer.col > 0 {
                while editor.cursor_position_in_buffer.col > 0 {
                    let c = editor.current_char().unwrap();
                    if !c.is_whitespace() {
                        break;
                    }
                    backward_char.execute(editor)?;
                }
                while editor.cursor_position_in_buffer.col > 0 {
                    let c = editor.current_char().unwrap();
                    if c.is_whitespace() {
                        break 'outer;
                    }
                    backward_char.execute(editor)?;
                }
                if editor.cursor_position_in_buffer.col == 0 {
                    break 'outer;
                }
            } else if editor.cursor_position_in_buffer.row > 0 {
                let mut previous_line = PreviousLine {};
                previous_line.execute(editor)?;
                let mut move_end_of_line = MoveEndOfLine {};
                move_end_of_line.execute(editor)?;
                while editor.cursor_position_in_buffer.col > 0 {
                    let c = editor.current_char().unwrap();
                    if c.is_whitespace() {
                        break 'outer;
                    }
                    backward_char.execute(editor)?;
                }
            } else {
                break;
            }
        }
        if editor.cursor_position_in_buffer.col != 0 {
            let mut forward_char = ForwardChar {};
            forward_char.execute(editor)?;
        }
        Ok(())
    }
}