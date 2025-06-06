use std::any::Any;

use crate::command::base::{Command, JumpCommandData};
use crate::command::region::get_region;
use crate::editor::{Editor, EditorCursorData};
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Yank {
    pub jump_command_data_opt: Option<JumpCommandData>,
    pub editor_cursor_data: Option<EditorCursorData>,
}

impl Default for Yank {
    fn default() -> Self {
        Self {
            jump_command_data_opt: None,
            editor_cursor_data: None,
        }
    }
}

impl Command for Yank {
    fn is_reusable(&self) -> bool {
        false
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if let Some(jump_command_data) = self.jump_command_data_opt {
            let region = get_region(editor, jump_command_data)?;
            let start = region.start.cursor_position_in_buffer;
            let end = region.end.cursor_position_in_buffer;
            let text = editor.buffer.yank(start, end)?;
            editor.unnamed_register = text;
            editor.unnamed_register_linewise = start.col == 0 && end.col == 0;
            editor.restore_cursor_data(region.start);
            self.editor_cursor_data = Some(region.start);
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
