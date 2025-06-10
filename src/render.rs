use crate::editor::Editor;
use crate::term::Terminal;
use std::io::Error;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub struct Renderer<'a> {
    editor: &'a Editor,
}

// currently rendering is done every time the user moves the cursor or edits a line
// optimization is possible by rendering only necessary lines which should improve performance on older systems
// possible optimizations:
// rerender only a single line when user edits it
// rerender only the line and the ones below it when the user does a newline
// rerender only the cursor when the user moves it
// rerender the entire screen only when it requires scrolling or it has been resized
// syntax highlighting is done every time a render operation is done, instead we should cache it and only recalculate when document changes
impl<'a> Renderer<'a> {
    /// This is called by Editor's render method
    pub fn new(editor: &'a Editor) -> Self {
        Renderer { editor }
    }

    /// Renders all the lines of the document and the cursor
    pub fn render(&self) -> Result<(), Error> {
        Terminal::clear()?;
        Terminal::hide_cursor()?;
        self.render_document_lines()?;
        self.render_status_bar()?;
        Terminal::show_cursor()?;
        self.render_cursor()?;
        Terminal::flush()?;
        Ok(())
    }

    /// re-renders a specific set of lines only
    pub fn re_render_line(&self, from: u16, to: u16) -> Result<(), Error> {
        Terminal::hide_cursor()?;
        let max_row = self.editor.term.height.saturating_sub(2); // Exclude status bar and last line
        for row in from..=to {
            if row >= max_row {
                break; // Don't render over status bar or below
            }
            Terminal::move_cursor(0, row)?;
            Terminal::clear_current_line()?;
            let doc_row = self.editor.top_offset + row;
            if doc_row < self.editor.docu.n_lines {
                self.render_content_line(doc_row)?;
            } else {
                self.render_empty_line()?;
            }
        }
        Terminal::show_cursor()?;
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
        let rich_line = &self.editor.docu.rich_lines[doc_row as usize];
        let available_width = width.saturating_sub((self.get_line_number_width() + 3) as u16);

        let highlighted_segments = &rich_line.line;

        let mut width_remaining = 0;
        let mut char_position = 0; // track position
        for segment in highlighted_segments {
            let segment_graphemes: Vec<&str> = segment.content.graphemes(true).collect();
            for grapheme in segment_graphemes {
                if char_position < self.editor.left_offset as usize {
                    char_position += 1;
                    continue;
                }
                let grapheme_width = grapheme.width() as u16;
                if width_remaining + grapheme_width > available_width {
                    Terminal::reset_color()?;
                    return Ok(());
                }
                Self::apply_styling(&segment.style)?;
                Terminal::print(grapheme)?;
                width_remaining += grapheme_width;
                char_position += 1;
            }
        }
        Terminal::reset_color()?;
        Ok(())
    }

    fn apply_styling(style: &syntect::highlighting::Style) -> Result<(), Error> {
        let fg = style.foreground;
        let foreground_color = crossterm::style::Color::Rgb {
            r: fg.r,
            g: fg.g,
            b: fg.b,
        };
        Terminal::set_foreground_color(foreground_color)?;
        Ok(())
    }

    fn render_empty_line(&self) -> Result<(), Error> {
        let empty_line = format!("{:>width$}", "~", width = self.get_line_number_width());
        Terminal::set_foreground_color(crossterm::style::Color::DarkGrey)?;
        Terminal::print(&empty_line)?;
        Terminal::reset_color()?;
        Ok(())
    }

    fn get_line_number_width(&self) -> usize {
        self.editor.docu.n_lines.to_string().len()
    }

    pub fn render_cursor(&self) -> Result<(), Error> {
        let cursor_screen_y = (self.editor.cursor_y.saturating_sub(self.editor.top_offset))
            .min(self.editor.term.height - 1);
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

    pub fn render_status_bar(&self) -> Result<(), Error> {
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
