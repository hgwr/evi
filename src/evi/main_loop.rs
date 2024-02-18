use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{self, ClearType},
    ExecutableCommand,
};
use std::io::{stdout, Write};

pub fn main_loop() {
    let mut stdout = stdout();

    terminal::enable_raw_mode();

    loop {
        if let Ok(Event::Key(key_event)) = event::read() {
            if key_event.code == KeyCode::Char('q') {
                break;
            }
        } else if let Ok(Event::Resize(width, height)) = event::read() {
            stdout.execute(terminal::Clear(ClearType::All));
            println!("Terminal resized to width: {}, height: {}", width, height);
        }
    }

    terminal::disable_raw_mode();
}
