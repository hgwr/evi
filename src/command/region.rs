use crossterm::event::{KeyCode, KeyModifiers};

use crate::command::base::Command;
use crate::command::base::CommandData;
use crate::command::base::JumpCommandData;
use crate::command::factory::command_factory;
use crate::editor::Editor;
use crate::editor::Region;
use crate::generic_error::GenericResult;
use crate::command::commands::move_cursor::MoveBeginningOfLine;
use super::commands::move_cursor::NextLine;

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
    } else {
        get_region_from_command(editor, jump_command_data)
    }
}

fn get_region_from_line_oriented_command(editor: &mut Editor, jump_command_data: JumpCommandData) -> GenericResult<Region> {
    let mut move_beginning_of_line = MoveBeginningOfLine;
    move_beginning_of_line.execute(editor)?;
    let start_cursor_data = editor.snapshot_cursor_data();
    let command_data: CommandData = jump_command_data.into();
    for _ in 0..command_data.count {
        let mut jump_command = command_factory(&command_data);
        jump_command.execute(editor)?;
    }
    let current_cursor_data = editor.snapshot_cursor_data();
    if start_cursor_data.cursor_position_in_buffer < current_cursor_data.cursor_position_in_buffer {
        let mut next_line = NextLine;
        next_line.execute(editor)?;
    }
    move_beginning_of_line.execute(editor)?;
    let end_cursor_data = editor.snapshot_cursor_data();
    Ok(Region {
        start: start_cursor_data,
        end: end_cursor_data,
    })
}

fn get_region_from_end_of_line_command(editor: &mut Editor) -> GenericResult<Region> {
    let start_cursor_data = editor.snapshot_cursor_data();
    let mut next_line = NextLine;
    next_line.execute(editor)?;
    let mut move_beginning_of_line = MoveBeginningOfLine;
    move_beginning_of_line.execute(editor)?;
    let end_cursor_data = editor.snapshot_cursor_data();
    Ok(Region {
        start: start_cursor_data,
        end: end_cursor_data,
    })
}

fn get_region_from_command(editor: &mut Editor, jump_command_data: JumpCommandData) -> GenericResult<Region> {
    let start_cursor_data = editor.snapshot_cursor_data();
    let command_data: CommandData = jump_command_data.into();
    for _ in 0..command_data.count {
        let mut jump_command = command_factory(&command_data);
        jump_command.execute(editor)?;
    }
    let end_cursor_data = editor.snapshot_cursor_data();
    Ok(Region {
        start: start_cursor_data,
        end: end_cursor_data,
    })
}
