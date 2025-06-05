use crossterm::cursor;

use crate::mode::Mode;
pub struct StatusBar {
    pub file_name: String,
    pub mode: Mode,
    pub line_number: u16,
    pub has_unsaved_changes: bool,
}

impl StatusBar {
    pub fn new(file_name: String, mode: Mode, line_number: u16, has_unsaved_changes: bool) -> Self {
        StatusBar {
            file_name,
            mode,
            line_number,
            has_unsaved_changes,
        }
    }
    pub fn format(
        &self,
        width: u16,
        unsaved: bool,
        cursor_y: u16,
        cursor_x: u16,
        n_lines: u16,
    ) -> String {
        let mut mode = String::from("│ ");
        mode.push_str(&self.mode.to_string());
        let mut left_side = String::from(" ");
        left_side.push_str(&self.file_name);
        if unsaved {
            left_side.push_str(" [+]");
        } else {
            left_side.push_str("    ");
        }
        left_side.push_str(&mode);

        let right_side = format!(
            "{} │ {}/{}  ",
            cursor_x,
            if cursor_y <= n_lines {
                cursor_y
            } else {
                n_lines
            },
            n_lines - 1
        );

        let status_bar = format!(
            "{:<width$}│{:>right_width$}",
            left_side,
            right_side,
            width = width as usize - right_side.len() - 2,
            right_width = right_side.len() - 1
        );
        status_bar
    }
}
