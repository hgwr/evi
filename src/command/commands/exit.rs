use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct ExitCommand;

impl Command for ExitCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.is_dirty {
            let result = editor.save_file();
            if let Err(e) = result {
                return Err(e);
            }
        }
        editor.should_exit = true;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
