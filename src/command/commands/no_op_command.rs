use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct NoOpCommand;

impl Command for NoOpCommand {
    fn execute(&mut self, _editor: &mut Editor) -> GenericResult<()> {
        // 何もしない
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
