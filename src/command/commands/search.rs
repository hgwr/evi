use std::any::Any;

use crate::command::base::Command;
use crate::editor::{Editor, SearchDirection};
use crate::generic_error::GenericResult;

pub struct RepeatSearch {
    pub same_direction: bool,
}

impl Command for RepeatSearch {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.repeat_search(self.same_direction)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
