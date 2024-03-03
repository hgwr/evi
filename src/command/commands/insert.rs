use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct Insert;
impl Command for Insert {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if (editor.mode == crate::editor::Mode::Insert) {
          // do nothing
        } else {
            editor.mode = crate::editor::Mode::Insert;
        }
        editor.status_line = "-- INSERT --".to_string();
        Ok(())
    }
}
