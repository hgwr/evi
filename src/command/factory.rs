use crate::command::base::{Command, CommandData, JumpCommandData};
use crate::command::commands::exit::ExitCommand;
use crate::command::commands::move_cursor::{NextLine, BackwardChar, ForwardChar, PreviousLine};
use crate::command::commands::no_op_command::NoOpCommand;
use crossterm::event::KeyCode;

pub fn command_factory(command_data: &CommandData) -> Box<dyn Command> {
    match command_data.command {
        KeyCode::Char('j') | KeyCode::Down => Box::new(NextLine {}),
        KeyCode::Char('k') | KeyCode::Up => Box::new(PreviousLine {}),
        KeyCode::Char('l') | KeyCode::Right => Box::new(ForwardChar {}),
        KeyCode::Char('h') | KeyCode::Left => Box::new(BackwardChar {}),
        KeyCode::Char('Z') => {
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
