use crate::command::base::{Command, CommandData, JumpCommandData};
use crate::command::commands::exit::ExitCommand;
use crate::command::commands::move_cursor::*;
use crate::command::commands::no_op_command::NoOpCommand;
use crossterm::event::KeyCode;

use super::commands::misc::DisplayFile;

pub fn command_factory(command_data: &CommandData) -> Box<dyn Command> {
    match command_data {
        CommandData {
            command: KeyCode::Char('j'),
            ..
        } | CommandData {
            command: KeyCode::Down,
            ..
        } => Box::new(NextLine {}),
        CommandData {
            command: KeyCode::Char('k'),
            ..
        } | CommandData {
            command: KeyCode::Up,
            ..
        } => Box::new(PreviousLine {}),
        CommandData {
            command: KeyCode::Char('l'),
            ..
        } | CommandData {
            command: KeyCode::Right,
            ..
        } => Box::new(ForwardChar {}),
        CommandData {
            command: KeyCode::Char('h'),
            ..
        } | CommandData {
            command: KeyCode::Left,
            ..
        } => Box::new(BackwardChar {}),
        CommandData {
            command: KeyCode::Char('0'),
            ..
        } => Box::new(MoveBeginningOfLine {}),
        CommandData {
            command: KeyCode::Char('$'),
            ..
        } => Box::new(MoveEndOfLine {}),

        // Control + g
        CommandData {
            command: KeyCode::Char('g'),
            modifiers,
            ..
        } if *modifiers == crossterm::event::KeyModifiers::CONTROL => {
            Box::new(DisplayFile {})
        },

        // ZZ
        CommandData {
            command: KeyCode::Char('Z'),
            ..
        } => {
            if let Some(JumpCommandData { count, command }) = command_data.range {
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
