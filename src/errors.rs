use std::fmt;
use colored::*; // Add `colored` crate to your dependencies in Cargo.toml

pub struct InvalidTokenError {
    pub message: String,
    pub line_as_string: String,
    pub line: usize,
    pub col: usize
}

impl fmt::Display for InvalidTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting line and column numbers
        let location = format!("{}:{}", self.line, self.col);

        // Formatting the error message in red
        let colored_message = self.message.red();

        // Write the output in the specified format
        write!(
            f,
            "{}    {}\n{}",
            location, self.line_as_string, colored_message
        )
    }
}

pub struct ParserError {
    pub message: String,
    pub line: usize,
    pub col: usize
} 