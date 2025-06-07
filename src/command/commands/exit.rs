use std::any::Any;

use crate::command::base::Command;
use crate::editor::Editor;
use crate::generic_error::GenericResult;

#[derive(Clone)]
pub struct ExitCommand;
impl Command for ExitCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        if editor.is_dirty {
            let result = editor.save_file();
            if let Err(e) = result {
                return Err(e);
            }
        }
        editor.should_exit = true;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct ExitWithSaveCommand;
impl Command for ExitWithSaveCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        let result = editor.save_file();
        if let Err(e) = result {
            return Err(e);
        }
        editor.should_exit = true;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct ExitWithoutSaveCommand;
impl Command for ExitWithoutSaveCommand {
    fn execute(&mut self, editor: &mut Editor) -> GenericResult<()> {
        editor.should_exit = true;
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
