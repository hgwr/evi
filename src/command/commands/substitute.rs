use std::any::Any;

use regex::RegexBuilder;

use crate::command::base::Command;
use crate::data::LineRange;
use crate::editor::Editor;
use crate::generic_error::{GenericError, GenericResult};

pub struct SubstituteCommand {
    pub line_range: LineRange,
    pub pattern: String,
    pub replacement: String,
    pub global: bool,
    pub ignore_case: bool,
}

impl Command for SubstituteCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let start = editor.get_line_number_from(&self.line_range.start);
        let end = editor.get_line_number_from(&self.line_range.end);

        let re = RegexBuilder::new(&self.pattern)
            .multi_line(true)
            .case_insensitive(self.ignore_case)
            .build()
            .map_err(|e| GenericError::from(e.to_string()))?;

        for i in start..=end {
            if i >= editor.buffer.lines.len() {
                continue;
            }
            let line = editor.buffer.lines[i].clone();
            let new_line = if self.global {
                re.replace_all(&line, self.replacement.as_str()).to_string()
            } else {
                re.replace(&line, self.replacement.as_str()).to_string()
            };
            editor.buffer.lines[i] = new_line;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

