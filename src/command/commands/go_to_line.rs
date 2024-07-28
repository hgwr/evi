use std::any::Any;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct GoToLineCommand {
    pub line_address: LineAddressType
}

impl Command for GoToLineCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        log::info!("GoToLineCommand execute");
        editor.go_to_line(&self.line_address)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
