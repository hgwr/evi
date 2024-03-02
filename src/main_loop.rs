use crossterm::{
    event::{self, Event, KeyEvent},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::{error::Error, io::stdout};
use std::io::Write;

use log::{error, info, warn};

use crate::command::compose::{compose, InputState};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub fn main_loop(editor: &mut Editor) -> GenericResult<()> {
    let mut stdout = stdout();
    let mut event_keys: Vec<KeyEvent> = Vec::new();

    terminal::enable_raw_mode()?;

    let terminal_size = terminal::size()?;
    editor.resize_terminal(terminal_size.0, terminal_size.1);

    loop {
        editor.render(&mut stdout)?;
        let result = event::read();
        match result {
            Ok(Event::Key(key_event)) => {
                info!("Key event: {:?}", key_event);
                event_keys.push(key_event);
                let input_state = compose(&event_keys);
                match input_state {
                    InputState::CommandCompleted(command_data) => {
                        info!("Command completed: {:?}", command_data);
                        editor.execute_command(command_data);
                        event_keys.clear();
                    }
                    InputState::CommandInvalid(key_codes) => {
                        //　TODO: error message
                        error!("Invalid command: {:?}", key_codes);
                        event_keys.clear();
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

    terminal::disable_raw_mode()?;
    stdout.execute(terminal::Clear(ClearType::All))?;
    stdout.execute(terminal::LeaveAlternateScreen)?;
    stdout.flush()?;

    Ok(())
}
