use std::fmt;

pub enum Mode {
    NORMAL,
    INSERT,
}

impl fmt::Display for Mode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mode::NORMAL => write!(f, "NORMAL"),
            Mode::INSERT => write!(f, "INSERT"),
        }
    }
}
