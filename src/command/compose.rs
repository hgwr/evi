use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use log::info;

use crate::command::base::{CommandData, JumpCommandData};
use crate::command::key_codes::{
    is_ctrl_command, is_editing_command_with_range, is_editing_command_without_range,
    is_jump_command,
};

// list of command patterns
//
// - immediate movement commands (l, h, j, k, etc.)
// - immediate editing commands (i, x, d, s, etc.)
// - movement commands with repeat specifications (3l, 4h, 5j, 6k, etc.)
// - Edit command with repeat specification (4x, 3i[str], etc.)
// - Edit commands with ranges (d3w, c4e, 4dl, etc.)

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyData {
    key_code: KeyCode,
    modifiers: KeyModifiers,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InputState {
    Start,

    AccumulateDigits(String),                          // Entering numbers
    CommandComposing(KeyData),                         // 'd', 'c' etc.
    CommandAndDigits(KeyData, String),                 // 'd3', 'c4' etc.
    DigitsAndCommand(usize, KeyData),                  // '3d', '4c' etc.
    DigitsAndCommandAndDigits(usize, KeyData, String), // '3d4', '4c3' etc.

    CommandCompleted(CommandData),
    CommandInvalid(String),
}

// Take vi command input, interpret it, and generate commands
pub fn compose(key_events: &Vec<KeyEvent>) -> InputState {
    info!("compose: {:?}", key_events);

    let mut input_state = InputState::Start;

    for event in key_events {
        match event {
            KeyEvent {
                code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('['),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                info!("Esc");
                return InputState::CommandCompleted(CommandData {
                    count: 1,
                    key_code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                    range: None,
                });
            }
            KeyEvent {
                code: KeyCode::Char(c),
                modifiers: KeyModifiers::NONE,
                ..
            } if c.is_digit(10) => {
                if let InputState::Start = input_state {
                    // 1st digit
                    if c == &'0' {
                        return InputState::CommandCompleted(CommandData {
                            count: 1,
                            key_code: KeyCode::Char('0'),
                            modifiers: KeyModifiers::NONE,
                            range: None,
                        });
                    } else {
                        input_state = InputState::AccumulateDigits(c.to_string());
                    }
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    // 2nd or later digit
                    input_state = InputState::AccumulateDigits(format!("{}{}", digits, c));
                } else if let InputState::CommandComposing(command) = input_state {
                    // 1st digit after command
                    input_state = InputState::CommandAndDigits(command, c.to_string());
                } else if let InputState::CommandAndDigits(command, digits) = input_state {
                    // 2nd or later digit after command
                    input_state = InputState::CommandAndDigits(command, format!("{}{}", digits, c));
                } else if let InputState::DigitsAndCommand(count, command) = input_state {
                    // When a number follows a cursor movement command that specifies the number of times
                    input_state =
                        InputState::DigitsAndCommandAndDigits(count, command, c.to_string());
                } else if let InputState::DigitsAndCommandAndDigits(count, command, digits) =
                    input_state
                {
                    // When a number follows a cursor movement command that specifies the number of times
                    input_state = InputState::DigitsAndCommandAndDigits(
                        count,
                        command,
                        format!("{}{}", digits, c),
                    );
                } else {
                    return InputState::CommandInvalid(format!("Invalid command: {:?}", event));
                }
            }
            KeyEvent {
                code, modifiers, ..
            } if (*modifiers == KeyModifiers::NONE || *modifiers == KeyModifiers::SHIFT)
                && is_jump_command(code) =>
            {
                if let InputState::Start = input_state {
                    return InputState::CommandCompleted(CommandData {
                        count: 1,
                        key_code: *code,
                        modifiers: *modifiers,
                        range: None,
                    });
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    return InputState::CommandCompleted(CommandData {
                        count,
                        key_code: *code,
                        modifiers: *modifiers,
                        range: None,
                    });
                } else if let InputState::CommandComposing(composing) = input_state {
                    let range = Some(JumpCommandData {
                        count: 1,
                        key_code: *code,
                        modifiers: *modifiers,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count: 1,
                        key_code: composing.key_code,
                        modifiers: composing.modifiers,
                        range,
                    });
                } else if let InputState::CommandAndDigits(composing, digits) = input_state {
                    let range = Some(JumpCommandData {
                        count: digits.parse().unwrap(),
                        key_code: *code,
                        modifiers: *modifiers,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count: 1,
                        key_code: composing.key_code,
                        modifiers: composing.modifiers,
                        range,
                    });
                } else if let InputState::DigitsAndCommand(count, composing) = input_state {
                    let range = Some(JumpCommandData {
                        count: 1,
                        key_code: *code,
                        modifiers: *modifiers,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count,
                        key_code: composing.key_code,
                        modifiers: composing.modifiers,
                        range,
                    });
                } else if let InputState::DigitsAndCommandAndDigits(count, composing, digits) =
                    input_state
                {
                    let range = Some(JumpCommandData {
                        count: digits.parse().unwrap(),
                        key_code: *code,
                        modifiers: *modifiers,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count,
                        key_code: composing.key_code,
                        modifiers: composing.modifiers,
                        range,
                    });
                } else {
                    return InputState::CommandInvalid(format!("Invalid command: {:?}", event));
                }
            }
            KeyEvent {
                code, modifiers, ..
            } if (*modifiers == KeyModifiers::NONE || *modifiers == KeyModifiers::SHIFT)
                && is_editing_command_without_range(code) =>
            {
                if let InputState::Start = input_state {
                    return InputState::CommandCompleted(CommandData {
                        count: 1,
                        key_code: *code,
                        modifiers: *modifiers,
                        range: None,
                    });
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    return InputState::CommandCompleted(CommandData {
                        count,
                        key_code: *code,
                        modifiers: *modifiers,
                        range: None,
                    });
                } else {
                    return InputState::CommandInvalid(format!("Invalid command: {:?}", event));
                }
            }
            KeyEvent {
                code, modifiers, ..
            } if (*modifiers == KeyModifiers::NONE || *modifiers == KeyModifiers::SHIFT)
                && is_editing_command_with_range(code) =>
            {
                if let InputState::Start = input_state {
                    input_state = InputState::CommandComposing(KeyData {
                        key_code: *code,
                        modifiers: *modifiers,
                    });
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    input_state = InputState::DigitsAndCommand(count, KeyData {
                        key_code: *code,
                        modifiers: *modifiers,
                    });
                } else if let InputState::CommandComposing(composing) = input_state {
                    if composing.key_code == *code {
                        // dd, cc, yy, etc.
                        let range = Some(JumpCommandData {
                            count: 1,
                            key_code: *code,
                            modifiers: *modifiers,
                        });
                        return InputState::CommandCompleted(CommandData {
                            count: 1,
                            key_code: composing.key_code,
                            modifiers: composing.modifiers,
                            range,
                        });
                    } else {
                        return InputState::CommandInvalid(format!("Invalid command: {:?}", event));
                    }
                } else if let InputState::CommandAndDigits(command, digits) = input_state {
                    return InputState::CommandInvalid(format!(
                        "Invalid command: {:?}{:?}{:?}",
                        command, digits, event
                    ));
                } else if let InputState::DigitsAndCommand(count, composing) = input_state {
                    if composing.key_code == *code {
                        // 3dd, 4cc, 5yy, etc.
                        let range = Some(JumpCommandData {
                            count: count,
                            key_code: *code,
                            modifiers: *modifiers,
                        });
                        return InputState::CommandCompleted(CommandData {
                            count: 1,
                            key_code: composing.key_code,
                            modifiers: composing.modifiers,
                            range,
                        });
                    } else {
                        return InputState::CommandInvalid(format!(
                            "Invalid command: {:?}{:?}{:?}",
                            composing.key_code, count, event
                        ));
                    }
                } else if let InputState::DigitsAndCommandAndDigits(count, command, digits) =
                    input_state
                {
                    return InputState::CommandInvalid(format!(
                        "Invalid command: {:?}{:?}{:?}{:?}",
                        command, count, digits, event
                    ));
                } else {
                    return InputState::CommandInvalid(format!("Invalid command: {:?}", event));
                }
            }
            KeyEvent {
                code,
                modifiers: KeyModifiers::CONTROL,
                ..
            } if is_ctrl_command(code) => {
                if let InputState::Start = input_state {
                    return InputState::CommandCompleted(CommandData {
                        count: 1,
                        key_code: *code,
                        modifiers: KeyModifiers::CONTROL,
                        range: None,
                    });
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    return InputState::CommandCompleted(CommandData {
                        count,
                        key_code: *code,
                        modifiers: KeyModifiers::CONTROL,
                        range: None,
                    });
                } else {
                    return InputState::CommandInvalid(format!("Invalid command: {:?}", event));
                }
            }
            KeyEvent {
                code: KeyCode::Enter,
                modifiers: KeyModifiers::NONE,
                ..
            } => {
                info!("Enter");
                return InputState::CommandCompleted(CommandData {
                    count: 1,
                    key_code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    range: None,
                });
            }
            _ => {
                info!("Other key: {:?}", event);
                ()
            }
        }
    }

    input_state
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_escape() {
        use super::compose;
        use super::InputState;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut key_events: Vec<KeyEvent> = Vec::new();
        key_events.push(KeyEvent {
            code: KeyCode::Esc,
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let input_state = compose(&key_events);
        assert_eq!(
            input_state,
            InputState::CommandCompleted(super::CommandData {
                count: 1,
                key_code: KeyCode::Esc,
                modifiers: KeyModifiers::NONE,
                range: None,
            })
        );
    }

    #[test]
    fn test_ctrl_g() {
        use super::compose;
        use super::InputState;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut key_events: Vec<KeyEvent> = Vec::new();
        key_events.push(KeyEvent {
            code: KeyCode::Char('g'),
            modifiers: KeyModifiers::CONTROL,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let input_state = compose(&key_events);
        assert_eq!(
            input_state,
            InputState::CommandCompleted(super::CommandData {
                count: 1,
                key_code: KeyCode::Char('g'),
                modifiers: KeyModifiers::CONTROL,
                range: None,
            })
        );
    }

    #[test]
    fn test_j() {
        use super::compose;
        use super::InputState;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut key_events: Vec<KeyEvent> = Vec::new();
        key_events.push(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let input_state = compose(&key_events);
        assert_eq!(
            input_state,
            InputState::CommandCompleted(super::CommandData {
                count: 1,
                key_code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                range: None,
            })
        );
    }

    #[test]
    fn test_4j() {
        use super::compose;
        use super::InputState;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut key_events: Vec<KeyEvent> = Vec::new();
        key_events.push(KeyEvent {
            code: KeyCode::Char('4'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let input_state = compose(&key_events);
        assert_eq!(
            input_state,
            InputState::CommandCompleted(super::CommandData {
                count: 4,
                key_code: KeyCode::Char('j'),
                modifiers: KeyModifiers::NONE,
                range: None,
            })
        );
    }

    #[test]
    fn test_dd() {
        use super::compose;
        use super::InputState;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut key_events: Vec<KeyEvent> = Vec::new();
        key_events.push(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let input_state = compose(&key_events);
        assert_eq!(
            input_state,
            InputState::CommandCompleted(super::CommandData {
                count: 1,
                key_code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                range: Some(super::JumpCommandData {
                    count: 1,
                    key_code: KeyCode::Char('d'),
                    modifiers: KeyModifiers::NONE,
                }),
            })
        );
    }

    #[test]
    fn test_d2j() {
        use super::compose;
        use super::InputState;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut key_events: Vec<KeyEvent> = Vec::new();
        key_events.push(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('2'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let input_state = compose(&key_events);
        assert_eq!(
            input_state,
            InputState::CommandCompleted(super::CommandData {
                count: 1,
                key_code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                range: Some(super::JumpCommandData {
                    count: 2,
                    key_code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::NONE,
                }),
            })
        );
    }

    #[test]
    fn test_3d4j() {
        use super::compose;
        use super::InputState;
        use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

        let mut key_events: Vec<KeyEvent> = Vec::new();
        key_events.push(KeyEvent {
            code: KeyCode::Char('3'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('d'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('4'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });
        key_events.push(KeyEvent {
            code: KeyCode::Char('j'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        });

        let input_state = compose(&key_events);
        assert_eq!(
            input_state,
            InputState::CommandCompleted(super::CommandData {
                count: 3,
                key_code: KeyCode::Char('d'),
                modifiers: KeyModifiers::NONE,
                range: Some(super::JumpCommandData {
                    count: 4,
                    key_code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::NONE,
                }),
            })
        );
    }
}
