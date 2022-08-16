pub struct InvalidTokenError {
    pub message: String,
    pub line_as_string: String,
    pub line: usize,
    pub col: usize
}