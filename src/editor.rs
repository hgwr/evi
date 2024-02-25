use std::path::PathBuf;

use crate::buffer::Buffer;
use crate::command::base::CommandData;
use crate::command::factory::command_factory;
use crate::render::render;

pub struct TerminalSize {
  pub width: u16,
  pub height: u16,
}

pub struct CursorPosition {
  pub row: usize,
  pub col: usize,
}

pub struct Editor {
  pub buffer: Buffer,
  editing_file_paths: Vec<PathBuf>,
  current_file_index: usize,
  pub is_dirty: bool,
  pub terminal_size: TerminalSize,
  pub cursor_position_on_screen: CursorPosition,
  pub cursor_position_in_buffer: CursorPosition,
}

impl Editor {
  pub fn new() -> Editor {
    Editor {
      buffer: Buffer::new(),
      editing_file_paths: Vec::new(),
      current_file_index: 0,
      is_dirty: false,
      terminal_size: TerminalSize { width: 0, height: 0 },
      cursor_position_on_screen: CursorPosition { row: 0, col: 0 },
      cursor_position_in_buffer: CursorPosition { row: 0, col: 0 },
    }
  }

  pub fn open_file(&mut self, file_path: &PathBuf) {
    self.buffer = Buffer::from_file(file_path);
    self.editing_file_paths.push(file_path.clone());
    self.current_file_index = self.editing_file_paths.len() - 1;
  }

  pub fn save_file(&self) {
    if let Some(file_path) = self.editing_file_paths.get(self.current_file_index) {
      self.buffer.to_file(file_path);
    } else {
      println!("No file to save");
    }
  }

  pub fn from_cmd_args(args: Vec<String>) -> Editor {
    let mut editor = Editor::new();
    // args で与えられた複数のファイル名のうち、最初のファイルを開き、残りを editing_file_paths に追加する
    if args.len() > 1 {
      editor.open_file(&PathBuf::from(&args[1]));
      for file_name in &args[2..] {
        editor.editing_file_paths.push(PathBuf::from(file_name));
      }
    }
    editor
  }

  pub fn current_file_name(&self) -> Option<String>{
    self.editing_file_paths.get(self.current_file_index)
      .map(|path| path.to_string_lossy().to_string())
  }

  pub fn resize_terminal(&mut self, width: u16, height: u16) {
    self.terminal_size = TerminalSize { width, height };
    let width: usize = self.terminal_size.width.into();
    if self.cursor_position_on_screen.col >= width {
      self.cursor_position_on_screen.col = width - 1;
    }
  }

  pub fn execute_command(&mut self, command_data: CommandData) {
    let mut command = command_factory(&command_data);
    command.execute(self);
  }

  pub fn render(self: &mut Editor, stdout: &mut std::io::Stdout) {
    render(self, stdout);
  }
}