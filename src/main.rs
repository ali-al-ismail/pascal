use crate::editor::Editor;

mod editor;
mod term;
fn main() {
    let mut editor = Editor::build().unwrap();
    editor.run();
}
