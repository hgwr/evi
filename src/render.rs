use std::io::Write;

use crossterm::{cursor, style, terminal, QueueableCommand};

use crate::{
    editor::{Editor, TerminalSize},
    generic_error::GenericResult,
    util::get_char_width,
};

pub fn render(editor: &mut Editor, stdout: &mut std::io::Stdout) -> GenericResult<()> {
    let mut stdout = stdout.lock();
    stdout.queue(terminal::Clear(terminal::ClearType::All))?;
    stdout.queue(cursor::MoveTo(0, 0))?;

    let mut cursor_position_on_writing = TerminalSize {
        width: 0,
        height: 0,
    };
    let mut start_row: usize = editor.window_position_in_buffer.row;
    let lines = &editor.buffer.lines;

    // Adjust start_row so that the screen is filled from the bottom when
    // nearing the end of the buffer. This prevents extra blank lines from
    // appearing below the last line when it wraps.
    let mut total_rows = 0usize;
    for line in &lines[start_row..] {
        let len = line.chars().count();
        total_rows += if len == 0 {
            1
        } else {
            (len - 1) / editor.terminal_size.width as usize + 1
        };
        if total_rows >= editor.content_height() as usize {
            break;
        }
    }
    if total_rows < editor.content_height() as usize {
        while start_row > 0 {
            let len = lines[start_row - 1].chars().count();
            let row_height = if len == 0 {
                1
            } else {
                (len - 1) / editor.terminal_size.width as usize + 1
            };
            if total_rows + row_height > editor.content_height() as usize {
                break;
            }
            start_row -= 1;
            total_rows += row_height;
        }
    }
    let row_offset = editor.window_position_in_buffer.row - start_row;
    let highlight_re = editor
        .last_search_pattern
        .as_ref()
        .and_then(|p| regex::Regex::new(p).ok());
    for (line_idx, line) in lines[start_row..].iter().enumerate() {
        let mut highlight: Vec<bool> = vec![false; line.chars().count()];
        if let Some(re) = &highlight_re {
            for mat in re.find_iter(line) {
                let start = line[..mat.start()].chars().count();
                let end = start + line[mat.start()..mat.end()].chars().count();
                for i in start..end {
                    if i < highlight.len() {
                        highlight[i] = true;
                    }
                }
            }
        }
        let line_len = line.chars().count();
        let mut wrapped_at_line_end = false;
        for (idx, c) in line.chars().enumerate() {
            if highlight.get(idx) == Some(&true) {
                stdout.queue(style::SetAttribute(style::Attribute::Reverse))?;
            }
            let char_width = get_char_width(c);
            stdout.queue(style::Print(c))?;
            if highlight.get(idx) == Some(&true) {
                stdout.queue(style::SetAttribute(style::Attribute::Reset))?;
            }
            cursor_position_on_writing.width += char_width as u16;
            if cursor_position_on_writing.width >= editor.terminal_size.width {
                cursor_position_on_writing.width = 0;
                cursor_position_on_writing.height += 1;
                stdout.queue(cursor::MoveTo(0, cursor_position_on_writing.height))?;
                if idx == line_len - 1 {
                    wrapped_at_line_end = true;
                }
            }
            if cursor_position_on_writing.height >= editor.content_height() {
                break;
            }
        }
        if (cursor_position_on_writing.width != 0 || line.is_empty() || !wrapped_at_line_end)
            && cursor_position_on_writing.height < editor.content_height() - 1
            && (line_idx + start_row + 1) < lines.len()
        {
            cursor_position_on_writing.width = 0;
            cursor_position_on_writing.height += 1;
            stdout.queue(cursor::MoveTo(0, cursor_position_on_writing.height))?;
        }
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

    let (cursor_col, cursor_row) = if editor.is_ex_command_mode() {
        (editor.get_ex_command_cursor_col(), editor.content_height())
    } else {
        (
            editor.cursor_position_on_screen.col as u16,
            (editor.cursor_position_on_screen.row as usize + row_offset) as u16,
        )
    };
    stdout.queue(cursor::MoveTo(cursor_col, cursor_row))?;
    stdout.flush()?;

    Ok(())
}
