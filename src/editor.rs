use crate::statusbar::StatusBar;
use crate::term::Terminal;
use crate::{document::Document, mode::Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, read};
use std::{fs, io::Error, path::Path};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
const NAME: &str = "pascal-editor";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    term: Terminal,
    quit: bool,
    mode: Mode,
    docu: Document,
    cursor_x: u16,
    cursor_y: u16,
    top_offset: u16,
    left_offset: u16,
    status_bar: StatusBar,
}

impl Editor {
    pub fn build(file_path: &str) -> Result<Editor, Error> {
        let docu = Document::new(file_path);
        let term = Terminal::build()?;
        let status_bar = StatusBar {
            file_name: Path::new(file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(file_path)
                .to_string(),
            mode: Mode::NORMAL,
            line_number: 0,
            has_unsaved_changes: false,
        };
        Ok(Editor {
            term,
            quit: false,
            mode: Mode::NORMAL,
            docu,
            cursor_x: 0,
            cursor_y: 0,
            top_offset: 0,
            left_offset: 0,
            status_bar,
        })
    }

    pub fn run(&mut self) {
        if let Err(e) = self.welcome() {
            panic!("Couldn't welcome because of: {e}");
        }
        // main editor loop

        if let Err(e) = self.render() {
            panic!("Couldn't render file: {e}");
        }

        loop {
            if self.quit {
                Terminal::clear().unwrap();
                break;
            }

            if let Err(e) = self.handle_event() {
                panic!("Error handling key presses: {e}");
            }
        }
    }

    fn welcome(&self) -> Result<(), Error> {
        let welcome_message = format!("{NAME} version-{VERSION}");
        let length = welcome_message.len() as u16;
        Terminal::move_cursor(((self.term.width - length) - 1) / 2, self.term.height / 3)?;
        Terminal::print(welcome_message)?;
        Terminal::move_cursor(0, 0)?;
        Terminal::flush()?;
        Ok(())
    }

    fn handle_event(&mut self) -> Result<(), Error> {
        let event = read()?;

        // TODO: HANDLE BASED ON MODE
        if let Some(key) = event.as_key_press_event() {
            match self.mode {
                Mode::NORMAL => {
                    self.handle_normal_mode_key_event(key);
                }
                Mode::INSERT => {
                    self.handle_insert_mode_key_event(key);
                }
            }
            Terminal::clear()?;
            self.render()?;
        }
        // TODO: HANDLE RESIZE EVENT TO RESIZE WIDTH HEIGHT OF TERM
        Ok(())
    }

    fn handle_normal_mode_key_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.quit = true;
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                self.docu.save();
                self.status_bar.has_unsaved_changes = false;
            }
            (
                KeyCode::Char('h' | 'j' | 'k' | 'l')
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Down
                | KeyCode::Up,
                KeyModifiers::NONE,
            ) => {
                self.handle_movement(key.code);
            }
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.enter_insert();
            }
            _ => {}
        }
    }

    fn handle_insert_mode_key_event(&mut self, key: KeyEvent) {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.enter_normal();
            }
            (
                KeyCode::Enter | KeyCode::Backspace | KeyCode::Tab | KeyCode::Char(_),
                KeyModifiers::NONE,
            ) => {
                self.handle_writing_event(key.code);
                self.status_bar.has_unsaved_changes = true;
            }
            (KeyCode::Left | KeyCode::Right | KeyCode::Up | KeyCode::Down, KeyModifiers::NONE) => {
                self.handle_movement(key.code);
            }
            _ => {}
        }
    }

    fn handle_writing_event(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                let line = self.cursor_y;
                let col = self.cursor_x;
                self.docu.insert_char(c, line, col);
                self.cursor_x += c.width().unwrap() as u16; // Move cursor right by the width of the character
            }
            KeyCode::Backspace => {
                // IF THE CURSOR ISNT POINTING AT THE LEFT EDGE
                if self.cursor_x > 0 {
                    let line = self.cursor_y;
                    let col = self.cursor_x - 1;
                    self.docu.remove_char(line, col);
                    self.cursor_x -= 1;
                }
                // IF THE CURSOR IS AT THE LEFT EDGE MOVE IT UP AND MERGE CURRENT LINE WITH LINE ABOVE
                else if self.cursor_y > 0 {
                    let prev_line = self.cursor_y - 1;
                    let prev_line_len =
                        self.docu.lines[prev_line as usize].graphemes(true).count() as u16;

                    // Join the current line with the one above it.
                    self.docu.join_lines(self.cursor_y);
                    self.cursor_y = prev_line;
                    self.cursor_x = prev_line_len;
                }
            }
            KeyCode::Enter => {
                let line = self.cursor_y;
                let col = self.cursor_x;
                self.docu.newline(line, col);
                // move cursor to new line
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
            KeyCode::Tab => {}
            _ => {}
        }
        self.update_offsets();
    }

    fn enter_insert(&mut self) {
        self.status_bar.mode = Mode::INSERT;
        self.mode = Mode::INSERT;
    }

    fn enter_normal(&mut self) {
        // TODO: REFLECT MODE CHANGE IN STATUS BAR
        self.status_bar.mode = Mode::NORMAL;
        self.mode = Mode::NORMAL;
    }

    // moves cursor based on directional key pressed
    fn handle_movement(&mut self, direction: KeyCode) {
        match direction {
            KeyCode::Char('h') | KeyCode::Left => {
                let line = &self.docu.lines[self.cursor_y as usize];
                let graphemes: Vec<&str> = line.graphemes(true).collect();
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
                    let width = graphemes[self.cursor_x as usize].width() as u16;
                }
            }
            KeyCode::Char('j') | KeyCode::Down => {
                if self.cursor_y + 1 < self.docu.n_lines {
                    self.cursor_y += 1;

                    // Clamp cursor_x to new line length so we dont get an out of bounds error if we move from a short line to a long one
                    let new_line = &self.docu.lines[self.cursor_y as usize];
                    let new_len = new_line.graphemes(true).count() as u16;
                    if self.cursor_x > new_len {
                        self.cursor_x = new_len;
                    }

                    if self.cursor_y >= self.top_offset + self.term.height - 1 {
                        self.top_offset += 1;
                    }
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if self.cursor_y > 0 {
                    self.cursor_y -= 1;

                    // Clamp for safety
                    let new_line = &self.docu.lines[self.cursor_y as usize];
                    let new_len = new_line.graphemes(true).count() as u16;
                    if self.cursor_x > new_len {
                        self.cursor_x = new_len;
                    }

                    if self.cursor_y < self.top_offset {
                        self.top_offset = self.top_offset.saturating_sub(1);
                    }
                }
            }
            KeyCode::Char('l') | KeyCode::Right => {
                let line = &self.docu.lines[self.cursor_y as usize];
                let graphemes: Vec<&str> = line.graphemes(true).collect();
                if (self.cursor_x as usize) < graphemes.len() {
                    let width = graphemes[self.cursor_x as usize].width() as u16;
                    self.cursor_x += 1;
                }
            }
            _ => {}
        }
        self.update_offsets();
    }

    fn update_top_offset(&mut self) {
        let height = self.term.height;
        if self.cursor_y < self.top_offset {
            self.top_offset = self.cursor_y;
        } else if self.cursor_y >= self.top_offset + height {
            self.top_offset = self.cursor_y - height + 1;
        }
    }

    fn update_left_offset(&mut self) {
        let line_number_width = (self.docu.n_lines.to_string().len() + 2) as u16;
        let available_width = self.term.width.saturating_sub(line_number_width);

        if self.cursor_x < self.left_offset {
            self.left_offset = self.cursor_x;
        } else if self.cursor_x >= self.left_offset + available_width {
            self.left_offset = self.cursor_x - available_width + 1;
        }
    }

    fn update_offsets(&mut self) {
        self.update_top_offset();
        self.update_left_offset();
    }

    /// Renders the lines of the document and the cursor
    fn render(&self) -> Result<(), Error> {
        self.render_document_lines()?;
        self.render_status_bar()?;
        self.render_cursor()?;
        Terminal::flush()?;
        Ok(())
    }

    fn render_document_lines(&self) -> Result<(), Error> {
        let height = self.term.height;
        for row in 0..height - 1 {
            Terminal::move_cursor(0, row)?;
            let doc_row = self.top_offset + row; // for vertical scrolling

            if doc_row < self.docu.n_lines {
                self.render_content_line(doc_row)?;
            } else {
                self.render_empty_line()?;
            }
        }
        self.render_cursor()?;
        Ok(())
    }

    fn render_content_line(&self, doc_row: u16) -> Result<(), Error> {
        self.render_line_number(doc_row)?;
        self.render_line_content(doc_row)?;

        Ok(())
    }

    fn render_line_number(&self, row: u16) -> Result<(), Error> {
        let line_number = row + 1;
        let line_number_str = format!(
            "{:>width$}",
            line_number,
            width = self.get_line_number_width()
        );
        // render the line number
        if row == self.cursor_y {
            Terminal::set_foreground_color(crossterm::style::Color::White)?;
        } else {
            Terminal::set_foreground_color(crossterm::style::Color::DarkGrey)?;
        }
        Terminal::print(line_number_str)?;

        // render the separator
        Terminal::set_foreground_color(crossterm::style::Color::DarkGrey)?;
        Terminal::print(" │ ")?;
        Terminal::reset_color()?;
        Ok(())
    }

    fn render_line_content(&self, doc_row: u16) -> Result<(), Error> {
        let width = self.term.width;
        let line = &self.docu.lines[doc_row as usize];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let mut rendered_line = String::new();
        let mut width_remaining = 0; // for horizontal scrolling
        let available_width = width.saturating_sub(self.get_line_number_width() as u16);

        // set up the line content while skipping left_offset number of graphemes
        for g in graphemes.iter().skip(self.left_offset as usize) {
            let graphene_width = g.width() as u16;
            if width_remaining + graphene_width > available_width {
                break; // stop rendering if exceed the available width
            }
            rendered_line.push_str(g);
            width_remaining += graphene_width;
        }
        Terminal::print(&rendered_line)?;
        Ok(())
    }

    fn render_empty_line(&self) -> Result<(), Error> {
        let empty_line = format!(
            "{:>width$} ",
            " ~ ",
            width = self.get_line_number_width() as usize - 3
        );
        Terminal::set_foreground_color(crossterm::style::Color::DarkGrey)?;
        Terminal::print(&empty_line)?;
        Terminal::reset_color()?;
        Ok(())
    }

    fn get_line_number_width(&self) -> usize {
        self.docu.n_lines.to_string().len() + 3
    }

    fn render_cursor(&self) -> Result<(), Error> {
        let cursor_screen_y =
            (self.cursor_y.saturating_sub(self.top_offset)).min(self.term.height - 1);
        let line = &self.docu.lines[self.cursor_y as usize];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let line_number_width = self.get_line_number_width() as u16;
        let mut cursor_screen_x = line_number_width
            + graphemes
                .iter()
                .skip(self.left_offset as usize)
                .take(self.cursor_x.saturating_sub(self.left_offset) as usize)
                .map(|g| g.width() as u16)
                .sum::<u16>();
        cursor_screen_x += 3; // for the line number and separator
        Terminal::move_cursor(cursor_screen_x, cursor_screen_y)?;
        Ok(())
    }

    fn render_status_bar(&self) -> Result<(), Error> {
        let status_bar = self.status_bar.format(
            self.term.width,
            self.status_bar.has_unsaved_changes,
            self.cursor_y,
            self.cursor_x,
            self.docu.n_lines,
        );

        // print status line at the bottom
        Terminal::move_cursor(0, self.term.height - 1)?;
        Terminal::set_background_color(crossterm::style::Color::White)?;
        Terminal::set_foreground_color(crossterm::style::Color::Black)?;
        Terminal::print(&status_bar)?;
        Terminal::reset_color()?;

        Ok(())
    }
}
