use std::fmt::*;
use std::ops::Neg;

#[derive(Clone, Copy)]
pub enum Constant {
    Integer(i32),
    Float(f64),
}

impl Neg for Constant {
    type Output = Self;
    fn neg(self) -> Self::Output {
        return match self {
            Constant::Float(val) => Constant::Float(-val),
            Constant::Integer(val) => Constant::Integer(-val),
        };
    }
}

impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let output_string = match self {
            Constant::Float(val) => format!("{}", val),
            Constant::Integer(val) => format!("{}", val),
        };

        write!(f, "{}", output_string)
    }
}

pub struct VM {
    pub chunk: Chunk,
    ip: usize,
    debug: bool,
    stacktrace: bool,
    stack: Vec<Constant>,
    //globals: Vec<Object>,
    //frames: Vec<Frame>,
}

#[derive(Clone)]
pub struct Chunk {
    // Contains the bytecode program resulted from the compilation
    code: Vec<u8>,
    // Vector for mapping bytecode to the lines of source code from which they originated
    lines_mapping_vector: Vec<(usize, usize)>,
    // Vector for compile-time constants for the program
    constant_pool: Vec<Constant>,
}

// OpCode code that holds all the information about a specific instruction in the bytecode of our VM
#[derive(FromPrimitive, ToPrimitive)]
#[num_traits = "num_derived_traits"]
pub enum OpCode {
    Return = 0,
    Constant,
    Negate,
    Unknown,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let output_string: &str = match *self {
            OpCode::Return => "RETURN",
            OpCode::Constant => "CONST",
            OpCode::Negate => "NEGATE",
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
        new_chunk.lines_mapping_vector.push((0, 0));
        new_chunk
    }

    pub fn write_chunk(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        if self.lines_mapping_vector.last().unwrap().1 != line {
            self.lines_mapping_vector.push((self.code.len(), line));
        }
    }

    fn get_line(&self, offset: usize) -> usize {
        return match self.lines_mapping_vector.iter().position(|&x| offset < x.0) {
            None => self.lines_mapping_vector.last().unwrap().1,
            Some(position) => {
                if position == 0 {
                    0
                } else {
                    self.lines_mapping_vector[position - 1].1
                }
            }
        };
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

            println!(
                "{:04}\t{}\t\t@{}",
                offset, current_instruction, source_code_line
            );

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
            Some(OpCode::Negate) => (OpCode::Negate.to_string(), 1),
            _ => (OpCode::Unknown.to_string(), 1),
        }
    }

    fn constant_instruction(&self, offset: usize) -> String {
        let constant = &self.constant_pool[self.code[offset + 1] as usize];
        return match constant {
            Constant::Integer(val) => format!("CONST INT {}", val),
            Constant::Float(val) => format!("CONST FLOAT {}", val),
        };
    }
}

impl VM {
    pub fn new(debug: bool, stacktrace: bool) -> Self {
        VM {
            chunk: Chunk::new(),
            ip: 0,
            debug,
            stacktrace,
            stack: Vec::new(),
            //globals: Vec::new(),
            //frames: Vec::new(),
        }
    }

    pub fn from_chunk(chunk: &Chunk, debug: bool, stacktrace: bool) -> Self {
        VM {
            chunk: chunk.clone(),
            ip: 0,
            debug,
            stacktrace,
            stack: Vec::new(),
            //globals: Vec::new(),
            //frames: Vec::new(),
        }
    }

    pub fn interpret(&mut self) -> u32 {
        self.ip = 0;
        return self.run();
    }

    fn run(&mut self) -> u32 {
        // Run as long as there is code to run
        while self.ip != self.chunk.code.len() {
            // Current instruction is the byte at which self.ip points in the chunk being executed
            let curr_instruction = self.chunk.code[self.ip];

            // If debug is on, disassemble the current instruction and print it
            if self.debug {
                println!(
                    "{:04}\t{}",
                    self.ip,
                    self.chunk.disassemble_instruction(self.ip).0
                );

                // If stacktrace is true, print the stack after every instruction as well
                if self.stacktrace {
                    println!("---Stacktrace---");
                    for elem in self.stack.iter() {
                        print!("{}, ", elem);
                    }
                    println!("\n-------");
                }
            }

            // Match the current byte to an OpCode, if it doesn't match, spit out an error, else execute the instruction
            let ip_offset = match num::FromPrimitive::from_u8(curr_instruction) {
                None => return 1,
                Some(OpCode::Return) => {
                    return 0;
                }
                Some(OpCode::Constant) => {
                    let constant = self.chunk.constant_pool[self.chunk.code[self.ip + 1] as usize];
                    self.stack.push(constant);
                    2
                }
                Some(OpCode::Negate) => {
                    let constant = -self.stack.pop().unwrap();
                    self.stack.push(constant);
                    1
                }
                Some(OpCode::Unknown) => return 1,
            };

            self.ip += ip_offset;
        }

        // If the loop runs zero times, return 0 because technically it executed succesfully
        return 0;
    }
}
