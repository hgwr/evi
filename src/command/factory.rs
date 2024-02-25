use crossterm::event::KeyCode;
use crate::command::base::{Command, CommandData, JumpCommandData};
use crate::command::commands::move_cursor::{DownOneLine, UpOneLine, RightOneChar, LeftOneChar};
use crate::command::commands::no_op_command::NoOpCommand;
use crate::command::commands::exit::ExitCommand;

pub fn command_factory(command_data: &CommandData) -> Box<dyn Command> {
  match command_data.command {
      KeyCode::Char('j') | KeyCode::Down => Box::new(DownOneLine {}),
      KeyCode::Char('k') | KeyCode::Up => Box::new(UpOneLine {}),
      KeyCode::Char('l') | KeyCode::Right => Box::new(RightOneChar {}),
      KeyCode::Char('h') | KeyCode::Left => Box::new(LeftOneChar {}),
      KeyCode::Char('Z') => {
        if let Some(JumpCommandData{count, command}) = command_data.range {
          if count == 1 && command == KeyCode::Char('Z') {
            Box::new(ExitCommand {})
          } else {
            Box::new(NoOpCommand {})
          }
        } else {
          Box::new(NoOpCommand {})
        }
      },
      _ => Box::new(NoOpCommand {}),
  }
}
