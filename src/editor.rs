use crate::render::Renderer;
use crate::statusbar::StatusBar;
use crate::term::Terminal;
use crate::{document::Document, mode::Mode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, read};
use std::{io::Error, path::Path};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;
const NAME: &str = "pascal-editor";
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Editor {
    pub term: Terminal,
    quit: bool,
    mode: Mode,
    pub docu: Document,
    pub cursor_x: u16,
    pub cursor_y: u16,
    pub top_offset: u16,
    pub left_offset: u16,
    pub status_bar: StatusBar,
}

impl Editor {
    pub fn build(file_path: &str) -> Result<Editor, Error> {
        let docu = Document::new(file_path);
        let term = Terminal::build()?;
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(file_path)
            .to_string();
        let status_bar = StatusBar::new(file_name, Mode::Normal, false);
        Ok(Editor {
            term,
            quit: false,
            mode: Mode::Normal,
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
                Terminal::move_cursor(0, 0).unwrap();
                break;
            }

            if let Err(e) = self.handle_event() {
                panic!("Error handling key presses: {e}");
            }
        }
    }

    fn render(&self) -> Result<(), Error> {
        Renderer::new(self).render()?;
        Ok(())
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
                Mode::Normal => {
                    self.handle_normal_mode_key_event(key);
                }
                Mode::Insert => {
                    self.handle_insert_mode_key_event(key);
                }
            }
            Terminal::clear()?;
            self.render()?;
        }
        // handle resize events
        if let Some(size) = event.as_resize_event() {
            let (width, height) = size;
            self.term.width = width;
            self.term.height = height;
            self.update_offsets();
            Terminal::clear()?;
            self.render()?;
        }
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
                KeyCode::Char('h' | 'j' | 'k' | 'l' | 'w' | 'b')
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
            (KeyCode::Char('t'), KeyModifiers::NONE) => {
                // move cursor to the bottom of the document
                if self.docu.n_lines > 0 {
                    self.cursor_y = self.docu.n_lines - 1;

                    // Set cursor to end of last line
                    if let Some(line) = self.docu.lines.get(self.cursor_y as usize) {
                        self.cursor_x = line.graphemes(true).count() as u16;
                    } else {
                        self.cursor_x = 0;
                    }
                } else {
                    self.cursor_y = 0;
                    self.cursor_x = 0;
                }
                self.update_offsets();
            }
            (KeyCode::Char('g'), KeyModifiers::NONE) => {
                // move cursor to the top of the document
                self.cursor_y = 0;
                self.cursor_x = 0;
                self.update_offsets();
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
            KeyCode::Tab => {
                let line = self.cursor_y;
                let col = self.cursor_x;
                for _ in 0..4 {
                    self.docu.insert_char(' ', line, col);
                    self.cursor_x += 1;
                }
            }
            _ => {}
        }
        self.update_offsets();
    }

    fn enter_insert(&mut self) {
        self.status_bar.mode = Mode::Insert;
        self.mode = Mode::Insert;
    }

    fn enter_normal(&mut self) {
        self.status_bar.mode = Mode::Normal;
        self.mode = Mode::Normal;
    }

    // moves cursor based on directional key pressed
    fn handle_movement(&mut self, direction: KeyCode) {
        match direction {
            KeyCode::Char('h') | KeyCode::Left => {
                if self.cursor_x > 0 {
                    self.cursor_x -= 1;
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

                    if self.cursor_y >= self.top_offset + self.term.height - 2 {
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
                    self.cursor_x += 1;
                }
            }
            KeyCode::Char('w') => {
                // move to the next word
                let line = self.cursor_y;
                let col = self.cursor_x;
                (self.cursor_y, self.cursor_x) = self.docu.next_word(line, col);
            }
            KeyCode::Char('b') => {
                // move to prev word
                let line = self.cursor_y;
                let col = self.cursor_x;
                (self.cursor_y, self.cursor_x) = self.docu.prev_word(line, col);
            }
            _ => {}
        }
        self.update_offsets();
    }

    fn update_top_offset(&mut self) {
        let margin = 4; // how many lines should be visible below the cursor at the bottom of the screen
        let bottom_content = self.term.height - 2; // the bottom content area which includes the status bar and the line below it
        if self.cursor_y < self.top_offset {
            self.top_offset = self.cursor_y;
        } else if self.cursor_y >= self.top_offset + bottom_content.saturating_sub(margin) {
            self.top_offset = self.cursor_y .saturating_sub(bottom_content.saturating_sub(margin + 1));
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

    
}
