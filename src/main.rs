use crate::editor::Editor;
use std::env::{self};
mod editor;
mod term;
fn main() {
    if let Some(file_name) = collect_args() {
        let mut editor = Editor::build(&file_name).unwrap();
        editor.run();
    } else {
        panic!("Couldn't parse file location...");
    }
}

fn collect_args() -> Option<String> {
    env::args().nth(1)
}
