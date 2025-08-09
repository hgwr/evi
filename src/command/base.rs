use std::any::Any;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::{editor::Editor, generic_error::GenericResult};

pub trait Command {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()>;

    fn is_reusable(&self) -> bool {
        true
    }

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

    fn as_any(&self) -> &dyn Any;
}

impl dyn Command {
    pub fn is<T: Command + 'static>(&self) -> bool {
        self.as_any().is::<T>()
    }

    pub fn downcast_ref<T: Command + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
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

impl Into<CommandData> for JumpCommandData {
    fn into(self) -> CommandData {
        CommandData {
            count: self.count,
            key_code: self.key_code,
            modifiers: self.modifiers,
            range: None,
        }
    }
}
