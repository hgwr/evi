use std::io::Write;
use std::path::PathBuf;

use crossterm::{
    terminal::{self, ClearType},
    ExecutableCommand,
};

use log::info;

use crate::command::base::CommandData;
use crate::command::factory::command_factory;
use crate::render::render;
use crate::{buffer::Buffer, command::base::ExecutedCommand, generic_error::GenericResult};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CursorPositionOnScreen {
    pub row: u16,
    pub col: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CursorPositionInBuffer {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EditorCursorData {
    pub cursor_position_on_screen: CursorPositionOnScreen,
    pub cursor_position_in_buffer: CursorPositionInBuffer,
    pub window_position_in_buffer: CursorPositionInBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Command,
    Insert,
}

pub struct Editor {
    pub buffer: Buffer,
    editing_file_paths: Vec<PathBuf>,
    current_file_index: usize,
    pub is_dirty: bool,
    mode: Mode,
    pub should_exit: bool,
    pub terminal_size: TerminalSize,
    pub cursor_position_on_screen: CursorPositionOnScreen,
    pub cursor_position_in_buffer: CursorPositionInBuffer,
    pub window_position_in_buffer: CursorPositionInBuffer,
    pub status_line: String,
    pub command_history: Vec<ExecutedCommand>,
    pub last_input_string: String,
}

impl Editor {
    pub fn new() -> Editor {
        Editor {
            buffer: Buffer::new(),
            editing_file_paths: Vec::new(),
            current_file_index: 0,
            is_dirty: false,
            mode: Mode::Command,
            should_exit: false,
            terminal_size: TerminalSize {
                width: 0,
                height: 0,
            },
            cursor_position_on_screen: CursorPositionOnScreen { row: 0, col: 0 },
            cursor_position_in_buffer: CursorPositionInBuffer { row: 0, col: 0 },
            window_position_in_buffer: CursorPositionInBuffer { row: 0, col: 0 },
            status_line: "".to_string(),
            command_history: Vec::new(),
            last_input_string: "".to_string(),
        }
    }

    pub fn open_file(&mut self, file_path: &PathBuf) {
        self.buffer = Buffer::from_file(file_path);
        self.editing_file_paths.push(file_path.clone());
        self.current_file_index = self.editing_file_paths.len() - 1;
    }

    pub fn save_file(&self) -> GenericResult<()> {
        if let Some(file_path) = self.editing_file_paths.get(self.current_file_index) {
            self.buffer.to_file(file_path)
        } else {
            println!("No file to save");
            GenericResult::Err("No file to save".to_string().into())
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

    pub fn current_file_name(&self) -> Option<String> {
        self.editing_file_paths
            .get(self.current_file_index)
            .map(|path| path.to_string_lossy().to_string())
    }

    pub fn resize_terminal(&mut self, width: u16, height: u16) {
        info!("Resize terminal to width: {}, height: {}", width, height);
        self.terminal_size = TerminalSize { width, height };
        if self.cursor_position_on_screen.col >= width {
            self.cursor_position_on_screen.col = width - 1;
        }
    }

    pub fn set_command_mode(&mut self) {
        match self.mode {
            Mode::Command => {}
            Mode::Insert => {
                self.mode = Mode::Command;
                if let Some(mut last_command) = self.command_history.pop() {
                    last_command
                        .command
                        .set_text(self.last_input_string.clone());
                    self.command_history.push(last_command);
                    info!("input string: {}", self.last_input_string);
                }
                self.status_line = "".to_string();
            }
        }
    }

    pub fn set_insert_mode(&mut self) {
        match self.mode {
            Mode::Command => {
                self.mode = Mode::Insert;
                self.status_line = "-- INSERT --".to_string();
                self.last_input_string = "".to_string();
            }
            Mode::Insert => {}
        }
    }

    pub fn is_command_mode(&self) -> bool {
        self.mode == Mode::Command
    }

    pub fn is_insert_mode(&self) -> bool {
        self.mode == Mode::Insert
    }

    pub fn snapshot_cursor_data(&self) -> EditorCursorData {
        EditorCursorData {
            cursor_position_on_screen: self.cursor_position_on_screen,
            cursor_position_in_buffer: self.cursor_position_in_buffer,
            window_position_in_buffer: self.window_position_in_buffer,
        }
    }

    pub fn restore_cursor_data(&mut self, cursor_data: EditorCursorData) {
        self.cursor_position_on_screen = cursor_data.cursor_position_on_screen;
        self.cursor_position_in_buffer = cursor_data.cursor_position_in_buffer;
        self.window_position_in_buffer = cursor_data.window_position_in_buffer;
    }

    pub fn execute_command(&mut self, command_data: CommandData) -> GenericResult<()> {
        let mut command = command_factory(&command_data);
        let result = command.execute(self);
        if command.is_undoable() {
            self.command_history.push(ExecutedCommand {
                command_data,
                command,
            });
        }
        result
    }

    pub fn undo(&mut self) -> GenericResult<()> {
        if let Some(mut executed_command) = self.command_history.pop() {
            executed_command.command.undo(self)
        } else {
            Ok(())
        }
    }

    pub fn render(self: &mut Editor, stdout: &mut std::io::Stdout) -> GenericResult<()> {
        render(self, stdout)
    }

    pub fn content_height(&self) -> u16 {
        self.terminal_size.height - 1
    }

    pub fn display_visual_bell(&mut self) -> GenericResult<()> {
        let mut stdout = std::io::stdout();
        stdout.write_all(b"\x07")?;
        stdout.flush()?;
        Ok(())
    }

    pub fn insert_char(&mut self, key_event: crossterm::event::KeyEvent) -> GenericResult<()> {
        if let crossterm::event::KeyCode::Char(c) = key_event.code {
            self.buffer.insert_char(
                self.cursor_position_in_buffer.row,
                self.cursor_position_in_buffer.col,
                c,
            )?;
            self.last_input_string.push(c);
            let char_width = crate::util::get_char_width(c);
            self.cursor_position_in_buffer.col += 1;
            self.cursor_position_on_screen.col += char_width;
            if self.cursor_position_on_screen.col >= self.terminal_size.width {
                self.cursor_position_on_screen.col = 0;
                if self.cursor_position_on_screen.row < self.content_height() {
                    self.cursor_position_on_screen.row += 1;
                } else {
                    self.window_position_in_buffer.row += 1;
                }
            }
        }
        Ok(())
    }

    pub fn backward_delete_char(&mut self) -> GenericResult<()> {
        if self.cursor_position_in_buffer.col > 0 && self.last_input_string.len() > 0 {
            self.buffer.delete_char(
                self.cursor_position_in_buffer.row,
                self.cursor_position_in_buffer.col - 1,
            )?;
            self.last_input_string.pop();
            self.cursor_position_in_buffer.col -= 1;
            let char = self.buffer.get_char(
                self.cursor_position_in_buffer.row,
                self.cursor_position_in_buffer.col,
            );
            if let Some(char) = char {
                let char_width = crate::util::get_char_width(char);
                self.cursor_position_on_screen.col -= char_width;
                if self.cursor_position_on_screen.col >= self.terminal_size.width {
                    self.cursor_position_on_screen.col = self.terminal_size.width - 1;
                    if self.cursor_position_on_screen.row > 0 {
                        self.cursor_position_on_screen.row -= 1;
                    } else if self.window_position_in_buffer.row > 0 {
                        self.window_position_in_buffer.row -= 1;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn append_new_line(&mut self) -> GenericResult<()> {
        let rest_of_line = self.buffer.lines[self.cursor_position_in_buffer.row]
            .chars()
            .skip(self.cursor_position_in_buffer.col)
            .collect::<String>();
        let new_line = self.buffer.lines[self.cursor_position_in_buffer.row]
            .chars()
            .take(self.cursor_position_in_buffer.col)
            .collect::<String>();
        self.buffer.lines[self.cursor_position_in_buffer.row] = new_line;
        self.buffer.lines.insert(self.cursor_position_in_buffer.row + 1, rest_of_line);
        self.cursor_position_in_buffer.row += 1;
        self.cursor_position_in_buffer.col = 0;
        if self.cursor_position_on_screen.row < self.content_height() {
            self.cursor_position_on_screen.row += 1;
        } else {
            self.window_position_in_buffer.row += 1;
        }
        self.cursor_position_on_screen.col = 0;
        self.last_input_string.push('\n');
        Ok(())
    }

}

impl Drop for Editor {
    fn drop(&mut self) {
        info!("Drop Editor");
        let mut stdout = std::io::stdout();
        terminal::disable_raw_mode().unwrap();
        stdout.execute(terminal::Clear(ClearType::All)).unwrap();
        stdout.execute(terminal::LeaveAlternateScreen).unwrap();
        stdout.flush().unwrap();
    }
}
