use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ReplaceChar {
    pub count: usize,
}

impl Default for ReplaceChar {
    fn default() -> Self {
        Self { count: 1 }
    }
}

impl Command for ReplaceChar {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_modeful(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.set_replace_char_mode_with_count(self.count);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
