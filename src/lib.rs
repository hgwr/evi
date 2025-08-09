pub mod buffer;
pub mod editor;
pub mod command {
    pub mod base;
    pub mod factory;
    pub mod region;
    pub mod commands; // use commands/mod.rs for submodules
    pub mod compose;
    pub mod key_codes;
}
pub mod util;
pub mod generic_error;
pub mod data;
pub mod ex {
    pub mod lexer;
    pub mod parser;
}
pub mod render;
