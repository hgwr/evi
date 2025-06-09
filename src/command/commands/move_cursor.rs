use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;
use crate::util::get_char_width;

#[derive(Clone)]
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
                if editor.cursor_position_on_screen.row < editor.max_content_row_index() {
                    editor.cursor_position_on_screen.row += 1;
                } else {
                    editor.window_position_in_buffer.row += 1;
                }
            }
        } else {
            editor.display_visual_bell()?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct MoveBeginningOfLine;
impl Command for MoveBeginningOfLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let mut backward_char = BackwardChar {};
        while editor.cursor_position_in_buffer.col > 0 {
            backward_char.execute(editor)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct MoveFirstNonBlank;
impl Command for MoveFirstNonBlank {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let row = editor.cursor_position_in_buffer.row;

        let mut move_bol = MoveBeginningOfLine {};
        move_bol.execute(editor)?;

        let mut forward_char = ForwardChar {};
        let num_chars = editor.buffer.lines[row].chars().count();
        while editor.cursor_position_in_buffer.col < num_chars {
            let c = editor.buffer.lines[row]
                .chars()
                .nth(editor.cursor_position_in_buffer.col)
                .unwrap();
            if !c.is_whitespace() {
                break;
            }
            forward_char.execute(editor)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct NextLine;
impl Command for NextLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let width = editor.terminal_size.width as usize;
        let current_row = editor.cursor_position_in_buffer.row;
        let current_col = editor.cursor_position_in_buffer.col;
        let line_height = editor.line_height(current_row);

        if current_col / width + 1 < line_height {
            let dest_col = (current_col + width)
                .min(editor.get_num_of_current_line_chars().saturating_sub(1));
            if dest_col > current_col {
                let mut f = ForwardChar {};
                for _ in 0..(dest_col - current_col) {
                    f.execute(editor)?;
                }
            }
        } else {
            let next_row = current_row + 1;
            if next_row < editor.buffer.lines.len() {
                let current_cursor_col_in_buffer = current_col;
                let mut move_end_of_line = MoveEndOfLine {};
                move_end_of_line.execute(editor)?;

                editor.cursor_position_in_buffer.row = next_row;
                if editor.cursor_position_on_screen.row < editor.max_content_row_index() {
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
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct PreviousLine;
impl Command for PreviousLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let width = editor.terminal_size.width as usize;
        let current_row = editor.cursor_position_in_buffer.row;
        let current_col = editor.cursor_position_in_buffer.col;

        if current_col / width > 0 {
            let dest_col = current_col.saturating_sub(width);
            let mut b = BackwardChar {};
            for _ in 0..(current_col - dest_col) {
                b.execute(editor)?;
            }
        } else if current_row > 0 {
            let current_cursor_col_in_buffer = current_col;
            if current_cursor_col_in_buffer > 0 {
                let mut move_beginning_of_line = MoveBeginningOfLine {};
                move_beginning_of_line.execute(editor)?;
            }

            editor.cursor_position_in_buffer.row -= 1;

            let num_of_lines_on_screen = editor.line_height(editor.cursor_position_in_buffer.row);

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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct ForwardWord;
impl Command for ForwardWord {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let mut forward_char = ForwardChar {};
        let mut num_of_chars = editor.get_num_of_current_line_chars();

        forward_char.execute(editor)?;

        'outer: loop {
            if editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.get_current_char().unwrap();
                    if !c.is_alphanumeric() && c != '_' {
                        break;
                    }
                    forward_char.execute(editor)?;
                }
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.get_current_char().unwrap();
                    if c.is_alphanumeric() || c == '_' {
                        break 'outer;
                    }
                    forward_char.execute(editor)?;
                }
            } else if editor.cursor_position_in_buffer.row + 1 < editor.buffer.lines.len() {
                let mut next_line = NextLine {};
                next_line.execute(editor)?;
                let mut move_beginning_of_line = MoveBeginningOfLine {};
                move_beginning_of_line.execute(editor)?;
                num_of_chars = editor.get_num_of_current_line_chars();
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.get_current_char().unwrap();
                    if c.is_alphanumeric() || c == '_' {
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
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
                    let c = editor.get_current_char().unwrap();
                    if !c.is_whitespace() {
                        break;
                    }
                    backward_char.execute(editor)?;
                }
                while editor.cursor_position_in_buffer.col > 0 {
                    let c = editor.get_current_char().unwrap();
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
                    let c = editor.get_current_char().unwrap();
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct PageDown;
impl Command for PageDown {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.page_down()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct PageUp;
impl Command for PageUp {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.page_up()
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
    fn next_previous_line_wrapping() {
        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["a".repeat(60), "b".repeat(120), "c".repeat(60)];

        let mut next_line = NextLine {};
        next_line.execute(&mut editor).unwrap();
        assert_eq!(editor.cursor_position_in_buffer.row, 1);
        assert_eq!(editor.cursor_position_on_screen.row, 1);
        assert_eq!(editor.window_position_in_buffer.row, 0);

        next_line.execute(&mut editor).unwrap();
        assert_eq!(editor.cursor_position_in_buffer.row, 1);
        assert_eq!(editor.cursor_position_on_screen.row, 2);
        assert_eq!(editor.window_position_in_buffer.row, 0);

        let mut previous_line = PreviousLine {};
        previous_line.execute(&mut editor).unwrap();
        assert_eq!(editor.cursor_position_in_buffer.row, 1);
        assert_eq!(editor.cursor_position_on_screen.row, 1);
        assert_eq!(editor.window_position_in_buffer.row, 0);
    }

    #[test]
    fn move_first_non_blank_basic() {
        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["  abc".to_string()];

        let mut forward_char = ForwardChar {};
        for _ in 0..4 {
            forward_char.execute(&mut editor).unwrap();
        }

        assert_eq!(editor.cursor_position_in_buffer.col, 4);

        let mut caret = MoveFirstNonBlank {};
        caret.execute(&mut editor).unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 2);
        assert_eq!(editor.cursor_position_on_screen.col, 2);
    }
}
