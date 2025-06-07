use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Replace;

impl Command for Replace {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_modeful(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.set_replace_mode();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
