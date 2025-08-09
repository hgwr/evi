use crossterm::event::{KeyCode, KeyModifiers};

use crate::command::base::Command;
use crate::command::base::CommandData;
use crate::command::base::JumpCommandData;
use crate::command::factory::command_factory;
use crate::editor::Editor;
use crate::editor::Region;
use crate::generic_error::GenericResult;
use crate::command::commands::move_cursor::MoveBeginningOfLine;
use super::commands::move_cursor::MoveEndOfLine;
use super::commands::move_cursor::NextLine;
use super::key_codes::is_editing_command_with_range;

fn is_line_oriented_command(jump_command_data: JumpCommandData) -> bool {
    let key_code = jump_command_data.key_code;
    let modifiers = jump_command_data.modifiers;
    // If key_code is ‘j’, ‘k’, Ctrl-f, Ctrl-b, etc., the command is regarded as line-oriented.
    match key_code {
        KeyCode::Char('j') | KeyCode::Char('k') => true,
        KeyCode::Char('f') | KeyCode::Char('b') => {
            if modifiers == KeyModifiers::CONTROL {
                true
            } else {
                false
            }
        }
        _ => false,
    }
}

fn is_end_of_line_command(jump_command_data: JumpCommandData) -> bool {
    let key_code = jump_command_data.key_code;
    // If key_code is ‘$’, it is regarded as a command to move to the end of the line.
    match key_code {
        KeyCode::Char('$') => true,
        _ => false,
    }
}

pub fn get_region(editor: &mut Editor, jump_command_data: JumpCommandData) -> GenericResult<Region> {
    if is_line_oriented_command(jump_command_data) {
        get_region_from_line_oriented_command(editor, jump_command_data)
    } else if is_end_of_line_command(jump_command_data) {
        get_region_from_end_of_line_command(editor)
    } else if is_editing_command_with_range(&jump_command_data.key_code) {
        get_region_on_this_line(editor)
    } else {
        get_region_from_command(editor, jump_command_data)
    }
}

fn get_region_from_line_oriented_command(editor: &mut Editor, jump_command_data: JumpCommandData) -> GenericResult<Region> {
    // migrate to new system
    editor.sync_new_from_old();
    // move to beginning of line
    let mut move_beginning_of_line = MoveBeginningOfLine;
    move_beginning_of_line.execute(editor)?; // command already syncs
    editor.sync_new_from_old();
    let start = editor.snapshot_new_cursor();
    let command_data: CommandData = jump_command_data.into();
    for _ in 0..command_data.count {
        let mut jump_command = command_factory(&command_data);
        jump_command.execute(editor)?;
    }
    editor.sync_new_from_old();
    // if moved downward, extend to whole next line start
    if editor.cursor.row > start.cursor.row {
        let mut next_line = NextLine;
        next_line.execute(editor)?;
    }
    // move to beginning of (current) line to mark end
    let mut move_beginning_of_line = MoveBeginningOfLine; // reuse name
    move_beginning_of_line.execute(editor)?;
    editor.sync_new_from_old();
    let end = editor.snapshot_new_cursor();
    // restore for non-destructive behaviour? callers expect cursor at beginning of region? Original code leaves at beginning (after second move_beginning_of_line)
    // Already at beginning -> fine.
    editor.sync_old_from_new();
    Ok(Region { start, end })
}

fn get_region_from_end_of_line_command(editor: &mut Editor) -> GenericResult<Region> {
    editor.sync_new_from_old();
    let start = editor.snapshot_new_cursor();
    let mut move_end_of_line = MoveEndOfLine;
    move_end_of_line.execute(editor)?;
    editor.sync_new_from_old();
    // ensure end.col points to line end (like inclusive)
    if let Some(line) = editor.buffer.lines.get(editor.cursor.row) {
    let len = line.chars().count();
    editor.cursor.col = if len == 0 { 0 } else { len - 1 };
    }
    let end = editor.snapshot_new_cursor();
    editor.sync_old_from_new();
    Ok(Region { start, end })
}

fn get_region_on_this_line(editor: &mut Editor) -> GenericResult<Region> {
    editor.sync_new_from_old();
    let mut move_beginning_of_line = MoveBeginningOfLine;
    move_beginning_of_line.execute(editor)?;
    editor.sync_new_from_old();
    let start = editor.snapshot_new_cursor();
    let mut next_line = NextLine;
    next_line.execute(editor)?;
    let mut move_beginning_of_line = MoveBeginningOfLine;
    move_beginning_of_line.execute(editor)?;
    editor.sync_new_from_old();
    let end = editor.snapshot_new_cursor();
    editor.sync_old_from_new();
    Ok(Region { start, end })
}

fn get_region_from_command(editor: &mut Editor, jump_command_data: JumpCommandData) -> GenericResult<Region> {
    editor.sync_new_from_old();
    let start = editor.snapshot_new_cursor();
    let command_data: CommandData = jump_command_data.into();
    for _ in 0..command_data.count {
        let mut jump_command = command_factory(&command_data);
        jump_command.execute(editor)?;
    }
    editor.sync_new_from_old();
    let end = editor.snapshot_new_cursor();
    editor.sync_old_from_new();
    Ok(Region { start, end })
}
