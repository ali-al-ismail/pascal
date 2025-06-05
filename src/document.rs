use std::fs;
use std::io::Error;

pub struct Document {
    pub lines: Vec<String>,
    pub n_lines: u16,
}

impl Document {
    pub fn new(file_name: &str) -> Result<Self, Error> {
        let file = fs::read_to_string(file_name)?;
        let lines: Vec<String> = file.lines().map(str::to_string).collect();
        let n_lines = lines.len() as u16;
        Ok(Document { lines, n_lines })
    }
}
