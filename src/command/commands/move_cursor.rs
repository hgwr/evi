use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;
use crate::util::{get_char_width, get_line_height};

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
                if editor.cursor_position_on_screen.row < editor.content_height() - 1 {
                    editor.cursor_position_on_screen.row += 1;
                } else {
                    editor.window_position_in_buffer.row += 1;
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct NextLine;
impl Command for NextLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let next_row = editor.cursor_position_in_buffer.row + 1;
        if next_row < editor.buffer.lines.len() {
            // 通常の場合：次の行に移動
            let current_cursor_col_in_buffer = editor.cursor_position_in_buffer.col;

            // 現在行の残りを一気に飛ばす。折り返し行数を計算し、
            // スクロールはここでのみ行う。
            let current_line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];

            // 現在のカーソル位置が行の何行目に相当するかを計算
            let mut width = 0usize;
            let mut cursor_row_in_line = 0usize;
            for (i, c) in current_line.chars().enumerate() {
                if i >= editor.cursor_position_in_buffer.col {
                    break;
                }
                width += get_char_width(c) as usize;
                if width >= editor.terminal_size.width as usize {
                    width = 0;
                    cursor_row_in_line += 1;
                }
            }

            // 行全体が何行に折り返されるかを計算
            width = 0;
            let mut line_height = 1usize;
            for c in current_line.chars() {
                width += get_char_width(c) as usize;
                if width >= editor.terminal_size.width as usize {
                    width = 0;
                    line_height += 1;
                }
            }

            let remaining_lines = line_height - cursor_row_in_line;
            let mut new_screen_row = editor.cursor_position_on_screen.row + remaining_lines as u16;
            if new_screen_row < editor.content_height() {
                editor.cursor_position_on_screen.row = new_screen_row;
            } else {
                let overflow = new_screen_row - (editor.content_height() - 1);
                let mut removed_screen_lines = 0usize;
                for i in 0..overflow as usize {
                    let line = &editor.buffer.lines[editor.window_position_in_buffer.row + i];
                    removed_screen_lines += get_line_height(line, editor.terminal_size.width);
                }
                editor.window_position_in_buffer.row += overflow as usize;
                let new_row = editor.cursor_position_on_screen.row as isize
                    + remaining_lines as isize
                    - removed_screen_lines as isize;
                editor.cursor_position_on_screen.row = if new_row < 0 { 0 } else { new_row as u16 };
            }

            editor.cursor_position_in_buffer.row = next_row;

            // 目的の列に移動
            let next_line = &editor.buffer.lines[editor.cursor_position_in_buffer.row];
            let num_of_chars_of_next_line = next_line.chars().count();
            let destination_col = if current_cursor_col_in_buffer > num_of_chars_of_next_line {
                num_of_chars_of_next_line
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
        } else {
            // ファイルの最後の行にいる場合：カーソルは移動せずにスクロールのみ試行
            // 画面の最下行（コンテンツ領域の最後）にカーソルがあり、かつ
            // まだスクロール可能な場合（バッファの表示開始位置を下げられる場合）
            if editor.cursor_position_on_screen.row >= editor.content_height() - 1 {
                // スクロール可能かチェック：現在のウィンドウ位置 + コンテンツ高さが
                // バッファの総行数より小さい場合
                let max_window_start =
                    if editor.buffer.lines.len() > editor.content_height() as usize {
                        editor.buffer.lines.len() - editor.content_height() as usize
                    } else {
                        0
                    };

                if editor.window_position_in_buffer.row < max_window_start {
                    editor.window_position_in_buffer.row += 1;
                    // カーソルの画面上の位置を調整（最下行に保持）
                    if editor.cursor_position_on_screen.row > 0 {
                        editor.cursor_position_on_screen.row -= 1;
                    }
                }
            }
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

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
                    if c.is_whitespace() {
                        break;
                    }
                    forward_char.execute(editor)?;
                }
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.get_current_char().unwrap();
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
                num_of_chars = editor.get_num_of_current_line_chars();
                while editor.cursor_position_in_buffer.col + 1 < num_of_chars {
                    let c = editor.get_current_char().unwrap();
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

    fn as_any(&self) -> &dyn Any {
        self
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
