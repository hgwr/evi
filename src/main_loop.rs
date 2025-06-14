use crossterm::{
    event::{self, Event, KeyEvent, KeyModifiers},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::stdout;
use std::io::Write;

use log::error;

use crate::command::compose::{compose, InputState, KeyData};
use crate::editor::Editor;
use crate::generic_error::GenericResult;

pub fn main_loop(editor: &mut Editor) -> GenericResult<()> {
    let mut stdout = stdout();
    let mut event_keys: Vec<KeyEvent> = Vec::new();
    let mut awaiting_register = false;

    terminal::enable_raw_mode()?;

    let terminal_size = terminal::size()?;
    editor.resize_terminal(terminal_size.0, terminal_size.1);

    loop {
        editor.render(&mut stdout)?;
        let result = event::read();
        match result {
            Ok(Event::Key(key_event)) => {
                if editor.is_command_mode() {
                    if awaiting_register {
                        if let event::KeyCode::Char(c) = key_event.code {
                            editor.pending_register = Some(c);
                        } else {
                            editor.status_line = "?".to_string();
                        }
                        awaiting_register = false;
                        continue;
                    }
                    if event_keys.len() == 0 && key_event.code == event::KeyCode::Char('"') {
                        awaiting_register = true;
                        continue;
                    }
                    if event_keys.len() == 0 && key_event.code == event::KeyCode::Char(':') {
                        // ex command begin
                        editor.set_ex_command_mode();
                        editor.status_line = ":".to_string();
                    } else if event_keys.len() == 0 && key_event.code == event::KeyCode::Char('/') {
                        editor.set_search_mode(crate::editor::SearchDirection::Forward);
                    } else if event_keys.len() == 0 && key_event.code == event::KeyCode::Char('?') {
                        editor.set_search_mode(crate::editor::SearchDirection::Backward);
                    } else {
                        event_keys.push(key_event);
                        let input_state = compose(&event_keys);
                        match input_state {
                            InputState::CommandCompleted(command_data) => {
                                editor.execute_command(command_data)?;
                                event_keys.clear();
                            }
                            InputState::CommandInvalid(key_codes) => {
                                error!("Invalid command: {:?}", key_codes);
                                editor.status_line = "?".to_string();
                                editor.display_visual_bell()?;
                                event_keys.clear();
                            }
                            _ => {}
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
                        KeyData {
                            key_code: event::KeyCode::Backspace,
                            ..
                        }
                        | KeyData {
                            key_code: event::KeyCode::Char('h'),
                            modifiers: KeyModifiers::CONTROL,
                        } => {
                            editor.backspace_ex_command();
                        }
                        KeyData {
                            key_code: event::KeyCode::Tab,
                            ..
                        } => {
                            editor.complete_ex_command();
                        }
                        KeyData {
                            key_code: event::KeyCode::Left,
                            ..
                        } => {
                            editor.move_ex_command_cursor_left();
                        }
                        KeyData {
                            key_code: event::KeyCode::Right,
                            ..
                        } => {
                            editor.move_ex_command_cursor_right();
                        }
                        KeyData {
                            key_code: event::KeyCode::Up,
                            ..
                        } => {
                            editor.previous_ex_command();
                        }
                        KeyData {
                            key_code: event::KeyCode::Down,
                            ..
                        } => {
                            editor.next_ex_command();
                        }
                        KeyData {
                            key_code: event::KeyCode::Char(c),
                            ..
                        } => {
                            editor.insert_ex_command_char(c);
                        }
                        _ => {}
                    }
                } else if editor.is_search_mode() {
                    let key_data: KeyData = key_event.into();
                    match key_data {
                        KeyData {
                            key_code: event::KeyCode::Enter,
                            ..
                        } => {
                            editor.execute_search_query()?;
                        }
                        KeyData {
                            key_code: event::KeyCode::Esc,
                            ..
                        } => {
                            editor.set_command_mode();
                            editor.status_line = "".to_string();
                        }
                        _ => {
                            editor.append_search_query(key_data);
                        }
                    }
                } else if editor.is_find_char_mode() {
                    match key_event.code {
                        event::KeyCode::Esc => {
                            editor.set_command_mode();
                            editor.status_line = "".to_string();
                        }
                        event::KeyCode::Char(c) => {
                            editor.execute_find_char(c)?;
                        }
                        _ => {}
                    }
                } else if editor.is_set_mark_mode() {
                    match key_event.code {
                        event::KeyCode::Esc => {
                            editor.set_command_mode();
                            editor.status_line = "".to_string();
                        }
                        event::KeyCode::Char(c) => {
                            editor.set_mark(c);
                            editor.set_command_mode();
                        }
                        _ => {}
                    }
                } else if editor.is_jump_mark_mode() {
                    match key_event.code {
                        event::KeyCode::Esc => {
                            editor.set_command_mode();
                            editor.status_line = "".to_string();
                        }
                        event::KeyCode::Char(c) => {
                            editor.jump_to_mark(c)?;
                            editor.set_command_mode();
                        }
                        _ => {}
                    }
                } else if editor.is_replace_char_mode() {
                    match key_event.code {
                        event::KeyCode::Esc => {
                            editor.set_command_mode();
                            editor.status_line = "".to_string();
                        }
                        event::KeyCode::Char(c) => {
                            for i in 0..editor.pending_replace_char_count {
                                if i + 1 == editor.pending_replace_char_count {
                                    editor.replace_char_at_cursor(c)?;
                                } else {
                                    editor.replace_char_and_move(c)?;
                                }
                            }
                            editor.set_command_mode();
                        }
                        _ => {}
                    }
                } else if editor.is_replace_mode() {
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
                                editor.replace_char_and_move(c)?;
                            }
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
                // ignore other events
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
