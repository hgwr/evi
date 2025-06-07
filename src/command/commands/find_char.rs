use std::any::Any;

use crate::command::base::Command;
use crate::editor::{Editor, SearchDirection};
use crate::generic_error::GenericResult;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FindChar {
    pub count: usize,
    pub direction: SearchDirection,
    pub inclusive: bool,
}

impl Command for FindChar {
    fn is_reusable(&self) -> bool {
        false
    }

    fn is_modeful(&self) -> bool {
        true
    }

    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.set_find_char_mode(self.direction, self.inclusive, self.count);
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct RepeatFindChar;

impl Command for RepeatFindChar {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.repeat_find_char()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
