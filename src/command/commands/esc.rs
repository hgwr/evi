use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct Esc;
impl Command for Esc {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.mode == crate::editor::Mode::Insert {
            editor.mode = crate::editor::Mode::Command;
        } else {
            editor.mode = crate::editor::Mode::Command;
            editor.display_visual_bell()?;
        }
        editor.status_line = "".to_string();
        Ok(())
    }
}
