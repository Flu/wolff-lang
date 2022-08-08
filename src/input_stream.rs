#[derive(Clone)]
pub struct InputStream {
    pos: u32,
    pub line: u32,
    pub col: u32,
    input: String
}

impl InputStream {
    pub fn new(input: &String) -> Self {
        InputStream {
            pos: 0,
            line: 0,
            col: 0,
            input: input.to_owned()
        }
    }

    pub fn next(&mut self) -> char {
        let next_char = self.peek();
        self.pos += 1;
        if next_char == '\n' {
            self.line += 1;
            self.col = 0;
        } else {
            self.col += 1;
        }

        next_char
    }

    pub fn peek(&self) -> char {
        self.get_char_at().unwrap()
    }

    pub fn eof(&self) -> bool {
        self.get_char_at().is_none()
    }

    fn get_char_at(&self) -> Option<char> {
        self.input.chars().nth(self.pos as usize)
    }

    pub fn _croak(_msg: &String) {
        unimplemented!("Send error message from this line and column and position")
    }
}