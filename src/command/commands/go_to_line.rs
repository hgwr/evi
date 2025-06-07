use std::any::Any;

use crate::command::base::Command;
use crate::command::commands::move_cursor::NextLine;
use crate::data::LineAddressType;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct GoToLineCommand {
    pub line_address: LineAddressType,
}

impl Command for GoToLineCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        log::info!("GoToLineCommand execute");
        let line_number = editor.get_line_number_from(&self.line_address);

        log::info!("line_number: {}", line_number);
        editor.cursor_position_in_buffer.row = 0;
        editor.cursor_position_in_buffer.col = 0;
        editor.cursor_position_on_screen.row = 0;
        editor.cursor_position_on_screen.col = 0;
        editor.window_position_in_buffer.row = 0;
        editor.window_position_in_buffer.col = 0;
        if line_number > 0 {
            let mut next_line = NextLine {};
            for _ in 0..line_number {
                next_line.execute(editor)?;
            }
        }

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
