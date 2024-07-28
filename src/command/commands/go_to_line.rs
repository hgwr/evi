use std::any::Any;

use crate::command::base::Command;
use crate::command::commands::move_cursor::NextLine;
use crate::data::{LineAddressType, LineRange, SimpleLineAddressType};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub struct GoToLineCommand {
    pub line_address: LineAddressType
}

impl Command for GoToLineCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        log::info!("GoToLineCommand execute");
        let line_number: isize = match self.line_address {
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::LineNumber(n)) => n as isize,
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::CurrentLine) => {
                editor.cursor_position_in_buffer.row as isize
            },
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::FirstLine) => 0,
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::LastLine) => {
                editor.buffer.lines.len() as isize
            },
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::AllLines) => {
                editor.buffer.lines.len() as isize
            },
            crate::data::LineAddressType::Absolute(SimpleLineAddressType::Pattern(_)) => {
                // TODO: Implement
                unimplemented!()
            },
            crate::data::LineAddressType::Relative(SimpleLineAddressType::FirstLine, i) => {
                0 + i
            },
            crate::data::LineAddressType::Relative(SimpleLineAddressType::LineNumber(n), i) => {
                (n as isize) + i
            },
            crate::data::LineAddressType::Relative(SimpleLineAddressType::CurrentLine, i) => {
                (editor.cursor_position_in_buffer.row as isize) + i
            },
            crate::data::LineAddressType::Relative(SimpleLineAddressType::LastLine, i) => {
                (editor.buffer.lines.len() as isize) + i
            },
            crate::data::LineAddressType::Relative(SimpleLineAddressType::AllLines, i) => {
                (editor.buffer.lines.len() as isize) + i
            },
            crate::data::LineAddressType::Relative(SimpleLineAddressType::Pattern(_), i) => {
                // TODO: Implement
                unimplemented!()
            },
        };

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
