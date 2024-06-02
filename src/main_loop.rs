use crossterm::{
    event::{self, Event, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::stdout;
use std::io::Write;

use log::{error, info};

use crate::command::compose::{compose, InputState, KeyData};
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
                if editor.is_command_mode() {
                    info!("Key event: {:?}", key_event);
                    if event_keys.len() == 0 && key_event.code == event::KeyCode::Char(':') {
                        // ex command begin
                        editor.set_ex_command_mode();
                        editor.status_line = ":".to_string();
                    } else {
                        event_keys.push(key_event);
                        let input_state = compose(&event_keys);
                        match input_state {
                            InputState::CommandCompleted(command_data) => {
                                info!("Command completed: {:?}", command_data);
                                editor.execute_command(command_data)?;
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
                } else if editor.is_ex_command_mode() {
                    let key_data: KeyData = key_event.into();
                    match key_data {
                        KeyData {
                            key_code: event::KeyCode::Enter,
                            ..
                        } => {
                            let command_data = editor.get_ex_command_data();
                            editor.execute_ex_command(command_data)?;
                            editor.set_command_mode();
                        }
                        KeyData {
                            key_code: event::KeyCode::Esc,
                            ..
                        } => {
                            editor.set_command_mode();
                            editor.status_line = "".to_string();
                        }
                        _ => {
                            editor.append_ex_command(key_data);
                        }
                    }
                } else if editor.is_insert_mode() {
                    let key_data: KeyData = key_event.into();
                    match key_data {
                        KeyData {
                            key_code: event::KeyCode::Enter,
                            ..
                        } => {
                            editor.append_new_line()?;
                        }
                        KeyData {
                            key_code: event::KeyCode::Esc,
                            ..
                        } => {
                            editor.set_command_mode();
                            editor.status_line = "".to_string();
                        }
                        KeyData {
                            key_code: event::KeyCode::Backspace,
                            ..
                        }
                        | KeyData {
                            key_code: event::KeyCode::Char('h'),
                            modifiers: KeyModifiers::CONTROL,
                        } => {
                            editor.backward_delete_char()?;
                        }
                        KeyData {
                            key_code: event::KeyCode::Char('l'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        } => {
                            editor.render(&mut stdout)?;
                        }
                        _ => {
                            if let crossterm::event::KeyCode::Char(c) = key_event.code {
                                editor.insert_char(c)?;
                            }
                        }
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
