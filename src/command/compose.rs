use crossterm::event::KeyCode;

use crate::command::key_codes::{is_jump_command, is_editing_command_without_range, is_editing_command_with_range};
use crate::command::base::{CommandData, JumpCommandData};

// コマンドのパターンのリスト
//
// - 即時移動コマンド (l, h, j, k など)
// - 即時編集コマンド (i, x, d, ｓ など)
// - 繰り返し指定付きの移動コマンド (3l, 4h, 5j, 6k など)
// - 繰り返し指定付きの編集コマンド (4x, 3i[str] など)
// - 範囲つきの編集コマンド (d3w, c4e, 4dl など)

pub enum InputState {
  Start,
  AccumulateDigits(String), // Entering numbers
  CommandComposing(KeyCode),  // 'd', 'c' etc.
  CommandAndDigits(KeyCode, String),  // 'd3', 'c4' etc.
  DigitsAndCommand(usize, KeyCode),  // '3d', '4c' etc.
  DigitsAndCommandAndDigits(usize, KeyCode, String),  // '3d4', '4c3' etc.

  CommandCompleted(CommandData),
  CommandInvalid(String),
  CommandIncomplete(),
}

// Take vi command input, interpret it, and generate commands
pub fn compose(key_codes: &Vec<KeyCode>) -> InputState {
  let mut input_state = InputState::Start;

  for key in key_codes {
    match key {
      KeyCode::Char(c) if c.is_digit(10) => {
        if let InputState::Start = input_state {
          // 1st digit
          input_state = InputState::AccumulateDigits(c.to_string());
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
          input_state = InputState::DigitsAndCommandAndDigits(count, command, c.to_string());
        } else if let InputState::DigitsAndCommandAndDigits(count, command, digits) = input_state {
          // When a number follows a cursor movement command that specifies the number of times
          input_state = InputState::DigitsAndCommandAndDigits(count, command, format!("{}{}", digits, c));
        } else {
          return InputState::CommandInvalid(format!("Invalid command: {:?}", key))
        }
      },
      _ if is_jump_command(key) => {
        if let InputState::Start = input_state {
          return InputState::CommandCompleted(CommandData{count: 1, command: *key, range: None});
        } else if let InputState::AccumulateDigits(digits) = input_state {
          let count =  digits.parse().unwrap();
          return InputState::CommandCompleted(CommandData{command: *key, count, range: None});
        } else if let InputState::CommandComposing(command) = input_state {
          let range = Some(JumpCommandData{count: 1, command: *key});
          return InputState::CommandCompleted(CommandData{count: 1, command, range});
        } else if let InputState::CommandAndDigits(command, digits) = input_state {
          let range = Some(JumpCommandData{count: digits.parse().unwrap(), command: *key});
          return InputState::CommandCompleted(CommandData{count: 1, command, range});
        } else if let InputState::DigitsAndCommand(count, command) = input_state {
          let range = Some(JumpCommandData{count: 1, command: *key});
          return InputState::CommandCompleted(CommandData{count: count, command: command, range});
        } else if let InputState::DigitsAndCommandAndDigits(count, command, digits) = input_state {
          let range = Some(JumpCommandData{count: digits.parse().unwrap(), command: *key});
          return InputState::CommandCompleted(CommandData{count, command, range});
        } else {
          return InputState::CommandInvalid(format!("Invalid command: {:?}", key))
        }
      },
      _ if is_editing_command_without_range(key) => {
        if let InputState::Start = input_state {
          return InputState::CommandCompleted(CommandData{count: 1, command: *key, range: None});
        } else if let InputState::AccumulateDigits(digits) = input_state {
          let count = digits.parse().unwrap();
          return InputState::CommandCompleted(CommandData{count, command: *key, range: None});
        } else {
          return InputState::CommandInvalid(format!("Invalid command: {:?}", key))
        }
      },
      _ if is_editing_command_with_range(key) => {
        if let InputState::Start = input_state {
          input_state = InputState::CommandComposing(*key);
        } else if let InputState::AccumulateDigits(digits) = input_state {
          let count = digits.parse().unwrap();
          input_state = InputState::DigitsAndCommand(count, *key);
        } else if let InputState::CommandComposing(command) = input_state {
          if command == *key {
            // dd, cc, yy, etc.
            let range = Some(JumpCommandData{count: 1, command: *key});
            return InputState::CommandCompleted(CommandData{count: 1, command, range});
          } else {
            return InputState::CommandInvalid(format!("Invalid command: {:?}", key))
          }
        } else if let InputState::CommandAndDigits(command, digits) = input_state {
          return InputState::CommandInvalid(format!("Invalid command: {:?}{:?}{:?}", command, digits, key))
        } else if let InputState::DigitsAndCommand(count, command) = input_state {
          if command == *key {
            // 3dd, 4cc, 5yy, etc.
            let range = Some(JumpCommandData{count: count, command: *key});
            return InputState::CommandCompleted(CommandData{count: 1, command, range});
          } else {
            return InputState::CommandInvalid(format!("Invalid command: {:?}{:?}{:?}", command, count, key))
          }
        } else if let InputState::DigitsAndCommandAndDigits(count, command, digits) = input_state {
          return InputState::CommandInvalid(format!("Invalid command: {:?}{:?}{:?}{:?}", command, count, digits, key))
        } else {
          return InputState::CommandInvalid(format!("Invalid command: {:?}", key))
        }
      },
      KeyCode::Esc => {
        return InputState::CommandCompleted(CommandData{count: 1, command: *key, range: None});
      },
      KeyCode::Enter => {
        return InputState::CommandCompleted(CommandData{count: 1, command: *key, range: None});
      },
      _ => {
        ()
      }
    }
  }

  input_state
}
