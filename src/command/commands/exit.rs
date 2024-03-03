use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct ExitCommand;

impl Command for ExitCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.is_dirty {
            // TODO: display a message to ask if the user wants to save the file
        } else {
            editor.should_exit = true;
        }
        Ok(())
    }
}
