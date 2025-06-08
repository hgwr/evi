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
        let current_col = editor.cursor_position_in_buffer.col;
        let max_row = editor.buffer.lines.len().saturating_sub(1);
        let target = if self.count == 0 { 0 } else { self.count - 1 };
        let target = target.min(max_row);
        let dest_col = editor
            .buffer
            .lines
            .get(target)
            .map(|line| line.chars().count().min(current_col))
            .unwrap_or(0);
        editor.move_cursor_to(target, dest_col)?;
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
        let current_col = editor.cursor_position_in_buffer.col;
        let max_row = editor.buffer.lines.len().saturating_sub(1);
        let target = if self.count == 0 {
            max_row
        } else {
            self.count - 1
        };
        let target = target.min(max_row);
        let dest_col = editor
            .buffer
            .lines
            .get(target)
            .map(|line| line.chars().count().min(current_col))
            .unwrap_or(0);
        editor.move_cursor_to(target, dest_col)?;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
