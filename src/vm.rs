use std::fmt::*;

pub enum Constant {
    Integer(i32),
    Float(f64)
}

pub struct Chunk {
    // Contains the bytecode program resulted from the compilation
    code: Vec<u8>,
    // Vector for mapping bytecode to the lines of source code from which they originated
    lines_mapping_vector: Vec<(usize, usize)>,
    // Vector for compile-time constants for the program
    constant_pool: Vec<Constant>
}

pub struct VM {
    pub chunk: Chunk,
    //ip: usize,
    //stack: Vec<Object>,
    //globals: Vec<Object>,
    //frames: Vec<Frame>,
}

// OpCode code that holds all the information about a specific instruction in the bytecode of our VM
#[derive(FromPrimitive, ToPrimitive)]
#[num_traits = "num_derived_traits"]
pub enum OpCode {
    Return = 0,
    Constant,
    Unknown,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let output_string: &str = match *self {
            OpCode::Return => "RETURN",
            OpCode::Constant => "CONST",
            OpCode::Unknown => "Unknown opcode byte, this is a big nono",
        };

        write!(f, "{}", output_string)
    }
}

impl Chunk {
    pub fn new() -> Self {
        let mut new_chunk = Chunk {
            code: Vec::new(),
            lines_mapping_vector: Vec::new(),
            constant_pool: Vec::new(),
        };
        new_chunk.lines_mapping_vector.push((0,0));
        new_chunk
    }

    pub fn write_chunk(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        if self.lines_mapping_vector.last().unwrap().1 != line {
            self.lines_mapping_vector.push((self.code.len(), line));
        }
    }

    pub fn get_line(&self, offset: usize) -> usize {
        return match self.lines_mapping_vector.iter().position(|&x| offset < x.0) {
            None => self.lines_mapping_vector.last().unwrap().1,
            Some(position) => if position == 0 { 0 } else { self.lines_mapping_vector[position - 1].1 }
        }
    }

    pub fn add_constant(&mut self, constant: Constant) -> u8 {
        self.constant_pool.push(constant);
        (self.constant_pool.len() - 1).try_into().unwrap()
    }

    pub fn disassemble_chunk(&mut self, name: &str) {
        println!("== {} ==", name);
        let mut offset = 0;
        loop {
            let (current_instruction, new_offset) = self.disassemble_instruction(offset);

            // Get source code line number that generated this bytecode sequence
            let source_code_line = self.get_line(offset);
            // if offset > 0 && source_code_line == self.get_line(offset - 1) {
            //     println!("{:04}\t{}", offset, current_instruction);
            // } else {
                println!("{:04}\t{}\t\t@{}", offset, current_instruction, source_code_line);
            // }
            offset += new_offset;
            if offset == self.code.len() {
                break;
            }
        }
        println!("=========");
    }

    fn disassemble_instruction(&self, offset: usize) -> (String, usize) {
        match num::FromPrimitive::from_u8(self.code[offset]) {
            None => ("Unknown operation byte".to_string(), 1),
            Some(OpCode::Return) => (OpCode::Return.to_string(), 1),
            Some(OpCode::Constant) => (self.constant_instruction(offset), 2),
            _ => (OpCode::Unknown.to_string(), 1),
        }
    }

    fn constant_instruction(&self, offset: usize) -> String {
        let constant = &self.constant_pool[self.code[offset+1] as usize];
        return match constant {
            Constant::Integer(val) => format!("CONST INT {}", val),
            Constant::Float(val) => format!("CONST FLOAT {}", val),
        }
    }
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: Chunk::new(),
            //ip: 0,
            //stack: Vec::new(),
            //globals: Vec::new(),
            //frames: Vec::new(),
        }
    }
}
