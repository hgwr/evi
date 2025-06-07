use std::any::Any;

use crate::command::base::Command;
use crate::data::LineRange;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct PrintCommand {
    #[cfg_attr(not(test), allow(dead_code))]
    pub line_range: LineRange,
}

impl Command for PrintCommand {
    fn execute(&mut self, _editor: &mut Editor) -> GenericResult<()> {
        // TODO: Implement PrintCommand
        log::info!("PrintCommand execute");
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
