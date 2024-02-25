use crossterm::{
    event::{self, Event},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::stdout;

use crate::editor::Editor;
use crate::command::compose::{InputState, compose};

pub fn main_loop(editor: &mut Editor) {
    let mut stdout = stdout();
    let mut input_keys = Vec::new();

    terminal::enable_raw_mode();

    loop {
        editor.render(&mut stdout);
        if let Ok(Event::Key(key_event)) = event::read() {
            input_keys.push(key_event.code);
            let input_state = compose(&input_keys);
            match input_state {
                InputState::CommandCompleted(command_data) => {
                    editor.execute_command(command_data);
                    input_keys.clear();
                },
                InputState::CommandInvalid(key_codes) => {
                    //　TODO: error message
                    input_keys.clear();
                },
                _ => {}
            }
        } else if let Ok(Event::Resize(width, height)) = event::read() {
            editor.resize_terminal(width, height);
        }
    }

    terminal::disable_raw_mode();
}