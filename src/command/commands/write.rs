use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Default, Clone)]
pub struct WriteCommand {
    #[cfg_attr(not(test), allow(dead_code))]
    pub force: bool,
}

impl Command for WriteCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let result = editor.save_file();
        if let Err(e) = result {
            editor.status_line = e.to_string();
            return Ok(());
        }
        if let Some(name) = editor.current_file_name() {
            editor.status_line = format!("\"{}\" written", name);
        } else {
            editor.status_line = "written".to_string();
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
