use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct Repeat;

impl Command for Repeat {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.repeat_last_command()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
