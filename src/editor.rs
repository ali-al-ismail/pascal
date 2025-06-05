use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, read};

use crate::term::Terminal;
use std::{fs, io::Error};

const NAME: &str = "pascal-editor";
const VERSION: &str = env!("CARGO_PKG_VERSION");

enum Mode {
    NORMAL,
    INSERT,
}

pub struct Editor {
    term: Terminal,
    quit: bool,
    mode: Mode,
    docu: Document,
    cursor_x: u16,
    cursor_y: u16,
    top_offset: u16,
}

struct Document {
    lines: Vec<String>,
    n_lines: u16,
}

impl Editor {
    pub fn build(file_path: &str) -> Result<Editor, Error> {
        let docu = Self::open_file(file_path)?;
        let term = Terminal::build()?;
        Ok(Editor {
            term,
            quit: false,
            mode: Mode::NORMAL,
            docu,
            cursor_x: 0,
            cursor_y: 0,
            top_offset: 0,
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
                break;
            }

            if let Err(e) = self.handle_event() {
                panic!("Error handling key presses: {e}");
            }
        }
    }

    fn tildize(&self) -> Result<(), Error> {
        for i in 0..self.term.height {
            Terminal::move_cursor(0, i)?;
            Terminal::print("~")?;
        }
        Terminal::flush()?;
        Ok(())
    }

    fn welcome(&self) -> Result<(), Error> {
        // tildes at the left side
        //self.tildize()?;

        // welcome message
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
                    self.handle_normal_mode_key_event(key)?;
                    Terminal::clear()?;
                    self.render()?;
                }
                Mode::INSERT => {
                    self.handle_insert_mode_key_event(key)?;
                    Terminal::clear()?;
                    self.render()?;
                }
            }
        }
        // TODO: HANDLE RESIZE EVENT TO RESIZE WIDTH HEIGHT OF TERM
        Ok(())
    }

    fn handle_normal_mode_key_event(&mut self, key: KeyEvent) -> Result<(), Error> {
        match (key.code, key.modifiers) {
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                self.quit = true;
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                //self.save();
                self.quit = true;
            }
            (
                KeyCode::Char('h' | 'j' | 'k' | 'l')
                | KeyCode::Left
                | KeyCode::Right
                | KeyCode::Down
                | KeyCode::Up,
                KeyModifiers::NONE,
            ) => {
                self.handle_movement(key.code)?;
            }
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.enter_insert();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_insert_mode_key_event(&mut self, key: KeyEvent) -> Result<(), Error> {
        match (key.code, key.modifiers) {
            (KeyCode::Esc, KeyModifiers::NONE) => {
                self.enter_normal();
            }
            _ => {}
        }
        Ok(())
    }

    fn enter_insert(&mut self) {
        // TODO: REFLECT MODE CHANGE IN STATUS BAR
        self.mode = Mode::INSERT;
    }

    fn enter_normal(&mut self) {
        // TODO: REFLECT MODE CHANGE IN STATUS BAR
        self.mode = Mode::NORMAL;
    }

    // moves cursor based on directional key pressed
    fn handle_movement(&mut self, direction: KeyCode) -> Result<(), Error> {
        // TODO: WRAP CURSOR IF RIGHT OR LEFT BOUNDS REACHED ON LINE
        // TODO: HANDLE HORIZONTAL OFFSET
        // TODO: GO TO TOP / BOTTOM BASED ON UP OR BOTTOM BOUNDS REACHED
        // TODO: HANDLE USING GRAPHEME CLUSTERS RATHER
        match direction {
            KeyCode::Char('h') | KeyCode::Left => {
                Terminal::move_left(1)?;
                self.cursor_x = self.cursor_x.saturating_sub(1);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                Terminal::move_down(1)?;
                self.cursor_y += 1;
            }
            KeyCode::Char('k') | KeyCode::Up => {
                Terminal::move_up(1)?;

                self.cursor_y = self.cursor_y.saturating_sub(1);
            }
            KeyCode::Char('l') | KeyCode::Right => {
                Terminal::move_right(1)?;
                self.cursor_x += 1;
            }
            _ => {}
        }
        self.update_top_offset();
        Ok(())
    }

    fn update_top_offset(&mut self) {
        let height = self.term.height;
        if self.cursor_y < self.top_offset {
            self.top_offset = self.cursor_y;
        } else if self.cursor_y >= self.top_offset + height {
            self.top_offset = self.cursor_y - height + 1;
        }
    }

    fn open_file(file_name: &str) -> Result<Document, Error> {
        let file = fs::read_to_string(file_name)?;
        let lines: Vec<String> = file.lines().map(str::to_string).collect();
        let n_lines = lines.len() as u16;
        Ok(Document { lines, n_lines })
    }

    fn render(&self) -> Result<(), Error> {
        // TODO: HORIZONTAL SCROLLING
        let n_lines = self.docu.n_lines;
        let height = self.term.height;
        let width = self.term.width;

        for row in 0..height {
            Terminal::move_cursor(0, row)?;
            let doc_row = self.top_offset + row;
            if doc_row < n_lines {
                let line = &self.docu.lines[doc_row as usize];
                Terminal::print(&line.chars().take(width as usize).collect::<String>())?;
            } else {
                Terminal::print("~")?;
            }
        }

        let cursor_screen_y = self.cursor_y.saturating_sub(self.top_offset);
        Terminal::move_cursor(self.cursor_x, cursor_screen_y)?;

        Terminal::flush()?;

        Ok(())
    }
}
