use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct Undo;
impl Command for Undo {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.undo()
    }
}
