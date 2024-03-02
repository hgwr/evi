use crossterm::{
    event::{self, Event},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::stdout;
use std::io::Write;

use log::{error, info, warn};

use crate::command::compose::{compose, InputState};
use crate::editor::Editor;

pub fn main_loop(editor: &mut Editor) {
    let mut stdout = stdout();
    let mut input_keys = Vec::new();

    terminal::enable_raw_mode().unwrap();

    let terminal_size = terminal::size().unwrap();
    editor.resize_terminal(terminal_size.0, terminal_size.1);

    loop {
        editor.render(&mut stdout);
        let result = event::read();
        match result {
            Ok(Event::Key(key_event)) => {
                info!("Key event: {:?}", key_event);
                input_keys.push(key_event.code);
                let input_state = compose(&input_keys);
                match input_state {
                    InputState::CommandCompleted(command_data) => {
                        info!("Command completed: {:?}", command_data);
                        editor.execute_command(command_data);
                        input_keys.clear();
                    }
                    InputState::CommandInvalid(key_codes) => {
                        //　TODO: error message
                        error!("Invalid command: {:?}", key_codes);
                        input_keys.clear();
                    }
                    _ => {
                        info!("Input state: {:?}", input_state);
                    }
                }
            }
            Ok(Event::Resize(width, height)) => {
                editor.resize_terminal(width, height);
            }
            _ => {
                info!("Other event: {:?}", result)
            }
        }
        if editor.should_exit {
            break;
        }
    }

    terminal::disable_raw_mode().unwrap();
    stdout.execute(terminal::Clear(ClearType::All)).unwrap();
    stdout.execute(terminal::LeaveAlternateScreen).unwrap();
    stdout.flush().unwrap();
}
