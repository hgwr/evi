use crossterm::event::{KeyCode, KeyModifiers};

use crate::{editor::Editor, generic_error::GenericResult};

pub trait Command {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()>;
    fn is_modeful(&self) -> bool {
        false
    }
    fn is_undoable(&self) -> bool {
        false
    }
    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let _ = editor;
        // do nothing
        Ok(())
    }
    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        let _ = editor;
        // do nothing
        Ok(None)
    }
    fn set_text(&mut self, _text: String) {
        // do nothing
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct JumpCommandData {
    pub count: usize,
    pub key_code: KeyCode,
    pub modifiers: KeyModifiers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CommandData {
    pub count: usize,
    pub key_code: KeyCode,
    pub modifiers: KeyModifiers,
    pub range: Option<JumpCommandData>,
}

pub struct ExecutedCommand {
    pub command_data: CommandData,
    pub command: Box<dyn Command>,
}
