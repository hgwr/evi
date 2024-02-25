use std::{char, io::{self, Write}};
use crossterm::{
    ExecutableCommand, QueueableCommand,
    terminal, cursor, style::{self, Stylize}
};
use unicode_width::UnicodeWidthChar;

use crate::editor::{Editor, TerminalSize};

pub fn render(editor: &mut Editor, stdout: &mut std::io::Stdout) {
  let mut stdout = stdout.lock();
  stdout.queue(terminal::Clear(terminal::ClearType::All)).unwrap();
  stdout.queue(cursor::MoveTo(0, 0)).unwrap();

  let mut cursor_position_on_writing = TerminalSize {
    width: 0,
    height: 0,
  };
  let start_row = editor.cursor_position_on_screen.row;
  let lines = &editor.buffer.lines;
  for line in &lines[start_row..] {
    for c in line.chars() {
      // check if c is double width character
      let char_width = UnicodeWidthChar::width(c).unwrap_or(0);
      stdout.queue(style::Print(c)).unwrap();
      cursor_position_on_writing.width += char_width as u16;
      if cursor_position_on_writing.width >= editor.terminal_size.width {
        cursor_position_on_writing.width = 0;
        cursor_position_on_writing.height += 1;
        stdout.queue(cursor::MoveTo(0, cursor_position_on_writing.height)).unwrap();
      }
      if cursor_position_on_writing.height >= editor.terminal_size.height {
        break;
      }
    }
  }

  stdout.queue(cursor::MoveTo(
    editor.cursor_position_on_screen.col as u16,
    editor.cursor_position_on_screen.row as u16,
  )).unwrap();
  stdout.flush().unwrap();
}
