use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct Undo;
impl Command for Undo {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.undo()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
