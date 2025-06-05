use crossterm::{
    QueueableCommand,
    cursor::{MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp},
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::{
    fmt::Display,
    io::{Error, Write, stdout},
};

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

    pub fn print<T: Display>(message: T) -> Result<(), Error> {
        stdout().queue(Print(message))?;
        Ok(())
    }

    pub fn move_cursor(column: u16, row: u16) -> Result<(), Error> {
        stdout().queue(MoveTo(column, row))?;
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

    pub fn set_background_color(color: Color) -> Result<(), Error> {
        stdout().queue(SetBackgroundColor(color))?;
        Self::flush()?;
        Ok(())
    }
    pub fn set_foreground_color(color: Color) -> Result<(), Error> {
        stdout().queue(SetForegroundColor(color))?;
        Self::flush()?;
        Ok(())
    }
    pub fn reset_color() -> Result<(), Error> {
        stdout().queue(ResetColor)?;
        Self::flush()?;
        Ok(())
    }
}
