use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct Esc;
impl Command for Esc {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.is_insert_mode() {
            editor.set_command_mode();
        } else {
            editor.set_command_mode();
            editor.display_visual_bell()?;
        }
        editor.status_line = "".to_string();
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
