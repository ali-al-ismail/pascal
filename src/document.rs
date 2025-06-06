use std::fs;
use std::io::Write;
use unicode_segmentation::UnicodeSegmentation;
pub struct Document {
    pub file_name: String,
    pub lines: Vec<String>,
    pub n_lines: u16,
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
        Document {
            file_name,
            lines,
            n_lines,
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
    }

    pub fn join_lines(&mut self, line: u16) {
        if line == 0 || line as usize >= self.lines.len() {
            return;
        }

        let current_line = self.lines.remove(line as usize);
        let prev_line_idx = line - 1;
        self.lines[prev_line_idx as usize].push_str(&current_line);
        self.n_lines -= 1;
    }

    pub fn newline(&mut self, line: u16, col: u16) {
        let line_str = self.lines[line as usize].clone();
        let mut graphemes: Vec<&str> =
            UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
        // case 1: user tries to add a new line at the end of the current line
        if col as usize == graphemes.len() {
            self.lines.insert(line as usize + 1, String::new());
            self.n_lines += 1;
            return;
        }
        // case 2: user tries to either newline at the start of in the middle of a line
        let new_line = graphemes.split_off(col as usize);
        self.lines[line as usize] = graphemes.concat();
        self.lines.insert(line as usize + 1, new_line.concat());
        self.n_lines += 1;
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
            if is_word_char(cur_char){
                break;
            }
            cur_col += 1;
        }

        (line, cur_col as u16)
    }

    pub fn prev_word(&self, line: u16, col: u16) -> (u16, u16) {
        let line_str = self.lines[line as usize].clone();
        let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
        let mut cur_col = col as usize;

        // are we on the first character of the current line? if so move to the previous line if it exists
        if cur_col == 0 {
            if line > 0 {
                let prev_line_str = self.lines[(line - 1) as usize].clone();
                let prev_graphemes: Vec<&str> = UnicodeSegmentation::graphemes(prev_line_str.as_str(), true).collect();
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
