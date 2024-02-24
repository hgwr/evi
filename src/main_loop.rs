use crossterm::{
    event::{self, Event},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::stdout;

use crate::editor::Editor;

pub fn main_loop(editor: &mut Editor) {
    let mut stdout = stdout();
    let mut input_keys = Vec::new();

    terminal::enable_raw_mode();

    loop {
        if let Ok(Event::Key(key_event)) = event::read() {
            input_keys.push(key_event.code);
        } else if let Ok(Event::Resize(width, height)) = event::read() {
            stdout.execute(terminal::Clear(ClearType::All));
            println!("Terminal resized to width: {}, height: {}", width, height);
        }
    }

    terminal::disable_raw_mode();
}


