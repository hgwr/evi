use crate::command::base::Command;
use crate::editor::Editor;

pub struct ExitCommand;

impl Command for ExitCommand {
    fn execute(&mut self, editor: &mut Editor) {
        if editor.is_dirty {
            // TODO: display a message to ask if the user wants to save the file
        } else {
            editor.should_exit = true;
        }
    }
}
