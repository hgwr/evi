use std::collections::HashMap;
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
    command::commands::go_to_file::{GoToFirstLine, GoToLastLine},
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
use regex::Regex;

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
    FindChar {
        direction: SearchDirection,
        inclusive: bool,
        count: usize,
    },
    SetMark,
    JumpMark,
    ExCommand,
    Search(SearchDirection),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FindCharInfo {
    pub direction: SearchDirection,
    pub inclusive: bool,
    pub target: char,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Mark {
    pub file_index: usize,
    pub position: CursorPositionInBuffer,
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
    pub last_command: Option<Vec<ExecutedCommand>>,
    pub unnamed_register: String,
    pub unnamed_register_linewise: bool,
    pub last_input_string: String,
    pub ex_command_data: String,
    pub ex_command_cursor: usize,
    pub ex_command_history: Vec<String>,
    pub ex_command_history_index: usize,
    pub ex_command_history_backup: String,
    pub search_query: String,
    pub last_search_pattern: Option<String>,
    pub last_search_direction: Option<SearchDirection>,
    pub last_find: Option<FindCharInfo>,
    pub pending_replace_char_count: usize,
    pub marks: HashMap<char, Mark>,
    pub registers: HashMap<char, String>,
    pub registers_linewise: HashMap<char, bool>,
    pub pending_register: Option<char>,
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
            last_command: None,
            unnamed_register: String::new(),
            unnamed_register_linewise: false,
            last_input_string: "".to_string(),
            ex_command_data: "".to_string(),
            ex_command_cursor: 0,
            ex_command_history: Vec::new(),
            ex_command_history_index: 0,
            ex_command_history_backup: String::new(),
            search_query: String::new(),
            last_search_pattern: None,
            last_search_direction: None,
            last_find: None,
            pending_replace_char_count: 1,
            marks: HashMap::new(),
            registers: HashMap::new(),
            registers_linewise: HashMap::new(),
            pending_register: None,
        }
    }

    pub fn open_file(&mut self, file_path: &PathBuf) {
        match Buffer::from_file(file_path) {
            Ok(buffer) => {
                self.buffer = buffer;
                self.editing_file_paths.push(file_path.clone());
                self.current_file_index = self.editing_file_paths.len() - 1;
            }
            Err(err) => {
                self.status_line = format!("Failed to open file: {}", err);
            }
        }
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
            Mode::FindChar { .. } => {
                self.mode = Mode::Command;
                self.status_line = "".to_string();
            }
            Mode::SetMark | Mode::JumpMark => {
                self.mode = Mode::Command;
                self.status_line = "".to_string();
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
                let chunk = vec![last_executed_command];
                self.last_command = Some(chunk.clone());
                self.command_history.push(chunk);
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
        self.last_command = Some(command_series.clone());
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
            Mode::FindChar { .. } => {
                self.mode = Mode::Insert;
                self.status_line = "-- INSERT --".to_string();
                self.last_input_string = String::new();
            }
            Mode::SetMark | Mode::JumpMark => {
                self.mode = Mode::Insert;
                self.status_line = "-- INSERT --".to_string();
                self.last_input_string = String::new();
            }
        }
    }

    pub fn set_insert_mode_beginning_of_line(&mut self) -> GenericResult<()> {
        let mut move_bol = crate::command::commands::move_cursor::MoveBeginningOfLine {};
        move_bol.execute(self)?;
        self.set_insert_mode();
        Ok(())
    }

    pub fn set_insert_mode_end_of_line(&mut self) -> GenericResult<()> {
        let mut move_eol = crate::command::commands::move_cursor::MoveEndOfLine {};
        move_eol.execute(self)?;
        self.set_insert_mode();
        Ok(())
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
            Mode::Search(_) => {
                self.mode = Mode::Replace;
                self.status_line = "-- REPLACE --".to_string();
                self.last_input_string = String::new();
                self.search_query.clear();
            }
            Mode::FindChar { .. } => {
                self.mode = Mode::Replace;
                self.status_line = "-- REPLACE --".to_string();
                self.last_input_string = String::new();
            }
            Mode::SetMark | Mode::JumpMark => {
                self.mode = Mode::Replace;
                self.status_line = "-- REPLACE --".to_string();
                self.last_input_string = String::new();
            }
        }
    }

    pub fn set_replace_char_mode_with_count(&mut self, count: usize) {
        self.set_replace_char_mode();
        self.pending_replace_char_count = count;
    }

    pub fn set_replace_char_mode(&mut self) {
        self.mode = Mode::ReplaceChar;
        self.last_input_string = String::new();
    }

    pub fn set_find_char_mode(
        &mut self,
        direction: SearchDirection,
        inclusive: bool,
        count: usize,
    ) {
        self.mode = Mode::FindChar {
            direction,
            inclusive,
            count,
        };
    }

    pub fn is_find_char_mode(&self) -> bool {
        matches!(self.mode, Mode::FindChar { .. })
    }

    pub fn set_mark_mode(&mut self) {
        self.mode = Mode::SetMark;
    }

    pub fn set_jump_mark_mode(&mut self) {
        self.mode = Mode::JumpMark;
    }

    pub fn is_set_mark_mode(&self) -> bool {
        matches!(self.mode, Mode::SetMark)
    }

    pub fn is_jump_mark_mode(&self) -> bool {
        matches!(self.mode, Mode::JumpMark)
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
        self.ex_command_data.clear();
        self.ex_command_cursor = 0;
        self.ex_command_history_index = self.ex_command_history.len();
        self.ex_command_history_backup.clear();
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
        let chunk = vec![ExecutedCommand {
            command_data,
            command,
        }];
        // Ex commands should be undoable but should not affect the repeat ('.') state.
        self.command_history.push(chunk);
        if !ex_command_str.is_empty() {
            self.ex_command_history.push(ex_command_str.to_string());
        }
        self.ex_command_history_index = self.ex_command_history.len();
        self.ex_command_data = "".to_string();
        self.ex_command_cursor = 0;
        Ok(())
    }

    fn update_ex_command_status(&mut self) {
        self.status_line = ":".to_owned() + &self.ex_command_data;
    }

    pub fn insert_ex_command_char(&mut self, c: char) {
        let byte_idx = Editor::char_to_byte_index(&self.ex_command_data, self.ex_command_cursor);
        self.ex_command_data.insert(byte_idx, c);
        self.ex_command_cursor += 1;
        self.update_ex_command_status();
    }

    pub fn backspace_ex_command(&mut self) {
        if self.ex_command_cursor > 0 {
            self.ex_command_cursor -= 1;
            let byte_idx =
                Editor::char_to_byte_index(&self.ex_command_data, self.ex_command_cursor);
            self.ex_command_data.remove(byte_idx);
            self.update_ex_command_status();
        }
    }

    pub fn move_ex_command_cursor_left(&mut self) {
        if self.ex_command_cursor > 0 {
            self.ex_command_cursor -= 1;
        }
    }

    pub fn move_ex_command_cursor_right(&mut self) {
        if self.ex_command_cursor < self.ex_command_data.chars().count() {
            self.ex_command_cursor += 1;
        }
    }

    pub fn complete_ex_command(&mut self) {
        use std::fs;
        use std::path::{Path, PathBuf};

        let cursor_byte = Editor::char_to_byte_index(&self.ex_command_data, self.ex_command_cursor);
        let before_cursor = &self.ex_command_data[..cursor_byte];
        let start_byte = before_cursor
            .rfind(|c: char| c.is_whitespace())
            .map(|i| i + 1)
            .unwrap_or(0);

        let prefix = before_cursor[..start_byte].trim_start();
        let command = prefix.split_whitespace().next().unwrap_or("");
        if command != "w" && command != "r" && command != "o" {
            return;
        }

        let partial = &before_cursor[start_byte..];
        let path = Path::new(partial);
        let dir = if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                PathBuf::from(".")
            } else {
                parent.to_path_buf()
            }
        } else {
            PathBuf::from(".")
        };
        let file_name_os_str = path.file_name().unwrap_or_default();
        let file_prefix = if partial.is_empty() || (partial == "." && file_name_os_str == std::ffi::OsStr::new(".")) {
            ""
        } else {
            file_name_os_str.to_str().unwrap_or("")
        };

        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut matches: Vec<String> = entries
            .filter_map(|e| {
                e.ok().and_then(|e| e.file_name().into_string().ok())
                    .filter(|name| name.starts_with(file_prefix))
            })
            .collect();

        matches.sort();
        let candidate = match matches.first() {
            Some(c) => c,
            None => return,
        };

        let mut completed_path = dir.join(candidate).to_string_lossy().to_string();
        if Path::new(&completed_path).is_dir() {
            completed_path.push('/');
        }

        let after_cursor = &self.ex_command_data[cursor_byte..];
        self.ex_command_data = format!(
            "{}{}{}",
            &self.ex_command_data[..start_byte],
            completed_path,
            after_cursor
        );
        let start_chars = self.ex_command_data[..start_byte + completed_path.len()].chars().count();
        self.ex_command_cursor = start_chars;
        self.update_ex_command_status();
    }

    pub fn previous_ex_command(&mut self) {
        if self.ex_command_history.is_empty() {
            return;
        }
        if self.ex_command_history_index == self.ex_command_history.len() {
            self.ex_command_history_backup = self.ex_command_data.clone();
        }
        if self.ex_command_history_index > 0 {
            self.ex_command_history_index -= 1;
            self.ex_command_data = self.ex_command_history[self.ex_command_history_index].clone();
            self.ex_command_cursor = self.ex_command_data.chars().count();
            self.update_ex_command_status();
        }
    }

    pub fn next_ex_command(&mut self) {
        if self.ex_command_history_index < self.ex_command_history.len() {
            self.ex_command_history_index += 1;
            if self.ex_command_history_index == self.ex_command_history.len() {
                self.ex_command_data = self.ex_command_history_backup.clone();
            } else {
                self.ex_command_data =
                    self.ex_command_history[self.ex_command_history_index].clone();
            }
            self.ex_command_cursor = self.ex_command_data.chars().count();
            self.update_ex_command_status();
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

    pub fn take_pending_register(&mut self) -> Option<char> {
        let r = self.pending_register;
        self.pending_register = None;
        r
    }

    pub fn store_register(&mut self, reg: Option<char>, text: String, linewise: bool) {
        self.unnamed_register = text.clone();
        self.unnamed_register_linewise = linewise;
        if let Some(ch) = reg {
            self.registers.insert(ch, text.clone());
            self.registers_linewise.insert(ch, linewise);
        }
    }

    pub fn fetch_register(&self, reg: Option<char>) -> (String, bool) {
        match reg {
            Some(ch) => {
                let text = self.registers.get(&ch).cloned().unwrap_or_default();
                let lw = *self.registers_linewise.get(&ch).unwrap_or(&false);
                (text, lw)
            }
            None => (self.unnamed_register.clone(), self.unnamed_register_linewise),
        }
    }

    pub fn move_cursor_to(&mut self, row: usize, col: usize) -> GenericResult<()> {
        self.cursor_position_in_buffer.row = 0;
        self.cursor_position_in_buffer.col = 0;
        self.cursor_position_on_screen.row = 0;
        self.cursor_position_on_screen.col = 0;
        self.window_position_in_buffer.row = 0;
        self.window_position_in_buffer.col = 0;
        let mut next_line = crate::command::commands::move_cursor::NextLine {};
        while self.cursor_position_in_buffer.row < row {
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

    fn search_line(
        &self,
        re: &Regex,
        start_row: usize,
        direction: SearchDirection,
    ) -> Option<usize> {
        if self.buffer.lines.is_empty() {
            return None;
        }
        match direction {
            SearchDirection::Forward => {
                for i in start_row..self.buffer.lines.len() {
                    if re.is_match(&self.buffer.lines[i]) {
                        return Some(i);
                    }
                }
                None
            }
            SearchDirection::Backward => {
                let mut i = start_row as isize;
                while i >= 0 {
                    if re.is_match(&self.buffer.lines[i as usize]) {
                        return Some(i as usize);
                    }
                    i -= 1;
                }
                None
            }
        }
    }

    pub fn get_ex_command_cursor_col(&self) -> u16 {
        let mut width = crate::util::get_char_width(':');
        for c in self.ex_command_data.chars().take(self.ex_command_cursor) {
            width += crate::util::get_char_width(c);
        }
        width
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
            let repeat_count = if command_data.count == 0 {
                1
            } else {
                command_data.count
            };
            for _ in 0..repeat_count {
                command.execute(self)?;
            }
            if command.is_undoable() {
                let chunk = vec![ExecutedCommand {
                    command_data,
                    command,
                }];
                self.last_command = Some(chunk.clone());
                self.command_history.push(chunk);
            }
        } else if !command.is_modeful() && !command.is_reusable() {
            // Commands like 'G' interpret the count as a target line rather than a repeat count.
            if command.is::<GoToFirstLine>() || command.is::<GoToLastLine>() {
                command.execute(self)?;
                if command.is_undoable() {
                    let chunk = vec![ExecutedCommand {
                        command_data,
                        command,
                    }];
                    self.last_command = Some(chunk.clone());
                    self.command_history.push(chunk);
                }
            } else {
                let mut command_chunk: Vec<ExecutedCommand> = Vec::new();
                let disassemble_command_data = CommandData {
                    count: 1,
                    ..command_data
                };
                let repeat_count = if command_data.count == 0 {
                    1
                } else {
                    command_data.count
                };
                for _ in 0..repeat_count {
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
                    self.last_command = Some(command_chunk.clone());
                    self.command_history.push(command_chunk);
                }
            }
        } else {
            command.execute(self)?;
            if command.is_undoable() {
                let chunk = vec![ExecutedCommand {
                    command_data,
                    command,
                }];
                self.last_command = Some(chunk.clone());
                self.command_history.push(chunk);
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

    pub fn repeat_last_command(&mut self) -> GenericResult<()> {
        if let Some(last_chunk_from_editor_state) = self.last_command.clone() {
            let mut new_chunk_for_history_and_next_repeat: Vec<ExecutedCommand> = Vec::new();
            for executed_command_orig_data in last_chunk_from_editor_state.into_iter() {
                let mut command_instance_for_iteration = executed_command_orig_data.command;
                let repeat_count = executed_command_orig_data.command_data.count;

                for _i in 0..repeat_count {
                    match command_instance_for_iteration.redo(self)? {
                        Some(next_command_state) => {
                            command_instance_for_iteration = next_command_state;
                        }
                        None => {
                            // command_instance_for_iteration was (presumably) mutated by redo(), so we continue to use it.
                        }
                    }
                }
                // After N redos, command_instance_for_iteration holds the final state of this command part
                new_chunk_for_history_and_next_repeat.push(ExecutedCommand {
                    command_data: executed_command_orig_data.command_data, // Store original CommandData (with its count)
                    command: command_instance_for_iteration,
                });
            }
            if !new_chunk_for_history_and_next_repeat.is_empty() {
                self.last_command = Some(new_chunk_for_history_and_next_repeat.clone());
                self.command_history
                    .push(new_chunk_for_history_and_next_repeat);
            }
        }
        Ok(())
    }

    pub fn render(self: &mut Editor, stdout: &mut std::io::Stdout) -> GenericResult<()> {
        render(self, stdout)
    }

    pub fn content_height(&self) -> u16 {
        self.terminal_size.height - 1
    }

    pub fn max_content_row_index(&self) -> u16 {
        self.content_height().saturating_sub(1)
    }

    pub fn page_down(&mut self) -> GenericResult<()> {
        let height = self.content_height() as usize;
        let overlap = if height > 1 { 2 } else { 1 };
        let scroll = height.saturating_sub(overlap);
        let col = self.cursor_position_in_buffer.col;
        let max_top = self.buffer.lines.len().saturating_sub(1);
        self.window_position_in_buffer.row =
            (self.window_position_in_buffer.row + scroll).min(max_top);
        self.cursor_position_in_buffer.row = self.window_position_in_buffer.row;
        self.cursor_position_in_buffer.col = 0;
        self.cursor_position_on_screen.row = 0;
        self.cursor_position_on_screen.col = 0;
        let dest_col = col.min(self.get_num_of_current_line_chars());
        self.move_cursor_within_line(dest_col)?;
        Ok(())
    }

    pub fn page_up(&mut self) -> GenericResult<()> {
        let height = self.content_height() as usize;
        let overlap = if height > 1 { 2 } else { 1 };
        let scroll = height.saturating_sub(overlap);
        let col = self.cursor_position_in_buffer.col;
        self.window_position_in_buffer.row =
            self.window_position_in_buffer.row.saturating_sub(scroll);
        self.cursor_position_in_buffer.row = self.window_position_in_buffer.row;
        self.cursor_position_in_buffer.col = 0;
        self.cursor_position_on_screen.row = 0;
        self.cursor_position_on_screen.col = 0;
        let dest_col = col.min(self.get_num_of_current_line_chars());
        self.move_cursor_within_line(dest_col)?;
        Ok(())
    }

    pub fn line_height(&self, row: usize) -> usize {
        let width = (self.terminal_size.width as usize).max(1); // Ensure width is at least 1 to prevent division by zero
        self.buffer
            .lines
            .get(row)
            .map(|line| {
                let len = line.chars().count();
                if len == 0 {
                    1
                } else {
                    (len - 1) / width + 1
                }
            })
            .unwrap_or(1)
    }

    pub fn scroll_to_cursor_bottom(&mut self) {
        let height = self.content_height() as usize;
        let mut rows_needed = self.line_height(self.cursor_position_in_buffer.row);
        let mut start = self.cursor_position_in_buffer.row;
        while start > 0 {
            let next_height = self.line_height(start - 1);
            if rows_needed + next_height > height {
                break;
            }
            start -= 1;
            rows_needed += next_height;
        }
        self.window_position_in_buffer.row = start;
        let mut pos = 0usize;
        for r in start..self.cursor_position_in_buffer.row {
            pos += self.line_height(r);
        }
        self.cursor_position_on_screen.row = pos as u16;
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
            if self.cursor_position_on_screen.row < self.max_content_row_index() {
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
            if self.cursor_position_on_screen.row < self.max_content_row_index() {
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

    fn move_cursor_within_line(&mut self, col: usize) -> GenericResult<()> {
        let current = self.cursor_position_in_buffer.col;
        if col > current {
            let mut f = crate::command::commands::move_cursor::ForwardChar {};
            for _ in 0..(col - current) {
                f.execute(self)?;
            }
        } else if col < current {
            let mut b = crate::command::commands::move_cursor::BackwardChar {};
            for _ in 0..(current - col) {
                b.execute(self)?;
            }
        }
        Ok(())
    }

    pub fn find_char_in_current_line(
        &mut self,
        direction: SearchDirection,
        inclusive: bool,
        count: usize,
        target: char,
    ) -> GenericResult<()> {
        if let Some(line) = self.buffer.lines.get(self.cursor_position_in_buffer.row) {
            let chars: Vec<char> = line.chars().collect();
            let mut idx = self.cursor_position_in_buffer.col;
            match direction {
                SearchDirection::Forward => {
                    idx += 1;
                    let mut remaining = count;
                    while idx < chars.len() {
                        if chars[idx] == target {
                            remaining -= 1;
                            if remaining == 0 {
                                let dest = if inclusive { idx } else { idx - 1 };
                                self.move_cursor_within_line(dest)?;
                                return Ok(());
                            }
                        }
                        idx += 1;
                    }
                    self.display_visual_bell()?;
                }
                SearchDirection::Backward => {
                    if idx > 0 {
                        idx -= 1;
                        let mut remaining = count;
                        loop {
                            if chars[idx] == target {
                                remaining -= 1;
                                if remaining == 0 {
                                    let dest = if inclusive { idx } else { idx + 1 };
                                    self.move_cursor_within_line(dest)?;
                                    return Ok(());
                                }
                            }
                            if idx == 0 {
                                break;
                            }
                            idx -= 1;
                        }
                    }
                    self.display_visual_bell()?;
                }
            }
        }
        Ok(())
    }

    pub fn execute_find_char(&mut self, c: char) -> GenericResult<()> {
        if let Mode::FindChar {
            direction,
            inclusive,
            count,
        } = self.mode
        {
            self.find_char_in_current_line(direction, inclusive, count, c)?;
            self.last_find = Some(FindCharInfo {
                direction,
                inclusive,
                target: c,
            });
            self.set_command_mode();
        }
        Ok(())
    }

    pub fn repeat_find_char(&mut self) -> GenericResult<()> {
        if let Some(info) = self.last_find {
            if info.direction == SearchDirection::Backward && !info.inclusive {
                if self.cursor_position_in_buffer.col > 0 {
                    let mut b = crate::command::commands::move_cursor::BackwardChar {};
                    b.execute(self)?;
                }
            }
            self.find_char_in_current_line(info.direction, info.inclusive, 1, info.target)?;
        } else {
            self.display_visual_bell()?;
        }
        Ok(())
    }

    pub fn set_mark(&mut self, c: char) {
        let mark = Mark {
            file_index: self.current_file_index,
            position: self.cursor_position_in_buffer,
        };
        self.marks.insert(c, mark);
    }

    pub fn jump_to_mark(&mut self, c: char) -> GenericResult<()> {
        if let Some(mark) = self.marks.get(&c) {
            if mark.file_index != self.current_file_index {
                if let Some(path) = self.editing_file_paths.get(mark.file_index) {
                    self.buffer = Buffer::from_file(path)?;
                    self.current_file_index = mark.file_index;
                }
            }
            self.move_cursor_to(mark.position.row, mark.position.col)?;
        } else {
            self.display_visual_bell()?;
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
        if self.cursor_position_on_screen.row < self.max_content_row_index() {
            self.cursor_position_on_screen.row += 1;
        } else {
            self.window_position_in_buffer.row += 1;
        }
        self.cursor_position_on_screen.col = 0;
        self.last_input_string.push('\n');
        Ok(())
    }

    fn resolve_simple_line_address(&mut self, addr: &SimpleLineAddressType) -> isize {
        match addr {
            SimpleLineAddressType::LineNumber(n) => {
                let input = *n as isize;
                if input == 0 {
                    0
                } else {
                    input - 1
                }
            }
            SimpleLineAddressType::CurrentLine => self.cursor_position_in_buffer.row as isize,
            SimpleLineAddressType::FirstLine => 0,
            SimpleLineAddressType::LastLine | SimpleLineAddressType::AllLines => {
                self.buffer.lines.len().saturating_sub(1) as isize
            }
            SimpleLineAddressType::Pattern(p) => {
                let re = Regex::new(&p.pattern).ok();
                if let Some(re) = re {
                    if let Some(idx) =
                        self.search_line(&re, self.cursor_position_in_buffer.row, SearchDirection::Forward)
                    {
                        idx as isize
                    } else {
                        self.buffer.lines.len().saturating_sub(1) as isize
                    }
                } else {
                    0
                }
            }
        }
    }

    pub fn get_line_number_from(&mut self, line_address: &LineAddressType) -> usize {
        let line_number: isize = match line_address {
            LineAddressType::Absolute(addr) => self.resolve_simple_line_address(addr),
            LineAddressType::Relative(addr, offset) => {
                self.resolve_simple_line_address(addr) + *offset
            }
        };

        // Negative results may occur for relative addresses (e.g., "-1"). Clamp
        // to zero before converting to `usize` to avoid underflow.
        if line_number < 0 {
            0
        } else {
            line_number as usize
        }
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
    use tempfile::NamedTempFile;

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
    fn test_editor_get_line_number_from_relative_negative() {
        let mut editor = Editor::new();
        editor.buffer.lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        // Offsetting from the current line at the top should clamp to the first line.
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Relative(
                SimpleLineAddressType::CurrentLine,
                -1,
            )),
            0
        );

        // Negative offset that stays within bounds should work normally.
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Relative(
                SimpleLineAddressType::LastLine,
                -1,
            )),
            1
        );

        // Large negative offsets should also clamp to zero.
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Relative(
                SimpleLineAddressType::FirstLine,
                -3,
            )),
            0
        );
    }

    #[test]
    fn test_editor_get_line_number_from_relative_positive() {
        let mut editor = Editor::new();
        editor.buffer.lines = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        editor.cursor_position_in_buffer.row = 1;

        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Relative(
                SimpleLineAddressType::CurrentLine,
                1,
            )),
            2
        );

        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Relative(
                SimpleLineAddressType::CurrentLine,
                5,
            )),
            6
        );
    }

    #[test]
    fn test_editor_get_line_number_from_pattern() {
        use crate::data::Pattern;
        let mut editor = Editor::new();
        editor.buffer.lines = vec![
            "foo".to_string(),
            "bar".to_string(),
            "baz".to_string(),
            "foo".to_string(),
        ];

        let addr = LineAddressType::Absolute(SimpleLineAddressType::Pattern(Pattern {
            pattern: "baz".to_string(),
        }));
        assert_eq!(editor.get_line_number_from(&addr), 2);

        let addr = LineAddressType::Relative(
            SimpleLineAddressType::Pattern(Pattern {
                pattern: "foo".to_string(),
            }),
            2,
        );
        assert_eq!(editor.get_line_number_from(&addr), 2);

        let addr = LineAddressType::Absolute(SimpleLineAddressType::Pattern(Pattern {
            pattern: "nomatch".to_string(),
        }));
        assert_eq!(editor.get_line_number_from(&addr), 3);
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

    #[test]
    fn test_named_register_yank_and_paste() {
        use crate::command::base::{CommandData, JumpCommandData};
        use crossterm::event::KeyCode;

        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["hello world".to_string(), "second".to_string()];

        // set pending register to 'a'
        editor.pending_register = Some('a');
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
        assert_eq!(editor.registers.get(&'a').unwrap(), "hello world\n");

        editor.move_cursor_to(1, 0).unwrap();
        editor.pending_register = Some('a');
        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('p'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: None,
        };
        editor.execute_command(cmd).unwrap();
        assert_eq!(editor.buffer.lines[2], "hello world");
    }

    #[test]
    fn test_find_char_and_repeat() {
        use crate::command::base::CommandData;
        use crossterm::event::KeyCode;

        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["abcabcabc".to_string()];

        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('f'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: None,
        };
        editor.execute_command(cmd).unwrap();
        editor.execute_find_char('c').unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 2);
        editor.repeat_find_char().unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 5);
    }

    #[test]
    fn test_backward_to_char_and_repeat() {
        use crate::command::base::CommandData;
        use crossterm::event::KeyCode;

        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["abcabcabc".to_string()];
        editor.move_cursor_to(0, 8).unwrap();

        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('T'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: None,
        };
        editor.execute_command(cmd).unwrap();
        editor.execute_find_char('a').unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 7);
        editor.repeat_find_char().unwrap();
        assert_eq!(editor.cursor_position_in_buffer.col, 4);
    }

    #[test]
    fn test_page_down_and_up() {
        let mut editor = Editor::new();
        editor.resize_terminal(20, 4);
        editor.buffer.lines = vec![
            "l1".to_string(),
            "l2".to_string(),
            "l3".to_string(),
            "l4".to_string(),
            "l5".to_string(),
        ];

        editor.page_down().unwrap();
        assert_eq!(editor.cursor_position_in_buffer.row, 1);

        editor.page_up().unwrap();
        assert_eq!(editor.cursor_position_in_buffer.row, 0);
    }

    #[test]
    fn test_scroll_to_cursor_bottom_wrapped_lines() {
        let mut editor = Editor::new();
        editor.resize_terminal(10, 5); // content height is 4
        editor.buffer.lines = vec!["a".repeat(30), "b".repeat(15)];
        editor.move_cursor_to(1, 0).unwrap();
        editor.scroll_to_cursor_bottom();
        assert_eq!(editor.window_position_in_buffer.row, 1);
        assert_eq!(editor.cursor_position_on_screen.row, 0);
    }

    #[test]
    fn test_open_file_failure_sets_status_line() {
        let temp = NamedTempFile::new().unwrap();
        let path = temp.path().to_path_buf();
        drop(temp);

        let mut editor = Editor::new();
        editor.open_file(&path);
        assert!(editor.status_line.starts_with("Failed to open file"));
    }

    #[test]
    fn test_append_at_end_of_line() {
        use crate::command::base::CommandData;
        use crossterm::event::KeyCode;

        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["abc".to_string()];
        editor.move_cursor_to(0, 2).unwrap();

        let cmd = CommandData {
            count: 1,
            key_code: KeyCode::Char('a'),
            modifiers: crossterm::event::KeyModifiers::NONE,
            range: None,
        };
        editor.execute_command(cmd).unwrap();

        assert!(editor.is_insert_mode());
        assert_eq!(editor.cursor_position_in_buffer.col, 3);
        assert_eq!(editor.cursor_position_on_screen.col, 3);
    }

    #[test]
    fn test_set_and_jump_mark() {
        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.buffer.lines = vec!["abc".to_string(), "def".to_string()];
        editor.move_cursor_to(1, 1).unwrap();
        editor.set_mark('a');
        editor.move_cursor_to(0, 0).unwrap();
        editor.jump_to_mark('a').unwrap();
        assert_eq!(editor.cursor_position_in_buffer.row, 1);
        assert_eq!(editor.cursor_position_in_buffer.col, 1);
    }

    #[test]
    fn test_jump_mark_across_files() {
        use std::fs::write;
        let tmp1 = NamedTempFile::new().unwrap();
        write(tmp1.path(), "one\ntwo\n").unwrap();
        let tmp2 = NamedTempFile::new().unwrap();
        write(tmp2.path(), "alpha\nbeta\n").unwrap();

        let mut editor = Editor::new();
        editor.resize_terminal(80, 24);
        editor.open_file(&tmp1.path().to_path_buf());
        editor.open_file(&tmp2.path().to_path_buf());

        // mark in first file
        editor.current_file_index = 0;
        editor.buffer = Buffer::from_file(&tmp1.path().to_path_buf()).unwrap();
        editor.move_cursor_to(1, 0).unwrap();
        editor.set_mark('a');

        // switch to second file
        editor.current_file_index = 1;
        editor.buffer = Buffer::from_file(&tmp2.path().to_path_buf()).unwrap();
        editor.jump_to_mark('a').unwrap();
        assert_eq!(editor.current_file_index, 0);
        assert_eq!(editor.cursor_position_in_buffer.row, 1);
    }
}
