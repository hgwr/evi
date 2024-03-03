use crossterm::event::{KeyCode, KeyModifiers};

use crate::{editor::Editor, generic_error::GenericResult};

pub trait Command {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()>;
}

pub trait EditingCommand: Command {
    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()>;
    fn set_range(&mut self, range: JumpCommandData);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JumpCommandData {
    pub count: usize,
    pub command: KeyCode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CommandData {
    pub count: usize,
    pub command: KeyCode,
    pub modifiers: KeyModifiers,
    pub range: Option<JumpCommandData>,
}
