use crossterm::{
    QueueableCommand,
    cursor::MoveTo,
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
        Ok(())
    }

    pub fn set_background_color(color: Color) -> Result<(), Error> {
        stdout().queue(SetBackgroundColor(color))?;
        Ok(())
    }
    pub fn set_foreground_color(color: Color) -> Result<(), Error> {
        stdout().queue(SetForegroundColor(color))?;
        Ok(())
    }
    pub fn reset_color() -> Result<(), Error> {
        stdout().queue(ResetColor)?;
        Ok(())
    }

    pub fn clear_current_line() -> Result<(), Error> {
        stdout().queue(Clear(ClearType::CurrentLine))?;
        Ok(())
    }

    pub fn noblink_cursor() -> Result<(), Error> {
        stdout().queue(crossterm::cursor::SetCursorStyle::SteadyBlock)?;
        Ok(())
    }

    pub fn hide_cursor() -> Result<(), Error> {
        stdout().queue(crossterm::cursor::Hide)?;
        Ok(())
    }

    pub fn show_cursor() -> Result<(), Error> {
        stdout().queue(crossterm::cursor::Show)?;
        Ok(())
    }
}
