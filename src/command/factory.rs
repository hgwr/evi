use crate::command::base::{Command, CommandData, JumpCommandData};
use crate::command::commands::exit::ExitCommand;
use crate::command::commands::move_cursor::*;
use crate::command::commands::no_op_command::NoOpCommand;
use crossterm::event::{KeyCode, KeyModifiers};

use super::commands::append::Append;
use super::commands::change::Change;
use super::commands::delete::{Delete, DeleteChar};
use super::commands::find_char::{FindChar, RepeatFindChar};
use super::commands::go_to_file::{GoToFirstLine, GoToLastLine};
use super::commands::insert::Insert;
use super::commands::misc::DisplayFile;
use super::commands::open_line::OpenLine;
use super::commands::paste::Paste;
use super::commands::search::RepeatSearch;
use super::commands::undo::Undo;
use super::commands::yank::Yank;

pub fn command_factory(command_data: &CommandData) -> Box<dyn Command> {
    match command_data {
        CommandData {
            key_code: KeyCode::Char('j'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Down,
            ..
        } => Box::new(NextLine {}),
        CommandData {
            key_code: KeyCode::Char('k'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Up,
            ..
        } => Box::new(PreviousLine {}),
        CommandData {
            key_code: KeyCode::Char('l'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Right,
            ..
        } => Box::new(ForwardChar {}),
        CommandData {
            key_code: KeyCode::Char('h'),
            ..
        }
        | CommandData {
            key_code: KeyCode::Left,
            ..
        } => Box::new(BackwardChar {}),
        CommandData {
            key_code: KeyCode::Char('0'),
            ..
        } => Box::new(MoveBeginningOfLine {}),
        CommandData {
            key_code: KeyCode::Char('$'),
            ..
        } => Box::new(MoveEndOfLine {}),

        // page movement commands
        CommandData {
            key_code: KeyCode::Char('f'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Box::new(PageDown {}),
        CommandData {
            key_code: KeyCode::Char('b'),
            modifiers: KeyModifiers::CONTROL,
            ..
        } => Box::new(PageUp {}),
        CommandData {
            key_code: KeyCode::PageDown,
            ..
        } => Box::new(PageDown {}),
        CommandData {
            key_code: KeyCode::PageUp,
            ..
        } => Box::new(PageUp {}),

        // jump commands
        CommandData {
            key_code: KeyCode::Char('w'),
            ..
        } => Box::new(ForwardWord {}),
        CommandData {
            key_code: KeyCode::Char('b'),
            ..
        } => Box::new(BackwardWord {}),
        CommandData {
            key_code: KeyCode::Char('n'),
            ..
        } => Box::new(RepeatSearch {
            same_direction: true,
        }),
        CommandData {
            key_code: KeyCode::Char('N'),
            ..
        } => Box::new(RepeatSearch {
            same_direction: false,
        }),
        CommandData {
            key_code: KeyCode::Char('f'),
            ..
        } => Box::new(FindChar {
            count: command_data.count,
            direction: crate::editor::SearchDirection::Forward,
            inclusive: true,
        }),
        CommandData {
            key_code: KeyCode::Char('F'),
            ..
        } => Box::new(FindChar {
            count: command_data.count,
            direction: crate::editor::SearchDirection::Backward,
            inclusive: true,
        }),
        CommandData {
            key_code: KeyCode::Char('t'),
            ..
        } => Box::new(FindChar {
            count: command_data.count,
            direction: crate::editor::SearchDirection::Forward,
            inclusive: false,
        }),
        CommandData {
            key_code: KeyCode::Char('T'),
            ..
        } => Box::new(FindChar {
            count: command_data.count,
            direction: crate::editor::SearchDirection::Backward,
            inclusive: false,
        }),
        CommandData {
            key_code: KeyCode::Char(';'),
            ..
        } => Box::new(RepeatFindChar {}),
        CommandData {
            key_code: KeyCode::Char('G'),
            ..
        } => Box::new(GoToLastLine {
            count: command_data.count,
        }),
        CommandData {
            key_code: KeyCode::Char('g'),
            range:
                Some(JumpCommandData {
                    key_code: KeyCode::Char('g'),
                    ..
                }),
            ..
        } => Box::new(GoToFirstLine {
            count: command_data.count,
        }),

        // insert commands
        CommandData {
            key_code: KeyCode::Char('i'),
            ..
        } => Box::new(Insert::default()),

        // append commands
        CommandData {
            key_code: KeyCode::Char('a'),
            ..
        } => Box::new(Append::default()),

        // open line commands
        CommandData {
            key_code: KeyCode::Char('o'),
            ..
        } => Box::new(OpenLine::default()),
        CommandData {
            key_code: KeyCode::Char('O'),
            ..
        } => Box::new(OpenLine {
            above: true,
            ..Default::default()
        }),

        // delete commands
        CommandData {
            key_code: KeyCode::Char('x'),
            ..
        } => Box::new(DeleteChar::default()),

        CommandData {
            key_code: KeyCode::Char('d'),
            range,
            ..
        } => Box::new(Delete {
            jump_command_data_opt: range.clone(),
            ..Default::default()
        }),

        CommandData {
            key_code: KeyCode::Char('c'),
            range,
            ..
        } => Box::new(Change {
            jump_command_data_opt: range.clone(),
            ..Default::default()
        }),

        CommandData {
            key_code: KeyCode::Char('y'),
            range,
            ..
        } => Box::new(Yank {
            jump_command_data_opt: range.clone(),
            ..Default::default()
        }),

        CommandData {
            key_code: KeyCode::Char('p'),
            ..
        } => Box::new(Paste::default()),

        CommandData {
            key_code: KeyCode::Char('P'),
            ..
        } => Box::new(Paste {
            before: true,
            ..Default::default()
        }),

        CommandData {
            key_code: KeyCode::Char('r'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Box::new(super::commands::replace_char::ReplaceChar {
            count: command_data.count,
        }),

        CommandData {
            key_code: KeyCode::Char('R'),
            modifiers: KeyModifiers::NONE,
            ..
        } => Box::new(super::commands::replace::Replace::default()),

        // undo command
        CommandData {
            key_code: KeyCode::Char('u'),
            ..
        } => Box::new(Undo {}),

        CommandData {
            key_code: KeyCode::Char('.'),
            ..
        } => Box::new(super::commands::repeat::Repeat {}),

        // Control + g
        CommandData {
            key_code: KeyCode::Char('g'),
            modifiers,
            ..
        } if *modifiers == crossterm::event::KeyModifiers::CONTROL => Box::new(DisplayFile {}),

        // ZZ
        CommandData {
            key_code: KeyCode::Char('Z'),
            ..
        } => {
            if let Some(JumpCommandData {
                count,
                key_code: command,
                ..
            }) = command_data.range
            {
                if count == 1 && command == KeyCode::Char('Z') {
                    Box::new(ExitCommand {})
                } else {
                    Box::new(NoOpCommand {})
                }
            } else {
                Box::new(NoOpCommand {})
            }
        }
        _ => Box::new(NoOpCommand {}),
    }
}
