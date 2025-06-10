use std::fs;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;

use crate::highlighting::{HighlightedSegment, Highlighter};

pub struct RichLine {
    pub line: Vec<HighlightedSegment>,
}

pub struct Document {
    pub file_name: String,
    pub extension: String,
    pub lines: Vec<String>,        // maybe make this a richline type instead?
    pub rich_lines: Vec<RichLine>, // cached syntax highlighting, i need to implement a more efficient way to store them but this'll do
    pub n_lines: u16,
    pub highlighter: Highlighter,
}

impl RichLine {
    pub fn new(highlighter: &Highlighter, line: &str, extension: &str) -> Self {
        let highlighted_segments = highlighter.highlight_line(line, extension);
        RichLine {
            line: highlighted_segments,
        }
    }

    pub fn empty() -> Self {
        RichLine { line: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.line.is_empty()
    }

    pub fn recalc(&mut self, highlighter: &Highlighter, line: &str, extension: &str) {
        self.line = highlighter.highlight_line(line, extension);
    }
}

impl Document {
    pub fn new(file_name: &str) -> Self {
        let file = fs::read_to_string(file_name).unwrap_or_else(|_| String::new());
        let mut lines: Vec<String> = file.lines().map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let n_lines = lines.len() as u16;
        let file_name = file_name.to_string();
        // get extension if any
        let extension = file_name
            .rsplit_once('.')
            .map_or_else(String::new, |(_, ext)| ext.to_string());

        let highlighter = Highlighter::new();
        let mut rich_lines: Vec<RichLine> = Vec::new();
        for _ in 0..n_lines {
            rich_lines.push(RichLine::empty());
        }
        Document {
            file_name,
            extension,
            lines,
            rich_lines,
            n_lines,
            highlighter,
        }
    }

    pub fn save(&self) {
        let content = self.lines.join("\n");
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_name)
            .unwrap_or_else(|e| {
                panic!("Couldn't open or create file: {}, err: {e}", self.file_name);
            });
        file.write_all(content.as_bytes()).unwrap_or_else(|e| {
            panic!("Couldn't write to file: {}, err: {e}", self.file_name);
        });
    }

    pub fn insert_char(&mut self, c: char, line: u16, col: u16) {
        if line as usize >= self.lines.len() {
            return;
        }
        let line_str = &mut self.lines[line as usize];
        let mut graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
        if col as usize > graphemes.len() {
            return;
        }
        let binding = c.to_string();
        graphemes.insert(col as usize, &binding);
        *line_str = graphemes.concat();
        self.rich_lines[line as usize].recalc(&self.highlighter, line_str, &self.extension);
    }

    pub fn remove_char(&mut self, line: u16, col: u16) {
        if line as usize >= self.lines.len() {
            return;
        }
        let line_str = &mut self.lines[line as usize];
        let mut graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
        if col as usize >= graphemes.len() {
            return;
        }
        graphemes.remove(col as usize);
        *line_str = graphemes.concat();
        self.rich_lines[line as usize].recalc(&self.highlighter, line_str, &self.extension);
    }

    pub fn join_lines(&mut self, line: u16) {
        if line == 0 || line as usize >= self.lines.len() {
            return;
        }

        let current_line = self.lines.remove(line as usize);
        let prev_line_idx = line - 1;
        self.lines[prev_line_idx as usize].push_str(&current_line);
        self.n_lines -= 1;
        self.rich_lines.remove(line as usize);
        self.rich_lines[prev_line_idx as usize].recalc(
            &self.highlighter,
            &self.lines[prev_line_idx as usize],
            &self.extension,
        );
    }

    pub fn newline(&mut self, line: u16, col: u16) {
        let line_str = self.lines[line as usize].clone();
        let mut graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
        // case 1: user tries to add a new line at the end of the current line
        if col as usize == graphemes.len() {
            self.lines.insert(line as usize + 1, String::new());
            self.rich_lines.insert(
                line as usize + 1,
                RichLine::new(&Highlighter::new(), "", &self.extension),
            );
            self.n_lines += 1;
            return;
        }
        // case 2: user tries to either newline at the start of in the middle of a line
        let new_line = graphemes.split_off(col as usize);
        self.lines[line as usize] = graphemes.concat();
        self.lines.insert(line as usize + 1, new_line.concat());
        self.n_lines += 1;

        self.rich_lines[line as usize].recalc(
            &self.highlighter,
            &self.lines[line as usize],
            &self.extension,
        );
        self.rich_lines.insert(
            line as usize + 1,
            RichLine::new(&Highlighter::new(), &new_line.concat(), &self.extension),
        );
    }

    /// finds the next word operating on the following boundaries: whitespace, punctuation, any non-word character or underscore
    pub fn next_word(&self, line: u16, col: u16) -> (u16, u16) {
        let line_str = self.lines[line as usize].clone();
        let graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
        let mut cur_col = col as usize;
        let len = graphemes.len();
        // are we on the last character of the line? if so move to the next one if there is one
        if cur_col >= len {
            if line + 1 < self.n_lines {
                return (line + 1, 0);
            }
            return (line, len as u16);
        }
        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';
        // find the next non-word character aka exit our current word
        let cur_char = graphemes[cur_col].chars().next().unwrap_or(' '); // ' ' just incase there is no char
        if is_word_char(cur_char) {
            while cur_col < len {
                let cur_char = graphemes[cur_col].chars().next().unwrap_or(' ');
                if !is_word_char(cur_char) {
                    break;
                }
                cur_col += 1;
            }
        }

        // successfully left next word, now find next one
        while cur_col < len {
            let cur_char = graphemes[cur_col].chars().next().unwrap_or(' ');
            if is_word_char(cur_char) {
                break;
            }
            cur_col += 1;
        }

        (line, cur_col as u16)
    }

    pub fn prev_word(&self, line: u16, col: u16) -> (u16, u16) {
        let line_str = self.lines[line as usize].clone();
        let graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
        let mut cur_col = col as usize;

        // are we on the first character of the current line? if so move to the previous line if it exists
        if cur_col == 0 {
            if line > 0 {
                let prev_line_str = self.lines[(line - 1) as usize].clone();
                let prev_graphemes: Vec<&str> =
                    UnicodeSegmentation::graphemes(prev_line_str.as_str(), true).collect();
                return (line - 1, prev_graphemes.len() as u16);
            }
            return (line, 0);
        }

        let is_word_char = |c: char| c.is_alphanumeric() || c == '_';

        // escape our current word
        while cur_col > 0 {
            let prev_char = graphemes[cur_col - 1].chars().next().unwrap_or(' ');
            if is_word_char(prev_char) {
                break;
            }
            cur_col -= 1;
        }

        // go to the previous one
        while cur_col > 0 {
            let prev_char = graphemes[cur_col - 1].chars().next().unwrap_or(' ');
            if !is_word_char(prev_char) {
                break;
            }
            cur_col -= 1;
        }

        (line, cur_col as u16)
    }
}
