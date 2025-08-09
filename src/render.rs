use std::io::Write;

use crossterm::{
    cursor,
    style::{self},
    terminal, QueueableCommand,
};
use log::info;

use crate::{
    editor::{Editor, TerminalSize},
    generic_error::GenericResult,
    util::get_char_width,
};

pub fn render(editor: &mut Editor, stdout: &mut std::io::Stdout) -> GenericResult<()> {
    info!("render");
    let mut stdout = stdout.lock();
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    let mut cursor_position_on_writing = TerminalSize {
        width: 0,
        height: 0,
    };
    let start_row: usize = editor.window_position_in_buffer.row;
    let lines = &editor.buffer.lines;
    for line in &lines[start_row..] {
        for c in line.chars() {
            // check if c is double width character
            let char_width = get_char_width(c);
            stdout.queue(style::Print(c))?;
            cursor_position_on_writing.width += char_width as u16;
            if cursor_position_on_writing.width >= editor.terminal_size.width {
                cursor_position_on_writing.width = 0;
                cursor_position_on_writing.height += 1;
                stdout.queue(cursor::MoveTo(0, cursor_position_on_writing.height))?;
            }
            if cursor_position_on_writing.height >= editor.content_height() {
                break;
            }
        }
        cursor_position_on_writing.width = 0;
        cursor_position_on_writing.height += 1;
        stdout.queue(cursor::MoveTo(0, cursor_position_on_writing.height))?;
    }

    // render status line
    cursor_position_on_writing.width = 0;
    cursor_position_on_writing.height = editor.content_height();
    stdout.queue(cursor::MoveTo(0, cursor_position_on_writing.height))?;
    for c in editor.status_line.chars() {
        stdout.queue(style::Print(c))?;
        let char_width = get_char_width(c);
        cursor_position_on_writing.width += char_width;
    }
    for _ in cursor_position_on_writing.width..editor.terminal_size.width {
        stdout.queue(style::Print(" "))?;
    }

    // カーソル位置をコンテンツ領域内に制限（ステータス行を超えないように）
    let cursor_row = std::cmp::min(
        editor.cursor_position_on_screen.row as u16,
        editor.content_height() - 1,
    );
    
    stdout.queue(cursor::MoveTo(
        editor.cursor_position_on_screen.col as u16,
        cursor_row,
    ))?;
    stdout.flush()?;

    Ok(())
}
