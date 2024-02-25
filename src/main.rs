mod buffer;
mod command;
mod editor;
mod main_loop;
mod render;

use log::{error, info, warn};

fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    info!("Start the editor");

    let mut editor = editor::Editor::from_cmd_args(std::env::args().collect());
    main_loop::main_loop(&mut editor);
}
