use crate::editor::Editor;
use std::env::{self};
mod document;
mod editor;
mod highlighting;
mod mode;
mod render;
mod statusbar;
mod term;
fn main() {
    if let Some(file_name) = collect_args() {
        match Editor::build(&file_name) {
            Ok(mut editor) => editor.run(),
            Err(e) => {
                eprintln!("Error: Could not open file '{file_name}': {e}");
                std::process::exit(1);
            }
        }
    } else {
        eprintln!("Couldn't parse file location...");
        std::process::exit(1);
    }
}

fn collect_args() -> Option<String> {
    env::args().nth(1)
}
