use crossterm::event::KeyCode;

use crate::editor::Editor;

pub trait Command {
  fn execute(&mut self, editor: &mut Editor);
}

pub trait EditingCommand: Command {
  fn undo(&mut self, editor: &mut Editor);
  fn set_range(&mut self, range: JumpCommandData);
}

#[derive(Clone, Copy, Debug)]
pub struct JumpCommandData {
  pub count: usize,
  pub command: KeyCode,
}

#[derive(Clone, Copy, Debug)]
pub struct CommandData {
  pub count: usize,
  pub command: KeyCode,
  pub range: Option<JumpCommandData>
}

