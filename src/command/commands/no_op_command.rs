use crate::command::base::Command;
use crate::editor::Editor;

pub struct NoOpCommand;

impl Command for NoOpCommand {
    fn execute(&mut self, _editor: &mut Editor) {
        // 何もしない
    }
}
