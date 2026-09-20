use crate::value::*;
use crate::opcode::*;

#[derive(PartialEq, Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub line: Vec<usize>,
    pub constants: Vec<Value>,
    pub arity: usize,
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            line: Vec::new(),
            constants: Vec::new(),
            arity: 0,
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.line.push(line);
    }

    pub fn write_opcode(&mut self, op: OpCode, line: usize) {
        self.write(op as u8, line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len()-1
    }
}