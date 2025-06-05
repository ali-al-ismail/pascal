use std::fs;
use std::io::Error;
use unicode_segmentation;
pub struct Document {
    pub lines: Vec<String>,
    pub n_lines: u16,
}

impl Document {
    pub fn new(file_name: &str) -> Result<Self, Error> {
        let file = fs::read_to_string(file_name)?;
        let mut lines: Vec<String> = file.lines().map(str::to_string).collect();
        if lines.is_empty() {
            lines.push(String::new());
        }
        let n_lines = lines.len() as u16;
        Ok(Document { lines, n_lines })
    }

    pub fn insert_char(&mut self){}

    pub fn remove_char(&mut self){}

    pub fn newline(&mut self) {}

    /// Splits the line at the cursor when user presses enter
    pub fn split_line(&mut self){}

}
