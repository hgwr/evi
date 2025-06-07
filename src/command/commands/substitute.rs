use std::any::Any;

use regex::RegexBuilder;

use crate::command::base::Command;
use crate::data::LineRange;
use crate::editor::Editor;
use crate::generic_error::{GenericError, GenericResult};

#[derive(Clone)]
pub struct SubstituteCommand {
    pub line_range: LineRange,
    pub pattern: String,
    pub replacement: String,
    pub global: bool,
    pub ignore_case: bool,
    pub original_lines: Vec<String>,
}

impl Command for SubstituteCommand {
    fn is_undoable(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let start = editor.get_line_number_from(&self.line_range.start);
        let end = editor.get_line_number_from(&self.line_range.end);

        let re = RegexBuilder::new(&self.pattern)
            .multi_line(true)
            .case_insensitive(self.ignore_case)
            .build()
            .map_err(|e| GenericError::from(e.to_string()))?;

        self.original_lines.clear();
        for i in start..=end {
            if i >= editor.buffer.lines.len() {
                continue;
            }
            let line = editor.buffer.lines[i].clone();
            self.original_lines.push(line.clone());
            let new_line = if self.global {
                re.replace_all(&line, self.replacement.as_str()).to_string()
            } else {
                re.replace(&line, self.replacement.as_str()).to_string()
            };
            editor.buffer.lines[i] = new_line;
        }
        Ok(())
    }

    fn undo(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let start = editor.get_line_number_from(&self.line_range.start);
        for (offset, line) in self.original_lines.iter().enumerate() {
            let idx = start + offset;
            if idx < editor.buffer.lines.len() {
                editor.buffer.lines[idx] = line.clone();
            }
        }
        Ok(())
    }

    fn redo(&mut self, editor: &mut Editor) -> GenericResult<Option<Box<dyn Command>>> {
        let mut new_cmd = Box::new(SubstituteCommand {
            line_range: self.line_range.clone(),
            pattern: self.pattern.clone(),
            replacement: self.replacement.clone(),
            global: self.global,
            ignore_case: self.ignore_case,
            original_lines: Vec::new(),
        });
        new_cmd.execute(editor)?;
        Ok(Some(new_cmd))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
