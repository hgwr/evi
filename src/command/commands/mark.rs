use std::any::Any;
use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct SetMark;

impl Command for SetMark {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.set_mark_mode();
        Ok(())
    }
    fn is_reusable(&self) -> bool { false }
    fn is_modeful(&self) -> bool { true }
    fn as_any(&self) -> &dyn Any { self }
}

#[derive(Clone)]
pub struct JumpMark;

impl Command for JumpMark {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.set_jump_mark_mode();
        Ok(())
    }
    fn is_reusable(&self) -> bool { false }
    fn is_modeful(&self) -> bool { true }
    fn as_any(&self) -> &dyn Any { self }
}

