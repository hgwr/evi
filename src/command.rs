use crossterm::event::KeyCode;
use crate::editor::Editor;

#[derive(Copy, Clone)]
pub enum Jump {
  Left(usize),
  Right(usize),
  Up(usize),
  Down(usize),
  WordForward(usize),
}

pub trait Command {
  fn execute(&mut self, editor: &mut Editor);
}

pub trait JumpCommand: Command {
  fn new(range: Jump) -> Self;
  fn get(&self) -> Jump;
}

pub trait EditingCommand: Command {
  fn undo(&mut self, editor: &mut Editor);
  fn set_range(&mut self, range: Jump);
}

pub struct InsertCommand {
  text: String,
}

impl InsertCommand {
  pub fn new(text: String) -> InsertCommand {
    InsertCommand { text }
  }
}

impl Command for InsertCommand {
  fn execute(&mut self, editor: &mut Editor) {
    let row = editor.buffer.lines.get_mut(editor.cursor_position.row).unwrap();
    // row.insert(editor.cursor_position.col, self.text.clone());
  }
}

impl EditingCommand for InsertCommand {
  fn set_range(&mut self, range: Jump) {
  }
  fn undo(&mut self, editor: &mut Editor) {
    let row = editor.buffer.lines.get_mut(editor.cursor_position.row).unwrap();
    row.remove(editor.cursor_position.col);
  }
}

pub struct ExitCommand {
}

impl Command for ExitCommand {
  fn execute(&mut self, editor: &mut Editor){
    std::process::exit(0);
  }
}

pub struct JumpRightCommand {
  jump: Jump,
}

impl Command for JumpRightCommand {
  fn execute(&mut self, editor: &mut Editor) {
    match self.jump {
      Jump::Right(n) => {
        editor.cursor_position.col += n;
      },
      _ => {},
    }
  }
}

impl JumpCommand for JumpRightCommand {
  fn new(range: Jump) -> Self {
    JumpRightCommand { jump: range }
  }
  fn get(&self) -> Jump {
    self.jump
  }
}

// コマンドのパターンのリスト
//
// - 即時移動コマンド (l, h, j, k など)
// - 即時編集コマンド (i, x, d, c など)
// - 繰り返し指定付きの移動コマンド (3l, 4h, 5j, 6k など)
// - 繰り返し指定付きの編集コマンド (4x など)
// - 範囲つきの編集コマンド (d3w, c4e, 4dl など)

enum InputState {
  Idle,
  AccumulateDigits(String),
  CommandCompleted(Box<dyn Command>),
  EditingCommandComposing(String),  // 'd', 'c' など
  EditingCommandWithRangeComposing(String, String),  // 'd3', 'c4' など
  EditingCommandWithDigitsComposing(String, String),  // '3d', '4c' など
  TwoCharCommandComposing(String),  // 'Z' など

  CommandInvalid(String),
  CommandIncomplete(),
}

// vi コマンドの入力を受け取り、それを解釈してコマンドを生成する
pub fn compose(key_codes: &Vec<KeyCode>) -> InputState {
  let mut input_state = InputState::Idle;

  for key in key_codes {
    match key {
      KeyCode::Char(c) if c.is_digit(10) => {
        if let InputState::Idle = input_state {
          input_state = InputState::AccumulateDigits(c.to_string());
        } else if let InputState::AccumulateDigits(digits) = input_state {
          input_state = InputState::AccumulateDigits(format!("{}{}", digits, c));
        } else if let InputState::EditingCommandComposing(command) = input_state {
          input_state = InputState::EditingCommandWithRangeComposing(command, c.to_string());
        } else if let InputState::EditingCommandWithRangeComposing(command, digits) = input_state {
          input_state = InputState::EditingCommandWithRangeComposing(command, format!("{}{}", digits, c));
        } else {
          return InputState::CommandInvalid(format!("Invalid command: {:?}", key))
        }
      },
      KeyCode::Char('l') | KeyCode::Right => {
        if let InputState::Idle = input_state {
          return InputState::CommandCompleted(Box::new(JumpRightCommand::new(Jump::Right(1))));
        } else if let InputState::AccumulateDigits(digits) = input_state {
          return InputState::CommandCompleted(Box::new(JumpRightCommand::new(Jump::Right(digits.parse().unwrap()))));
        } else if let InputState::EditingCommandComposing(command) = input_state {
          return InputState::CommandCompleted(Box::new(JumpRightCommand::new(Jump::Right(1))));
        } else if let InputState::EditingCommandWithRangeComposing(command, digits) = input_state {
          return InputState::CommandCompleted(Box::new(JumpRightCommand::new(Jump::Right(digits.parse().unwrap()))));
        } else {
          return InputState::CommandInvalid(format!("Invalid command: {:?}", key))
        }
      },
      KeyCode::Char('i') => {
      },
      KeyCode::Char('x') => {
      },
      KeyCode::Char('Z') => {
      },
      _ => {
        ()
      }
    }
  }

  input_state
}
