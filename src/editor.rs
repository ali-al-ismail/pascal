use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, read},
    queue,
    style::Print,
    terminal::{Clear, ClearType},
};

use crate::term::Terminal;
use std::{fs, io::{stdout, Error, Stdout, Write}};

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
    docu: Vec<String>
}

impl Editor {
    pub fn build(file_path: &str) -> Result<Editor, Error> {
        let docu = Self::open_file(file_path)?;
        let term = Terminal::build()?;
        Ok(Editor {
            term,
            quit: false,
            mode: Mode::NORMAL,
            docu
        })
    }

    pub fn run(&mut self) {
        if let Err(e) = self.welcome() {
            panic!("Couldn't welcome because of: {e}");
        }
        // main editor loop
        self.render();
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
                }
                _ => {}
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
                //self.insertmode
            }
            _ => {}
        }
        Ok(())
    }

    // moves cursor based on directional key pressed
    fn handle_movement(&self, direction: KeyCode) -> Result<(), Error> {
        // TODO: MOVE BASED CURSOR BASED ON DIRECTIONAL KEY
        // WRAP CURSOR IF RIGHT OR LEFT BOUNDS REACHED
        // GO TO TOP / BOTTOM BASED ON UP OR BOTTOM BOUNDS REACHED
        match direction {
            (KeyCode::Char('h') | KeyCode::Left) => {
                Terminal::move_left(1)?;
            }
            (KeyCode::Char('j') | KeyCode::Right) => {
                Terminal::move_right(1)?;
            }
            (KeyCode::Char('k') | KeyCode::Down) => {
                Terminal::move_down(1)?;
            }
            (KeyCode::Char('l') | KeyCode::Up) => {
                Terminal::move_up(1)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn open_file(file_name: &str) -> Result<Vec<String>, Error>{
        let file = fs::read_to_string(file_name)?;
        Ok(file.lines().map(str::to_string).collect())
    }

    fn render(&self) -> Result<(), Error> {
        for (i, line) in self.docu.iter().enumerate() {
            Terminal::move_cursor(0, i as u16)?;
            Terminal::print(line)?;
        }
        Ok(())
     }
}
