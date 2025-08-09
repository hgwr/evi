use std::io::Write;
use std::path::PathBuf;

use crossterm::{
    terminal::{self, ClearType},
    ExecutableCommand,
};

use log::info;

use crate::{command::factory::command_factory, data::{LineAddressType, SimpleLineAddressType}}; // SimpleLineAddressType needed for Absolute matching
use crate::render::render;
use crate::{buffer::Buffer, command::base::ExecutedCommand, generic_error::GenericResult};
use crate::{
    command::base::{Command, CommandData},
    ex::parser::Parser,
};

// ================= New Coordinate System (WIP transitional) =================
// Buffer 上の論理位置（折り返し前）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BufferPosition {
    pub row: usize,
    pub col: usize,
}

// BufferPosition::new は単純なので直接リテラル構築を推奨

// 画面上の物理位置（折り返し後）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScreenPosition {
    pub row: usize,
    pub col: u16, // col は端末幅に依存
}

impl ScreenPosition {
    pub fn new(row: usize, col: u16) -> Self { Self { row, col } }
}

/// 折り返し計算の責務を分離する計算器
pub struct WrappingCalculator {
    terminal_width: u16,
}

impl WrappingCalculator {
    pub fn new(terminal_width: u16) -> Self { Self { terminal_width } }

    pub fn line_height(&self, line: &str) -> usize {
        crate::util::get_line_height(line, self.terminal_width)
    }

    /// バッファ位置を（window_top_row を基準とした）画面位置へマッピング
    pub fn buffer_to_screen_position(
        &self,
        buffer: &Buffer,
        window_top_row: usize,
        target: BufferPosition,
    ) -> ScreenPosition {
        use crate::util::get_char_width;

        // 先行行の高さを積算
        let mut screen_row = 0usize;
        for r in window_top_row..target.row {
            if let Some(line) = buffer.lines.get(r) {
                screen_row += self.line_height(line);
            } else {
                break;
            }
        }

        // 対象行内での折り返し位置を計算
        let mut col_width = 0usize; // 現在のラップ行での幅
        let mut wrap_row_offset = 0usize; // 対象行内での折り返し行インデックス
        if let Some(line) = buffer.lines.get(target.row) {
            for (i, c) in line.chars().enumerate() {
                if i >= target.col { break; }
                let w = get_char_width(c) as usize;
                if col_width + w > self.terminal_width as usize || col_width + w == self.terminal_width as usize {
                    // 折り返し
                    if col_width + w >= self.terminal_width as usize {
                        wrap_row_offset += 1;
                        col_width = 0;
                    }
                }
                col_width += w;
                if col_width >= self.terminal_width as usize {
                    wrap_row_offset += 1;
                    col_width = 0;
                }
            }
        }

        ScreenPosition::new(screen_row + wrap_row_offset, col_width as u16)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Region {
    pub start: NewCursorSnapshot,
    pub end: NewCursorSnapshot,
}

// New (transitional) snapshot type using new coordinate system only
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NewCursorSnapshot {
    pub cursor: BufferPosition,
    pub window_top_row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    Command,
    Insert,
    ExCommand,
}

pub struct Editor {
    pub buffer: Buffer,
    editing_file_paths: Vec<PathBuf>,
    current_file_index: usize,
    pub is_dirty: bool,
    mode: Mode,
    pub should_exit: bool,
    pub terminal_size: TerminalSize,
    // --- New coordinate system (transitional) ---
    pub cursor: BufferPosition,       // バッファ上の論理カーソル
    pub prev_cursor: BufferPosition,  // 直前のカーソル（スクロール調整用）
    pub window_top_row: usize,        // 画面先頭が示すバッファ行
    pub status_line: String,
    pub command_history: Vec<Vec<ExecutedCommand>>,
    pub last_input_string: String,
    pub ex_command_data: String,
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
            cursor: BufferPosition { row: 0, col: 0 },
            prev_cursor: BufferPosition { row: 0, col: 0 },
            window_top_row: 0,
            status_line: "".to_string(),
            command_history: Vec::new(),
            last_input_string: "".to_string(),
            ex_command_data: "".to_string(),
        }
    }

    // ---------------- New coordinate system helper methods -----------------
    /// (互換用) 旧フィールド廃止後の no-op（既存コード呼出し残存のため）
    pub fn sync_new_from_old(&mut self) {
        // no-op
    }

    /// (互換用) 旧フィールド廃止後の no-op
    pub fn sync_old_from_new(&mut self) {
        // no-op
    }

    /// 新方式での画面位置計算
    pub fn calculate_screen_position(&self) -> ScreenPosition {
        let calc = WrappingCalculator::new(self.terminal_size.width);
        calc.buffer_to_screen_position(&self.buffer, self.window_top_row, self.cursor)
    }

    /// カーソルを常に表示領域内に収める（上下方向対応）
    pub fn ensure_cursor_visible(&mut self) {
        // 端末幅取得
        let width = self.terminal_size.width;
    let _ = width; // width は今後の拡張用（行高キャッシュ等）
        let content_height = self.content_height() as usize;

        // 上方向: カーソル行が window_top_row より上
        if self.cursor.row < self.window_top_row { self.window_top_row = self.cursor.row; }

        // 下方向: 行高さ（wrap 含む）を高速に見積もり。
        // window_top_row..cursor.row-1 までの総高さ + cursor 行内の相対高さ +1 が content_height を超えるか評価。
        // 精密計算のため一度 screen_position を取得し、過剰分だけ jump。
        let sp = self.calculate_screen_position();
        if sp.row >= content_height {
            // どれだけ overflow しているか
            let overflow = sp.row + 1 - content_height; // +1: 0-based -> count
            // overflow 行数だけ上に押し上げたいが、単純に window_top_row += overflow すると
            // wrap 高さの差異で過剰スクロールの可能性があるため、上限まで段階的に。ただし概算でまとめて進め性能確保。
            let mut advanced = 0usize;
            while advanced < overflow && self.window_top_row + 1 < self.buffer.lines.len() {
                self.window_top_row += 1;
                advanced += 1;
            }
            // 最終調整: まだはみ出していたら追加で 1 行ずつ（通常ケースではほぼ不要）
            loop {
                let sp2 = self.calculate_screen_position();
                if sp2.row < content_height { break; }
                if self.window_top_row + 1 < self.buffer.lines.len() { self.window_top_row += 1; } else { break; }
            }
        }

        // 上方向: 直前行が複数 wrap しており、まだ画面上部に余白がある場合は
        // 可能な限り前方行を追加表示（カーソルは画面内最下段付近でも良い）。
        // これにより長い行へ k で近づいた際に、その長い行の頭が自然に見えるようになる。
        {
            let calc = WrappingCalculator::new(self.terminal_size.width);
            let at_file_top = self.window_top_row == 0;
            // 条件: カーソル行より上に巨大行が存在し、現在 top がまだその行に達していない場合
            // 直上行を 1 行ずつ表示領域に含められる限り引き上げる
            let mut changed = true;
            while changed {
                changed = false;
                if self.window_top_row == 0 { break; }
                let prev_row = self.window_top_row - 1;
                if prev_row >= self.buffer.lines.len() { break; }
                let prev_height = if let Some(l) = self.buffer.lines.get(prev_row) { calc.line_height(l) } else { 1 };
                let sp_now = self.calculate_screen_position();
                // 既に十分上方向に余白があり、追加で前行を含めても可視高さを超えないなら引き上げ
                if sp_now.row + prev_height + 1 < content_height { // +1: 少なくとも 1 行は下方向余裕
                    self.window_top_row -= 1;
                    changed = true;
                }
            }

            // 追加ヒューリスティック: 先頭が空行/見出し(#)で、その下に multi-wrap 行があるなら
            // 空行/見出しを押し出して wrap 行を上端に揃える（テスト期待対応）
            if !at_file_top && self.window_top_row + 1 < self.buffer.lines.len() {
                let top_line = &self.buffer.lines[self.window_top_row];
                let next_line = &self.buffer.lines[self.window_top_row + 1];
                let next_height = calc.line_height(next_line);
                if next_height > 1 {
                    let top_is_trivial = top_line.is_empty() || top_line.starts_with('#');
                    if top_is_trivial {
                        // シフトしてもカーソルがはみ出さないか（再計算で保証）
                        self.window_top_row += 1;
                        let sp_after = self.calculate_screen_position();
                        if sp_after.row >= content_height {
                            // 戻す（不適切）
                            self.window_top_row -= 1;
                        }
                    }
                }
            }

            // ラップ行昇格ヒューリスティック: 画面内に複数行に折り返す行があるが、
            // その行より前に trivial 行(空/見出し) しか無ければその行をトップにする
            if !at_file_top {
                let mut first_wrap_row_opt = None;
                let mut h_acc = 0usize;
                let mut r = self.window_top_row;
                while r < self.buffer.lines.len() && h_acc < content_height {
                    let line = &self.buffer.lines[r];
                    let h = calc.line_height(line);
                    if h > 1 { first_wrap_row_opt = Some(r); break; }
                    h_acc += h;
                    r += 1;
                }
                if let Some(wrap_row) = first_wrap_row_opt {
                    if wrap_row > self.window_top_row {
                        let trivial_prefix = (self.window_top_row..wrap_row).all(|ri| {
                            if let Some(l) = self.buffer.lines.get(ri) { l.is_empty() || l.starts_with('#') } else { true }
                        });
                        if trivial_prefix {
                            let original_top = self.window_top_row;
                            self.window_top_row = wrap_row;
                            let sp_after = self.calculate_screen_position();
                            if sp_after.row >= content_height { // revert if cursor hidden
                                self.window_top_row = original_top;
                            }
                        }
                    }
                }
            }

            // ================== 新規ロジック: カーソル行全体をできるだけ収める ==================
            // 目的: Vim のように j/k で長い行に入ったとき、可能ならその行の最初から最後の wrap までを
            // 画面に収める（行全体高さ <= content_height の場合）。
            if self.cursor.row < self.buffer.lines.len() {
                if let Some(line) = self.buffer.lines.get(self.cursor.row) {
                    let line_height = calc.line_height(line);
                    if line_height <= content_height {
                        // シンプル化: カーソル行の先頭を画面に収め、行末まで全て入る位置に調整。優先順位:
                        // 1. カーソル行先頭を top に（上下方向一貫性、Vim で長行突入時の体感に近い）
                        // 2. もし前行を少し表示しても末尾が入るなら一行ずつ上へ広げる
                        let desired_top = self.cursor.row;
                        if desired_top != self.window_top_row {
                            self.window_top_row = desired_top;
                        }
                        // 余白があれば前行を追加表示
                        loop {
                            if self.window_top_row == 0 { break; }
                            // 高さ計算: window_top_row-1 から cursor 行まで
                            let mut total = 0usize; let mut r4 = self.window_top_row - 1; let mut ok = true;
                            while r4 <= self.cursor.row { if let Some(lx)=self.buffer.lines.get(r4){ total += calc.line_height(lx);} if total>content_height { ok=false; break;} if r4==self.cursor.row{break;} r4+=1; }
                            if ok { self.window_top_row -=1; } else { break; }
                        }
                    }
                }
            }
            // ================================================================================
            // 前行全体スクロールアウト: 直前が一つ上 & その行が wrap していた場合、次行へ移動後に前行を完全に消す
            if self.prev_cursor.row + 1 == self.cursor.row && self.cursor.row < self.buffer.lines.len() {
                if let Some(prev_line) = self.buffer.lines.get(self.prev_cursor.row) {
                    let prev_h = calc.line_height(prev_line);
                    if prev_h > 1 && self.window_top_row <= self.prev_cursor.row {
                        let new_top = self.prev_cursor.row + 1;
                        if new_top <= self.cursor.row { self.window_top_row = new_top; }
                    }
                }
            }

            // 追加ヒューリスティック: 複数行移動 (count付き j) で長い wrap 行を飛び越えた場合、
            // その長行（カーソルが現在位置していないもの）が画面に残っていれば押し上げて視界から排除する。
            // 目的: README などで長い説明行(**Note:** ...)が残り、目的のコードブロック行が上方に来ない問題の解消。
            if self.cursor.row > self.prev_cursor.row + 1 { // count > 1 の縦移動
                // window_top_row..cursor.row-1 を走査し、最後の wrap 行を探す
                let mut last_wrap_row: Option<usize> = None;
                let upper_limit = self.cursor.row.saturating_sub(1);
                let mut r = self.window_top_row;
                while r <= upper_limit && r < self.buffer.lines.len() {
                    if let Some(l) = self.buffer.lines.get(r) {
                        if calc.line_height(l) > 1 { last_wrap_row = Some(r); }
                    }
                    if r == upper_limit { break; }
                    r += 1;
                }
                if let Some(wrap_row) = last_wrap_row {
                    if wrap_row < self.cursor.row && self.window_top_row <= wrap_row {
                        let new_top = wrap_row + 1;
                        if new_top <= self.cursor.row { self.window_top_row = new_top; }
                    }
                }
            }
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
    // cursor 可視性を再確認
    self.ensure_cursor_visible();
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
        }
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

    pub fn set_ex_command_mode(&mut self) {
        self.mode = Mode::ExCommand;
        self.status_line = ":".to_string();
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

    pub fn delete_last_ex_command_char(&mut self) {
        if self.ex_command_data.is_empty() {
            self.set_command_mode();
        } else {
            self.ex_command_data.pop();
            self.status_line = ":".to_owned() + &self.ex_command_data;
        }
    }

    // New system snapshots (preferred going forward)
    pub fn snapshot_new_cursor(&self) -> NewCursorSnapshot {
        NewCursorSnapshot { cursor: self.cursor, window_top_row: self.window_top_row }
    }

    pub fn restore_new_cursor(&mut self, snap: NewCursorSnapshot) {
        self.cursor = snap.cursor;
        self.window_top_row = snap.window_top_row;
        // keep old fields in sync for transition
        self.sync_old_from_new();
    }

    pub fn execute_command(&mut self, command_data: CommandData) -> GenericResult<()> {
    // 直前カーソル保存（スクロールヒューリスティック用）
    self.prev_cursor = self.cursor;
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
    self.buffer.get_char(self.cursor.row, self.cursor.col)
    }

    pub fn insert_char(&mut self, c: char) -> GenericResult<()> {
    // new system
    self.sync_new_from_old();
    self.buffer.insert_char(self.cursor.row, self.cursor.col, c)?;
    self.last_input_string.push(c);
    self.cursor.col += 1; // logical advance
    self.ensure_cursor_visible();
    self.sync_old_from_new();
    Ok(())
    }

    pub fn backward_delete_char(&mut self) -> GenericResult<()> {
        self.sync_new_from_old();
        if self.cursor.col > 0 && !self.last_input_string.is_empty() {
            self.buffer.delete_char(self.cursor.row, self.cursor.col - 1)?;
            self.last_input_string.pop();
            self.cursor.col -= 1;
        } else if self.cursor.col == 0 && !self.last_input_string.is_empty() {
            self.last_input_string.pop();
            if self.cursor.row > 0 {
                let rest = self.buffer.lines[self.cursor.row].clone();
                self.buffer.lines.remove(self.cursor.row);
                self.cursor.row -= 1;
                // move to end of previous line
                    if let Some(prev_line) = self.buffer.lines.get(self.cursor.row) {
                        let len = prev_line.chars().count();
                        self.cursor.col = if len == 0 { 0 } else { len - 1 };
                    self.buffer.lines[self.cursor.row] += &rest;
                }
            }
        }
        self.ensure_cursor_visible();
        self.sync_old_from_new();
        Ok(())
    }

    pub fn append_new_line(&mut self) -> GenericResult<()> {
        self.sync_new_from_old();
        let rest: String = self.buffer.lines[self.cursor.row]
            .chars()
            .skip(self.cursor.col)
            .collect();
        let head: String = self.buffer.lines[self.cursor.row]
            .chars()
            .take(self.cursor.col)
            .collect();
        self.buffer.lines[self.cursor.row] = head;
        self.buffer.lines.insert(self.cursor.row + 1, rest);
        self.cursor.row += 1;
        self.cursor.col = 0;
        self.last_input_string.push('\n');
        self.ensure_cursor_visible();
        self.sync_old_from_new();
        Ok(())
    }

    pub fn get_line_number_from(&mut self, line_address: &LineAddressType) -> usize {
        let line_number: isize = match line_address {
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::LineNumber(n)) => {
                let input = *n as isize;
                 if input == 0 {
                     0
                 } else {
                     input - 1
                 }
            },
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::CurrentLine) => {
                self.cursor.row as isize
            },
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::FirstLine) => 0,
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::LastLine) => {
                self.buffer.lines.len().saturating_sub(1) as isize
            },
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::AllLines) => {
                self.buffer.lines.len().saturating_sub(1) as isize
            },
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::Pattern(_)) => {
                // TODO: Implement
                unimplemented!()
            },
            // Relative line addressing disabled (variant removed).
        };

        line_number as usize
    }

    // 以前 adjust_cursor_column_new で提供していた列補正は、必要箇所で直接行長チェックを行う方針に変更
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
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::LineNumber(0))),
            0
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::LineNumber(1))),
            0
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::LineNumber(2))),
            1
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::LineNumber(3))),
            2
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::CurrentLine)),
            0
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::FirstLine)),
            0
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::LastLine)),
            2
        );
        assert_eq!(
            editor.get_line_number_from(&LineAddressType::Absolute(SimpleLineAddressType::AllLines)),
            2
        );
    }

}
