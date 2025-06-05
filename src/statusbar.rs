use crate::mode::Mode;

pub struct StatusBar {
    pub file_name: String,
    pub mode: Mode,
    pub line_number: u16,
    pub has_unsaved_changes: bool,
}