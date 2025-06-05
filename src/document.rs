use std::fs;
use std::io::Write;
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
            unicode_segmentation::UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
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
            unicode_segmentation::UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
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
            unicode_segmentation::UnicodeSegmentation::graphemes(line_str.as_str(), true).collect();
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

}
