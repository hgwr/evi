use std::any::Any;

use regex::Regex;

use crate::command::base::Command;
use crate::data::{LineAddressType, LineRange, SimpleLineAddressType, Token};
use crate::editor::Editor;
use crate::ex::parser::Parser;
use crate::generic_error::{GenericError, GenericResult};

pub struct GlobalCommand {
    pub line_range: LineRange,
    pub pattern: String,
    pub command_tokens: Option<Vec<Token>>,
    pub invert: bool,
}

impl Command for GlobalCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let re = Regex::new(&self.pattern).map_err(|e| GenericError::from(e.to_string()))?;
        let start = editor.get_line_number_from(&self.line_range.start);
        let end = editor.get_line_number_from(&self.line_range.end);

        let mut matched_indices = Vec::new();
        for i in start..=end {
            if i >= editor.buffer.lines.len() {
                continue;
            }
            let line = &editor.buffer.lines[i];
            let is_match = re.is_match(line);
            if (!self.invert && is_match) || (self.invert && !is_match) {
                matched_indices.push(i);
            }
        }

        let mut offset: isize = 0;
        for idx in matched_indices {
            let current = (idx as isize + offset) as usize;
            if current >= editor.buffer.lines.len() {
                break;
            }
            editor.move_cursor_to(current, 0)?;
            let before_len = editor.buffer.lines.len();
            if let Some(tokens) = &self.command_tokens {
                let mut parser = Parser::from_tokens(tokens.clone());
                let mut cmd = parser.parse()?;
                cmd.execute(editor)?;
            } else {
                let mut print = crate::command::commands::print::PrintCommand {
                    line_range: LineRange {
                        start: LineAddressType::Absolute(SimpleLineAddressType::CurrentLine),
                        end: LineAddressType::Absolute(SimpleLineAddressType::CurrentLine),
                    },
                };
                print.execute(editor)?;
            }
            let after_len = editor.buffer.lines.len();
            offset += after_len as isize - before_len as isize;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
