use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;
// (legacy imports removed after refactor)

pub struct ForwardChar;
impl Command for ForwardChar {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.sync_new_from_old();
        if let Some(line) = editor.buffer.lines.get(editor.cursor.row) {
            let num_of_chars = line.chars().count();
            if editor.cursor.col + 1 < num_of_chars {
                editor.cursor.col += 1;
                editor.ensure_cursor_visible(); // wrap 末尾でのスクロール調整
            }
        }
        editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct BackwardChar;
impl Command for BackwardChar {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
    editor.sync_new_from_old();
    if editor.cursor.col > 0 { editor.cursor.col -= 1; }
    // 上方向スクロールは未実装（後で ensure_cursor_visible 拡張）
    editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct MoveBeginningOfLine;
impl Command for MoveBeginningOfLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
    editor.sync_new_from_old();
    editor.cursor.col = 0;
    editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct MoveEndOfLine;
impl Command for MoveEndOfLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.sync_new_from_old();
        if let Some(line) = editor.buffer.lines.get(editor.cursor.row) {
            let len = line.chars().count();
            // Vi の $ は行末文字の上にカーソルを置く（1 文字も無い場合は 0）
            editor.cursor.col = if len == 0 { 0 } else { len - 1 };
        }
        editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct NextLine;
impl Command for NextLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        // --- transitional sync (old->new) ---
        editor.sync_new_from_old();

        if editor.cursor.row + 1 < editor.buffer.lines.len() {
            let preserve_col = editor.cursor.col;
            editor.cursor.row += 1;
            // 行長を超えないよう補正
            let line_len = editor.buffer.lines[editor.cursor.row].chars().count();
            if editor.cursor.col >= line_len { editor.cursor.col = if line_len == 0 { 0 } else { line_len - 1 }; }
            // 表示調整
            editor.ensure_cursor_visible();
            // 列復元 (wrap 内で短い場合は補正済)
            if line_len == 0 { editor.cursor.col = 0; }
            else { editor.cursor.col = std::cmp::min(preserve_col, line_len - 1); }
            // 直後に最終行がまだ画面に入っていない場合、余裕が 1 行以下なら先行スクロールしておく
            if editor.cursor.row + 1 < editor.buffer.lines.len() {
                // 画面残り高さを概算（現在カーソルの screen row を計算）
                let sp = editor.calculate_screen_position();
                let remaining = (editor.content_height() as usize).saturating_sub(sp.row + 1);
                if remaining == 0 { // もう余裕が無い: 次の j で確実にスクロールさせるため先に 1 行押し上げる
                    if editor.window_top_row + 1 < editor.buffer.lines.len() {
                        editor.window_top_row += 1; // 予防スクロール
                    }
                }
            }
    } // EOF 時はスクロールしない（行移動が無いので表示維持）

        // --- transitional sync (new->old) ---
        editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct PreviousLine;
impl Command for PreviousLine {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.sync_new_from_old();
        if editor.cursor.row > 0 { editor.cursor.row -= 1; }
        // 行長補正
        if let Some(line) = editor.buffer.lines.get(editor.cursor.row) {
            let len = line.chars().count();
            if editor.cursor.col > len { editor.cursor.col = len; }
        }
    editor.ensure_cursor_visible();
        editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ForwardWord;
impl Command for ForwardWord {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.sync_new_from_old();
        // 1文字進めてから「単語境界」を探す挙動を再現
        if let Some(line) = editor.buffer.lines.get(editor.cursor.row) {
            if editor.cursor.col < line.chars().count().saturating_sub(1) {
                editor.cursor.col += 1;
            }
        }
        loop {
            let line_opt = editor.buffer.lines.get(editor.cursor.row).cloned();
            if line_opt.is_none() { break; }
            let line = line_opt.unwrap();
            let len = line.chars().count();
            if editor.cursor.col + 1 < len {
                // consume non-space
                while editor.cursor.col + 1 < len {
                    let ch = line.chars().nth(editor.cursor.col).unwrap();
                    if ch.is_whitespace() { break; }
                    editor.cursor.col += 1;
                }
                // consume spaces
                while editor.cursor.col + 1 < len {
                    let ch = line.chars().nth(editor.cursor.col).unwrap();
                    if !ch.is_whitespace() { break; }
                    editor.cursor.col += 1;
                }
                break;
            } else if editor.cursor.row + 1 < editor.buffer.lines.len() {
                editor.cursor.row += 1;
                editor.cursor.col = 0;
                // skip leading spaces of next line
                if let Some(nl) = editor.buffer.lines.get(editor.cursor.row) {
                    while editor.cursor.col + 1 < nl.chars().count() {
                        let ch = nl.chars().nth(editor.cursor.col).unwrap();
                        if !ch.is_whitespace() { break; }
                        editor.cursor.col += 1;
                    }
                }
                break;
            } else { break; }
        }
        editor.ensure_cursor_visible();
        editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct BackwardWord;
impl Command for BackwardWord {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.sync_new_from_old();
        // Move back at least one char if possible
        if editor.cursor.col > 0 { editor.cursor.col -= 1; }
        else if editor.cursor.row > 0 { editor.cursor.row -= 1; if let Some(l) = editor.buffer.lines.get(editor.cursor.row) { editor.cursor.col = l.chars().count().saturating_sub(1); } }
        loop {
            if let Some(line) = editor.buffer.lines.get(editor.cursor.row) {
                if editor.cursor.col == 0 { break; }
                // skip spaces leftwards
                while editor.cursor.col > 0 {
                    let ch = line.chars().nth(editor.cursor.col).unwrap();
                    if !ch.is_whitespace() { break; }
                    editor.cursor.col -= 1;
                }
                // skip non-spaces leftwards
                while editor.cursor.col > 0 {
                    let ch = line.chars().nth(editor.cursor.col).unwrap();
                    if ch.is_whitespace() { editor.cursor.col += 1; break; }
                    editor.cursor.col -= 1;
                }
                break;
            } else { break; }
        }
        editor.ensure_cursor_visible();
        editor.sync_old_from_new();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
