use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use log::{error, info, warn};

use crate::command::base::{CommandData, JumpCommandData};
use crate::command::key_codes::{
    is_ctrl_command, is_editing_command_with_range, is_editing_command_without_range, is_jump_command
};

// コマンドのパターンのリスト
//
// - 即時移動コマンド (l, h, j, k など)
// - 即時編集コマンド (i, x, d, ｓ など)
// - 繰り返し指定付きの移動コマンド (3l, 4h, 5j, 6k など)
// - 繰り返し指定付きの編集コマンド (4x, 3i[str] など)
// - 範囲つきの編集コマンド (d3w, c4e, 4dl など)

#[derive(Debug)]
pub enum InputState {
    Start,

    AccumulateDigits(String),                          // Entering numbers
    CommandComposing(KeyCode),                         // 'd', 'c' etc.
    CommandAndDigits(KeyCode, String),                 // 'd3', 'c4' etc.
    DigitsAndCommand(usize, KeyCode),                  // '3d', '4c' etc.
    DigitsAndCommandAndDigits(usize, KeyCode, String), // '3d4', '4c3' etc.

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
                    command: KeyCode::Esc,
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
                            command: KeyCode::Char('0'),
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
                        command: *code,
                        range: None,
                    });
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    return InputState::CommandCompleted(CommandData {
                        command: *code,
                        count,
                        range: None,
                    });
                } else if let InputState::CommandComposing(command) = input_state {
                    let range = Some(JumpCommandData {
                        count: 1,
                        command: *code,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count: 1,
                        command,
                        range,
                    });
                } else if let InputState::CommandAndDigits(command, digits) = input_state {
                    let range = Some(JumpCommandData {
                        count: digits.parse().unwrap(),
                        command: *code,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count: 1,
                        command,
                        range,
                    });
                } else if let InputState::DigitsAndCommand(count, command) = input_state {
                    let range = Some(JumpCommandData {
                        count: 1,
                        command: *code,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count: count,
                        command: command,
                        range,
                    });
                } else if let InputState::DigitsAndCommandAndDigits(count, command, digits) =
                    input_state
                {
                    let range = Some(JumpCommandData {
                        count: digits.parse().unwrap(),
                        command: *code,
                    });
                    return InputState::CommandCompleted(CommandData {
                        count,
                        command,
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
                        command: *code,
                        range: None,
                    });
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    return InputState::CommandCompleted(CommandData {
                        count,
                        command: *code,
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
                    input_state = InputState::CommandComposing(*code);
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    input_state = InputState::DigitsAndCommand(count, *code);
                } else if let InputState::CommandComposing(command) = input_state {
                    if command == *code {
                        // dd, cc, yy, etc.
                        let range = Some(JumpCommandData {
                            count: 1,
                            command: *code,
                        });
                        return InputState::CommandCompleted(CommandData {
                            count: 1,
                            command,
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
                } else if let InputState::DigitsAndCommand(count, command) = input_state {
                    if command == *code {
                        // 3dd, 4cc, 5yy, etc.
                        let range = Some(JumpCommandData {
                            count: count,
                            command: *code,
                        });
                        return InputState::CommandCompleted(CommandData {
                            count: 1,
                            command,
                            range,
                        });
                    } else {
                        return InputState::CommandInvalid(format!(
                            "Invalid command: {:?}{:?}{:?}",
                            command, count, event
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
                        command: *code,
                        range: None,
                    });
                } else if let InputState::AccumulateDigits(digits) = input_state {
                    let count = digits.parse().unwrap();
                    return InputState::CommandCompleted(CommandData {
                        count,
                        command: *code,
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
                    command: KeyCode::Enter,
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
