use std::{fmt::Display, io::{stdout, Error, Stdout, Write}};
use crossterm::{cursor::{MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp}, style::Print, terminal::{Clear, ClearType}, QueueableCommand};

pub struct Terminal {
    pub width: u16,
    pub height: u16,
}

impl Terminal {
    pub fn build() -> Result<Terminal, Error> {
        let (width, height) = crossterm::terminal::size()?;
        crossterm::terminal::enable_raw_mode()?;
        Self::clear()?;
        Ok(Terminal { width, height })
    }

    pub fn clear() -> Result<(), Error> {
        stdout().queue(Clear(ClearType::All))?;
        Ok(())
    }

    pub fn flush() -> Result<(), Error> {
        stdout().flush()?;
        Ok(())
    }

    pub fn print<T: Display>(message: T)-> Result<(), Error>{
        stdout().queue(Print(message))?;
        Ok(())
    }

    pub fn move_cursor(row: u16, column: u16) -> Result<(), Error> {
        stdout().queue(MoveTo(row, column))?;
        Self::flush()?;
        Ok(())
    }

    pub fn move_left(n: u16) -> Result<(), Error> {
        stdout().queue(MoveLeft(n))?;
        Self::flush()?;
        Ok(())
    }
    pub fn move_right(n: u16) -> Result<(), Error> {
        stdout().queue(MoveRight(n))?;
        Self::flush()?;
        Ok(())
    }
    pub fn move_up(n: u16) -> Result<(), Error> {
        stdout().queue(MoveUp(n))?;
        Self::flush()?;
        Ok(())
    }
    pub fn move_down(n: u16) -> Result<(), Error> {
        stdout().queue(MoveDown(n))?;
        Self::flush()?;
        Ok(())
    }
    
}