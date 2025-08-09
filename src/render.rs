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
    let start_row: usize = editor.window_top_row;
    let lines = &editor.buffer.lines;
    'outer: for line in &lines[start_row..] {
        // 画面内容領域（ステータスライン除く）を超える場合は描画終了
        if cursor_position_on_writing.height >= editor.content_height() { break; }
        for c in line.chars() {
            let char_width = get_char_width(c);
            stdout.queue(style::Print(c))?;
            cursor_position_on_writing.width += char_width as u16;
            if cursor_position_on_writing.width >= editor.terminal_size.width {
                // 折り返し
                cursor_position_on_writing.width = 0;
                cursor_position_on_writing.height += 1;
                if cursor_position_on_writing.height >= editor.content_height() { break; }
                stdout.queue(cursor::MoveTo(0, cursor_position_on_writing.height))?;
            }
        }
        // 次の実行位置を次行先頭へ。直前の行で高さが一杯になった場合は抜ける。
        cursor_position_on_writing.width = 0;
        cursor_position_on_writing.height += 1;
        if cursor_position_on_writing.height >= editor.content_height() { break 'outer; }
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

    // 新しい計算ベースのカーソル位置
    let sp = editor.calculate_screen_position();
    let cursor_row = std::cmp::min(sp.row as u16, editor.content_height() - 1);
    stdout.queue(cursor::MoveTo(sp.col as u16, cursor_row))?;
    stdout.flush()?;

    Ok(())
}
