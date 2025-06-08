use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GoToFirstLine {
    pub count: usize,
}

impl Command for GoToFirstLine {
    fn is_reusable(&self) -> bool {
        false
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let target = if self.count == 0 { 0 } else { self.count - 1 };
        let max_row = editor.buffer.lines.len().saturating_sub(1);
        editor.move_cursor_to(target.min(max_row), 0)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GoToLastLine {
    pub count: usize,
}

impl Command for GoToLastLine {
    fn is_reusable(&self) -> bool {
        false
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let max_row = editor.buffer.lines.len().saturating_sub(1);
        let target = if self.count == 0 {
            max_row
        } else {
            self.count - 1
        };
        editor.move_cursor_to(target.min(max_row), 0)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
