mod main_loop;
mod buffer;
mod editor;
pub mod command;

fn main() {
    let mut editor = editor::Editor::from_cmd_args(std::env::args().collect());
    main_loop::main_loop(&mut editor);
}
