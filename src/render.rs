use crate::editor::Editor;
use std::io::Error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;
use crate::term::Terminal;

/// Renders the cursor and the lines of the document
pub struct Renderer<'a> {
    editor: &'a Editor,
}

impl<'a> Renderer<'a> {
    /// This is called by Editor's render method
    pub fn new(editor: &'a Editor) -> Self {
        Renderer { editor }
    }

    /// Renders the lines of the document and the cursor
    pub fn render(&self) -> Result<(), Error> {
        self.render_document_lines()?;
        self.render_status_bar()?;
        self.render_cursor()?;
        Terminal::flush()?;
        Ok(())
    }

    fn render_document_lines(&self) -> Result<(), Error> {
        let height = self.editor.term.height;
        for row in 0..height - 1 {
            Terminal::move_cursor(0, row)?;
            let doc_row = self.editor.top_offset + row; // for vertical scrolling

            if doc_row < self.editor.docu.n_lines {
                self.render_content_line(doc_row)?;
            } else {
                self.render_empty_line()?;
            }
        }
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
        if row == self.editor.cursor_y {
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
        let width = self.editor.term.width;
        let line = &self.editor.docu.lines[doc_row as usize];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let mut rendered_line = String::new();
        let mut width_remaining = 0; // for horizontal scrolling
        let available_width = width.saturating_sub((self.get_line_number_width() +3) as u16);

        // set up the line content while skipping left_offset number of graphemes
        for g in graphemes.iter().skip(self.editor.left_offset as usize) {
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
            "{:>width$}",
            "~",
            width = self.get_line_number_width()
        );
        Terminal::set_foreground_color(crossterm::style::Color::DarkGrey)?;
        Terminal::print(&empty_line)?;
        Terminal::reset_color()?;
        Ok(())
    }

    fn get_line_number_width(&self) -> usize {
        self.editor.docu.n_lines.to_string().len()
    }

    fn render_cursor(&self) -> Result<(), Error> {
        let cursor_screen_y =
            (self.editor.cursor_y.saturating_sub(self.editor.top_offset)).min(self.editor.term.height - 1);
        let line = &self.editor.docu.lines[self.editor.cursor_y as usize];
        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let line_number_width = (self.get_line_number_width() + 3) as u16;
        let cursor_screen_x = line_number_width
            + graphemes
                .iter()
                .skip(self.editor.left_offset as usize)
                .take(self.editor.cursor_x.saturating_sub(self.editor.left_offset) as usize)
                .map(|g| g.width() as u16)
                .sum::<u16>();
        Terminal::move_cursor(cursor_screen_x, cursor_screen_y)?;
        Ok(())
    }

    fn render_status_bar(&self) -> Result<(), Error> {
        let status_bar = self.editor.status_bar.format(
            self.editor.term.width,
            self.editor.status_bar.has_unsaved_changes,
            self.editor.cursor_y,
            self.editor.cursor_x,
            self.editor.docu.n_lines,
        );

        // print status line at the bottom
        Terminal::move_cursor(0, self.editor.term.height - 2)?;
        Terminal::set_background_color(crossterm::style::Color::DarkBlue)?;
        Terminal::set_foreground_color(crossterm::style::Color::Black)?;
        Terminal::clear_current_line()?;
        Terminal::print(&status_bar)?;
        Terminal::reset_color()?;

        Ok(())
    }
}