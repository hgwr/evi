use crate::command::base::{Command, CommandData, JumpCommandData};
use crate::command::commands::exit::ExitCommand;
use crate::command::commands::move_cursor::*;
use crate::command::commands::no_op_command::NoOpCommand;
use crossterm::event::KeyCode;

use super::commands::append::Append;
use super::commands::delete::{Delete, DeleteChar};
use super::commands::insert::Insert;
use super::commands::misc::DisplayFile;
use super::commands::undo::Undo;

pub fn command_factory(command_data: &CommandData) -> Box<dyn Command> {
    match command_data {
        CommandData {
            key_code: KeyCode::Char('j'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Down,
            ..
        } => Box::new(NextLine {}),
        CommandData {
            key_code: KeyCode::Char('k'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Up,
            ..
        } => Box::new(PreviousLine {}),
        CommandData {
            key_code: KeyCode::Char('l'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Right,
            ..
        } => Box::new(ForwardChar {}),
        CommandData {
            key_code: KeyCode::Char('h'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Left,
            ..
        } => Box::new(BackwardChar {}),
        CommandData {
            key_code: KeyCode::Char('0'),
            ..
        } => Box::new(MoveBeginningOfLine {}),
        CommandData {
            key_code: KeyCode::Char('$'),
            ..
        } => Box::new(MoveEndOfLine {}),

        // jump commands
        CommandData {
            key_code: KeyCode::Char('w'),
            ..
        } => Box::new(ForwardWord {}),
        CommandData {
            key_code: KeyCode::Char('b'),
            ..
        } => Box::new(BackwardWord {}),

        // insert commands
        CommandData {
            key_code: KeyCode::Char('i'),
            ..
        } => Box::new(Insert::default()),

        // append commands
        CommandData {
            key_code: KeyCode::Char('a'),
            ..
        } => Box::new(Append::default()),

        // delete commands
        CommandData {
            key_code: KeyCode::Char('x'),
            ..
        } => Box::new(DeleteChar::default()),

        CommandData {
            key_code: KeyCode::Char('d'),
            range,
            ..
        } => Box::new(Delete {
            jump_command_data_opt: range.clone(),
            ..Default::default()
        }),

        // undo command
        CommandData {
            key_code: KeyCode::Char('u'),
            ..
        } => Box::new(Undo {}),

        // Control + g
        CommandData {
            key_code: KeyCode::Char('g'),
            modifiers,
            ..
        } if *modifiers == crossterm::event::KeyModifiers::CONTROL => Box::new(DisplayFile {}),

        // ZZ
        CommandData {
            key_code: KeyCode::Char('Z'),
            ..
        } => {
            if let Some(JumpCommandData {
                count,
                key_code: command,
                ..
            }) = command_data.range
            {
                if count == 1 && command == KeyCode::Char('Z') {
                    Box::new(ExitCommand {})
                } else {
                    Box::new(NoOpCommand {})
                }
            } else {
                Box::new(NoOpCommand {})
            }
        }
        _ => Box::new(NoOpCommand {}),
    }
}
