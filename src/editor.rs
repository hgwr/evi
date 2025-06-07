use std::io::Write;
use std::path::PathBuf;

use crossterm::{
    terminal::{self, ClearType},
    ExecutableCommand,
};

use log::info;

use crate::render::render;
use crate::{
    buffer::Buffer,
    command::base::ExecutedCommand,
    generic_error::{GenericError, GenericResult},
};
use crate::{
    buffer::CursorPositionInBuffer,
    command::base::{Command, CommandData},
    ex::parser::Parser,
};
use crate::{
    command::factory::command_factory,
    data::{LineAddressType, SimpleLineAddressType},
};

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
pub struct EditorCursorData {
    pub cursor_position_on_screen: CursorPositionOnScreen,
    pub cursor_position_in_buffer: CursorPositionInBuffer,
    pub window_position_in_buffer: CursorPositionInBuffer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Region {
    pub start: EditorCursorData,
    pub end: EditorCursorData,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Command,
    Insert,
    Replace,
    ReplaceChar,
    ExCommand,
    Search(SearchDirection),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SearchDirection {
    Forward,
    Backward,
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
    pub command_history: Vec<Vec<ExecutedCommand>>,
    pub unnamed_register: String,
    pub unnamed_register_linewise: bool,
    pub last_input_string: String,
    pub ex_command_data: String,
    pub search_query: String,
    pub last_search_pattern: Option<String>,
    pub last_search_direction: Option<SearchDirection>,
    pub pending_replace_char_count: usize,
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
            unnamed_register: String::new(),
            unnamed_register_linewise: false,
            last_input_string: "".to_string(),
            ex_command_data: "".to_string(),
            search_query: String::new(),
            last_search_pattern: None,
            last_search_direction: None,
            pending_replace_char_count: 1,
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
            Mode::ExCommand => {
                self.mode = Mode::Command;
                self.status_line = "".to_string();
            }
            Mode::Insert => {
                self.mode = Mode::Command;
                self.convert_repetitive_command_history_to_commands_history();
                self.status_line = "".to_string();
            }
            Mode::Replace => {
                self.mode = Mode::Command;
                self.convert_repetitive_command_history_to_commands_history();
                self.status_line = "".to_string();
            }
            Mode::ReplaceChar => {
                self.mode = Mode::Command;
                self.status_line = "".to_string();
                self.pending_replace_char_count = 1;
            }
            Mode::Search(_) => {
                self.mode = Mode::Command;
                self.status_line = "".to_string();
                self.search_query.clear();
            }
        }
    }

    fn convert_repetitive_command_history_to_commands_history(&mut self) {
        if let Some(mut last_command_chunk) = self.command_history.pop() {
            let mut last_executed_command = last_command_chunk.pop().unwrap();
            last_executed_command
                .command
                .set_text(self.last_input_string.clone());

            let count = last_executed_command.command_data.count;
            if count == 1 {
                self.command_history.push(vec![last_executed_command]);
            } else if count >= 2 {
                last_executed_command.command_data.count = 1;
                let command_data: CommandData = last_executed_command.command_data.clone();
                self.do_repetitive_command(
                    count,
                    command_data,
                    Some(last_executed_command.command),
                );
            } else {
                panic!("count: {}", count);
            }
            info!("input string: {}", self.last_input_string);
        }
    }

    fn do_repetitive_command(
        &mut self,
        count: usize,
        command_data: CommandData,
        mut command_opt: Option<Box<dyn Command>>,
    ) {
        let mut command_series: Vec<ExecutedCommand> = Vec::new();
        for _ in 1..count {
            if let Some(mut command) = command_opt {
                let redo_result = command.redo(self);
                command_series.push(ExecutedCommand {
                    command_data: command_data.clone(),
                    command,
                });
                info!("command_series.len(): {}", command_series.len());
                if let Ok(Some(next_command)) = redo_result {
                    command_opt = Some(next_command);
                } else {
                    command_opt = None;
                    break;
                }
            }
        }
        if let Some(command) = command_opt {
            command_series.push(ExecutedCommand {
                command_data: command_data.clone(),
                command,
            });
            info!("command_series.len(): {}", command_series.len());
        }
        info!("### command_series.len(): {}", command_series.len());
        self.command_history.push(command_series);
    }

    pub fn set_insert_mode(&mut self) {
        match self.mode {
            Mode::ExCommand => {
                self.mode = Mode::Insert;
                self.status_line = "".to_string();
            }
            Mode::Command => {
                self.mode = Mode::Insert;
                self.status_line = "-- INSERT --".to_string();
                self.last_input_string = "".to_string();
            }
            Mode::Insert => {}
            Mode::Replace | Mode::ReplaceChar => {
                self.mode = Mode::Insert;
                self.status_line = "-- INSERT --".to_string();
                self.last_input_string = String::new();
            }
            Mode::Search(_) => {
                self.mode = Mode::Insert;
                self.status_line = "-- INSERT --".to_string();
                self.last_input_string = String::new();
                self.search_query.clear();
            }
        }
    }

    pub fn set_replace_mode(&mut self) {
        match self.mode {
            Mode::Command | Mode::ReplaceChar => {
                self.mode = Mode::Replace;
                self.status_line = "-- REPLACE --".to_string();
                self.last_input_string = String::new();
            }
            Mode::Insert => {
                self.mode = Mode::Replace;
                self.status_line = "-- REPLACE --".to_string();
            }
            Mode::Replace => {}
            Mode::ExCommand => {
                self.mode = Mode::Replace;
                self.status_line = "-- REPLACE --".to_string();
            }
        self.set_replace_char_mode_with_count(1);
    }

    pub fn set_replace_char_mode_with_count(&mut self, count: usize) {
        self.pending_replace_char_count = count;
            Mode::Search(_) => {
                self.mode = Mode::Replace;
                self.status_line = "-- REPLACE --".to_string();
                self.last_input_string = String::new();
                self.search_query.clear();
            }
        }
    }

    pub fn set_replace_char_mode(&mut self) {
        self.mode = Mode::ReplaceChar;
        self.last_input_string = String::new();
    }

    pub fn is_replace_mode(&self) -> bool {
        matches!(self.mode, Mode::Replace)
    }

    pub fn is_replace_char_mode(&self) -> bool {
        matches!(self.mode, Mode::ReplaceChar)
    }

    pub fn is_command_mode(&self) -> bool {
        self.mode == Mode::Command
    }

    pub fn is_insert_mode(&self) -> bool {
        self.mode == Mode::Insert
    }

    pub fn is_ex_command_mode(&self) -> bool {
        self.mode == Mode::ExCommand
    }

    pub fn is_search_mode(&self) -> bool {
        matches!(self.mode, Mode::Search(_))
    }

    pub fn set_ex_command_mode(&mut self) {
        self.mode = Mode::ExCommand;
        self.status_line = ":".to_string();
    }

    pub fn set_search_mode(&mut self, direction: SearchDirection) {
        self.mode = Mode::Search(direction);
        self.search_query = String::new();
        self.status_line = match direction {
            SearchDirection::Forward => "/".to_string(),
            SearchDirection::Backward => "?".to_string(),
        };
    }

    pub fn get_ex_command_data(&self) -> String {
        self.ex_command_data.clone()
    }

    pub fn execute_ex_command(&mut self, ex_command_str: String) -> GenericResult<()> {
        let ex_command_str = ex_command_str.trim();
        let mut parser = Parser::new(ex_command_str);
        let result = parser.parse();
        if let Err(e) = result {
            info!("Error: {}", e.to_string());
            self.status_line = e.to_string();
            self.ex_command_data = "".to_string();
            return Ok(());
        }
        let mut command = result.unwrap();
        command.execute(self)?;
        let command_data = CommandData {
            count: 1,
            key_code: crossterm::event::KeyCode::Char(':'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: None,
        };
        self.command_history.push(vec![ExecutedCommand {
            command_data,
            command,
        }]);
        self.ex_command_data = "".to_string();
        Ok(())
    }

    pub fn append_ex_command(&mut self, key_data: crate::command::compose::KeyData) {
        if let crate::command::compose::KeyData {
            key_code: crossterm::event::KeyCode::Char(c),
            ..
        } = key_data
        {
            self.ex_command_data.push(c);
            self.status_line = ":".to_owned() + &self.ex_command_data.clone();
        }
    }

    pub fn append_search_query(&mut self, key_data: crate::command::compose::KeyData) {
        if let crate::command::compose::KeyData {
            key_code: crossterm::event::KeyCode::Char(c),
            ..
        } = key_data
        {
            self.search_query.push(c);
            let prefix = match self.mode {
                Mode::Search(SearchDirection::Forward) => "/",
                Mode::Search(SearchDirection::Backward) => "?",
                _ => "",
            };
            self.status_line = prefix.to_string() + &self.search_query.clone();
        }
    }

    pub fn execute_search_query(&mut self) -> GenericResult<()> {
        if let Mode::Search(direction) = self.mode {
            let pattern = self.search_query.clone();
            self.set_command_mode();
            self.search(direction, &pattern)?;
        }
        Ok(())
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

    pub fn move_cursor_to(&mut self, row: usize, col: usize) -> GenericResult<()> {
        self.cursor_position_in_buffer.row = 0;
        self.cursor_position_in_buffer.col = 0;
        self.cursor_position_on_screen.row = 0;
        self.cursor_position_on_screen.col = 0;
        self.window_position_in_buffer.row = 0;
        self.window_position_in_buffer.col = 0;
        let mut next_line = crate::command::commands::move_cursor::NextLine {};
        for _ in 0..row {
            next_line.execute(self)?;
        }
        let mut forward_char = crate::command::commands::move_cursor::ForwardChar {};
        for _ in 0..col {
            forward_char.execute(self)?;
        }
        Ok(())
    }

    fn char_to_byte_index(s: &str, idx: usize) -> usize {
        s.char_indices()
            .nth(idx)
            .map(|(i, _)| i)
            .unwrap_or_else(|| s.len())
    }

    fn find_next_match(
        &self,
        re: &regex::Regex,
        direction: SearchDirection,
    ) -> Option<CursorPositionInBuffer> {
        let total_lines = self.buffer.lines.len();
        let start_row = self.cursor_position_in_buffer.row;
        let start_col = self.cursor_position_in_buffer.col;
        match direction {
            SearchDirection::Forward => {
                let mut row = start_row;
                let mut first = true;
                loop {
                    let line = &self.buffer.lines[row];
                    let search_start = if first { start_col + 1 } else { 0 };
                    let start_byte = Self::char_to_byte_index(line, search_start);
                    if let Some(mat) = re.find(&line[start_byte..]) {
                        let col = search_start
                            + line[start_byte..start_byte + mat.start()].chars().count();
                        return Some(CursorPositionInBuffer { row, col });
                    }
                    row = (row + 1) % total_lines;
                    if row == start_row && !first {
                        break;
                    }
                    first = false;
                }
                None
            }
            SearchDirection::Backward => {
                let mut row = start_row;
                let mut first = true;
                loop {
                    let line = &self.buffer.lines[row];
                    let search_end = if first {
                        start_col + 1
                    } else {
                        line.chars().count()
                    };
                    let end_byte = Self::char_to_byte_index(line, search_end);
                    let slice = &line[..end_byte];
                    let mut last = None;
                    for mat in re.find_iter(slice) {
                        last = Some(mat);
                    }
                    if let Some(mat) = last {
                        let col = slice[..mat.start()].chars().count();
                        return Some(CursorPositionInBuffer { row, col });
                    }
                    if row == 0 {
                        row = total_lines - 1;
                    } else {
                        row -= 1;
                    }
                    if row == start_row && !first {
                        break;
                    }
                    first = false;
                }
                None
            }
        }
    }

    pub fn search(&mut self, direction: SearchDirection, pattern: &str) -> GenericResult<()> {
        use regex::Regex;
        let re = Regex::new(pattern).map_err(|e| GenericError::from(e.to_string()))?;
        if let Some(pos) = self.find_next_match(&re, direction) {
            self.move_cursor_to(pos.row, pos.col)?;
            self.last_search_pattern = Some(pattern.to_string());
            self.last_search_direction = Some(direction);
        } else {
            self.display_visual_bell()?;
        }
        Ok(())
    }

    pub fn repeat_search(&mut self, same_direction: bool) -> GenericResult<()> {
        if let Some(pattern) = self.last_search_pattern.clone() {
            if let Some(dir) = self.last_search_direction {
                let dir = if same_direction {
                    dir
                } else {
                    match dir {
                        SearchDirection::Forward => SearchDirection::Backward,
                        SearchDirection::Backward => SearchDirection::Forward,
                    }
                };
                self.search(dir, &pattern)?;
            }
        } else {
            self.display_visual_bell()?;
        }
        Ok(())
    }

    pub fn execute_command(&mut self, command_data: CommandData) -> GenericResult<()> {
        let mut command = command_factory(&command_data);
        if !command.is_modeful() && command.is_reusable() {
            for _ in 0..command_data.count {
                command.execute(self)?;
            }
            if command.is_undoable() {
                self.command_history.push(vec![ExecutedCommand {
                    command_data,
                    command,
                }]);
            }
        } else if !command.is_modeful() && !command.is_reusable() {
            let mut command_chunk: Vec<ExecutedCommand> = Vec::new();
            let disassemble_command_data = CommandData {
                count: 1,
                ..command_data
            };
            for _ in 0..command_data.count {
                let mut command = command_factory(&disassemble_command_data);
                command.execute(self)?;
                if command.is_undoable() {
                    command_chunk.push(ExecutedCommand {
                        command_data: disassemble_command_data.clone(),
                        command,
                    });
                }
            }
            if command_chunk.len() > 0 {
                self.command_history.push(command_chunk);
            }
        } else {
            command.execute(self)?;
            if command.is_undoable() {
                self.command_history.push(vec![ExecutedCommand {
                    command_data,
                    command,
                }]);
            }
        }
        Ok(())
    }

    pub fn undo(&mut self) -> GenericResult<()> {
        if let Some(mut last_command_chunk) = self.command_history.pop() {
            while let Some(mut executed_command) = last_command_chunk.pop() {
                for _ in 0..executed_command.command_data.count {
                    executed_command.command.undo(self)?;
                }
            }
            Ok(())
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

    pub fn get_current_char(&self) -> Option<char> {
        self.buffer.get_char(
            self.cursor_position_in_buffer.row,
            self.cursor_position_in_buffer.col,
        )
    }

    pub fn get_num_of_current_line_chars(&self) -> usize {
        self.buffer
            .lines
            .get(self.cursor_position_in_buffer.row)
            .map(|line| line.chars().count())
            .unwrap_or(0)
    }

    pub fn insert_char(&mut self, c: char) -> GenericResult<()> {
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

        Ok(())
    }

    pub fn replace_char_at_cursor(&mut self, c: char) -> GenericResult<()> {
        let row = self.cursor_position_in_buffer.row;
        let col = self.cursor_position_in_buffer.col;
        if col < self.get_num_of_current_line_chars() {
            self.buffer.delete_char(row, col)?;
        }
        self.buffer.insert_char(row, col, c)?;
        Ok(())
    }

    pub fn replace_char_and_move(&mut self, c: char) -> GenericResult<()> {
        self.replace_char_at_cursor(c)?;
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
        } else if self.cursor_position_in_buffer.col == 0 && self.last_input_string.len() > 0 {
            self.last_input_string.pop();
            if self.cursor_position_in_buffer.row > 0 {
                let rest_of_line = self.buffer.lines[self.cursor_position_in_buffer.row].clone();
                self.buffer.lines.remove(self.cursor_position_in_buffer.row);
                let mut previous_line = crate::command::commands::move_cursor::PreviousLine {};
                previous_line.execute(self)?;
                let mut move_end_of_line = crate::command::commands::move_cursor::MoveEndOfLine {};
                move_end_of_line.execute(self)?;
                self.buffer.lines[self.cursor_position_in_buffer.row] += &rest_of_line;
                let mut forward_char = crate::command::commands::move_cursor::ForwardChar {};
                forward_char.execute(self)?;
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
        self.buffer
            .lines
            .insert(self.cursor_position_in_buffer.row + 1, rest_of_line);
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

    pub fn get_line_number_from(&mut self, line_address: &LineAddressType) -> usize {
        let line_number: isize = match line_address {
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::LineNumber(n)) => {
                let input = (*n as isize);
                if input == 0 {
                    0
                } else {
                    input - 1
                }
            }
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::CurrentLine) => {
                self.cursor_position_in_buffer.row as isize
            }
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::FirstLine) => 0,
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::LastLine) => {
                self.buffer.lines.len().saturating_sub(1) as isize
            }
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::AllLines) => {
                self.buffer.lines.len().saturating_sub(1) as isize
            }
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::Pattern(_)) => {
                // TODO: Implement
                unimplemented!()
            }
            crate::data::LineAddressType::Relative(SimpleLineAddressType::FirstLine, i) => 0 + i,
            crate::data::LineAddressType::Relative(SimpleLineAddressType::LineNumber(n), i) => {
                (*n as isize) + i
            }
            crate::data::LineAddressType::Relative(SimpleLineAddressType::CurrentLine, i) => {
                (self.cursor_position_in_buffer.row as isize) + i
            }
            crate::data::LineAddressType::Relative(SimpleLineAddressType::LastLine, i) => {
                (self.buffer.lines.len().saturating_sub(1) as isize) + i
            }
            crate::data::LineAddressType::Relative(SimpleLineAddressType::AllLines, i) => {
                (self.buffer.lines.len().saturating_sub(1) as isize) + i
            }
            crate::data::LineAddressType::Relative(SimpleLineAddressType::Pattern(_), i) => {
                // TODO: Implement
                unimplemented!()
            }
        };

        line_number as usize
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_get_line_number_from_absolute() {
        let mut editor = Editor::new();
        editor.buffer.lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(
                SimpleLineAddressType::LineNumber(0)
            )),
            0
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(
                SimpleLineAddressType::LineNumber(1)
            )),
            0
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(
                SimpleLineAddressType::LineNumber(2)
            )),
            1
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(
                SimpleLineAddressType::LineNumber(3)
            )),
            2
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(
                SimpleLineAddressType::CurrentLine
            )),
            0
        );
        assert_eq!(
            editor
                .get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::FirstLine)),
            0
        );
        assert_eq!(
            editor
                .get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::LastLine)),
            2
        );
        assert_eq!(
            editor
                .get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::AllLines)),
            2
        );
    }

    #[test]
    fn test_search_forward_and_repeat() {
        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["abc def abc".to_string()];
        editor.search(SearchDirection::Forward, "abc").unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 8);
        editor.repeat_search(true).unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 0);
    }

    #[test]
    fn test_search_backward() {
        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["abc def abc".to_string()];
        editor.move_cursor_to(0, 10).unwrap();
        editor.search(SearchDirection::Backward, "abc").unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 8);
        editor.repeat_search(false).unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 0);
    }

    #[test]
    fn test_yank_and_paste() {
        use crate::command::base::{CommandData, JumpCommandData};
        use crossterm::event::KeyCode;

        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["hello world".to_string(), "second".to_string()];

        // yank word 'hello'
        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('y'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: Some(JumpCommandData {
                count: 1,
                key_code: KeyCode::Char('w'),
                modifiers: crossterm::event::KeyModifiers::NONE,
            }),
        };
        editor.execute_command(cmd).unwrap();
        assert_eq!(editor.unnamed_register, "hello ");

        // paste before start of second line
        editor.move_cursor_to(1, 0).unwrap();
        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('P'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: None,
        };
        editor.execute_command(cmd).unwrap();
        assert_eq!(editor.buffer.lines[1], "hello second");

        // yank line 1 (linewise)
        editor.move_cursor_to(0, 0).unwrap();
        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('y'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: Some(JumpCommandData {
                count: 1,
                key_code: KeyCode::Char('y'),
                modifiers: crossterm::event::KeyModifiers::NONE,
            }),
        };
        editor.execute_command(cmd).unwrap();
        assert_eq!(editor.unnamed_register, "hello world\n");

        // paste below last line
        editor.move_cursor_to(1, 0).unwrap();
        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('p'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: None,
        };
        editor.execute_command(cmd).unwrap();
        assert_eq!(editor.buffer.lines[2], "hello world");
    }
}
