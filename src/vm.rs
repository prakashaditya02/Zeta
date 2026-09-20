use std::rc::Rc;
use std::collections::HashMap;
use crate::chunk::*;
use crate::value::*;
use crate::opcode::*;

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    frames: Vec<CallFrame>
}

pub struct CallFrame  {
    chunk: Rc<Chunk>,
    ip: usize,
    base: usize
}

impl VM {
    fn new() -> VM {
        VM {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
        }
    }

    fn interpret(&mut self, chunk: Chunk) {
        let chunk = Rc::new(chunk);
        let callframe = CallFrame {
            chunk: chunk,
            ip: 0,
            base: 0,
        };
        self.frames.push(callframe);
        loop {
            let frame = self.frames.last_mut().unwrap();

            let instruction = frame.chunk.code[frame.ip];
            frame.ip += 1;

            let opcode = byte_to_opcode(instruction).unwrap();
            match opcode {
                OpCode::True  => {
                    self.stack.push(Value::Bool(true));
                },

                OpCode::False => {
                    self.stack.push(Value::Bool(false));
                },

                OpCode::Nil => {
                    self.stack.push(Value::Nil);
                },

                OpCode::Pop => {
                    self.stack.pop();
                },

                OpCode::Negate => {
                    let val = self.stack.pop().unwrap();
                    match val {
                        Value::Number(n) => {
                            self.stack.push(Value::Number(-n));
                        },

                        _ => {
                            panic!("Operand must be a number");
                        }
                    }
                },

                OpCode::Not => {
                    let op = self.stack.pop().unwrap();
                    self.stack.push(Value::Bool(op.is_falsy()));
                },

                OpCode::Add => {
                    let op2 = self.stack.pop().unwrap();
                    let op1 = self.stack.pop().unwrap();

                    match (op1, op2) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a+b));
                        },

                        _ => {
                            panic!("Operands must be numbers");
                        }
                    }
                },

                OpCode::Subtract => {
                    let op2 = self.stack.pop().unwrap();
                    let op1 = self.stack.pop().unwrap();

                    match (op1, op2) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a-b));
                        },

                        _ => {
                            panic!("Operands must be numbers");
                        }
                    }
                },

                OpCode::Multiply => {
                    let op2 = self.stack.pop().unwrap();
                    let op1 = self.stack.pop().unwrap();

                    match (op1, op2) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a*b));
                        },

                        _ => {
                            panic!("Operands must be numbers");
                        }
                    }
                },

                OpCode::Divide => {
                    let op2 = self.stack.pop().unwrap();
                    let op1 = self.stack.pop().unwrap();


                    match (op1, op2) {
                        (Value::Number(a), Value::Number(b)) => {
                            if b == 0.0 {
                                panic!("Divisor cannot be zero");
                            }
                            self.stack.push(Value::Number(a/b));
                        },

                        _ => {
                            panic!("Operands must be numbers");
                        }
                    }
                },

                OpCode::Equal => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(left == right));
                },

                OpCode::Greater => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Bool(a>b));
                        },

                        _ => {
                            panic!("Operands must be numbers");
                        }
                    }
                },

                OpCode::Less => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Bool(a<b));
                        },

                        _ => {
                            panic!("Operands must be numbers");
                        }
                    }
                },

                OpCode::Constant => {
                    let idx = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    let val = frame.chunk.constants[idx as usize].clone();
                    self.stack.push(val);
                },

                OpCode::DefineGlobal => {
                    let idx = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    let name_val = &frame.chunk.constants[idx as usize];
                    let name = match name_val {
                        Value::String(s) => s.clone(),
                        _ => panic!()
                    };

                    let value = self.stack.pop().unwrap();
                    self.globals.insert(name, value);
                },

                OpCode::GetGlobal => {
                    let idx = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    let name_val = &frame.chunk.constants[idx as usize];
                    let name = match name_val{
                        Value::String(s) => s.clone(),
                        _ => panic!()
                    };

                    match self.globals.get(&name) {
                        Some(x) => self.stack.push(x.clone()),
                        None => panic!("Undefined variable")
                    }
                },

                OpCode::SetGlobal => {
                    let idx = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    let name_val = &frame.chunk.constants[idx as usize];
                    let name = match name_val {
                        Value::String(s) => s.clone(),
                        _ => panic!()
                    };

                    let value = self.stack.last().unwrap().clone();

                    match self.globals.contains_key(&name) {
                        true => self.globals.insert(name, value),
                        false => panic!("Undefined Variable")
                    };
                },

                OpCode::GetLocal => {
                    let slot = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    let val = self.stack[frame.base + slot as usize].clone();
                    self.stack.push(val);
                },

                OpCode::SetLocal => {
                    let slot = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    let value = self.stack.last().unwrap().clone();
                    self.stack[frame.base + slot as usize] = value;
                },

                OpCode::Jump => {
                    let high = frame.chunk.code[frame.ip];
                    frame.ip += 1;
                    let low = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    frame.ip += (high as usize *256) + low as usize;
                },

                OpCode::JumpIfFalse => {
                    let high = frame.chunk.code[frame.ip];
                    frame.ip += 1;
                    let low = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    let offset = (high as usize * 256) + low as usize;

                    if self.stack.last().unwrap().is_falsy() {
                        frame.ip += offset;
                    }
                },

                OpCode::Loop => {
                    let high = frame.chunk.code[frame.ip];
                    frame.ip += 1;
                    let low = frame.chunk.code[frame.ip];
                    frame.ip += 1;

                    frame.ip -= (high as usize * 256) + low as usize;
                },

                OpCode::Call => {
                    let arg_count = frame.chunk.code[frame.ip];
                    frame.ip += 1;
                    let callee_idx = self.stack.len() - arg_count as usize - 1;
                    let callee = self.stack[callee_idx].clone();

                    match callee {
                        Value::Fun(chunk) => {
                            if arg_count as usize != chunk.arity {
                                panic!("Expected {} number of arguments but recieved {}", chunk.arity, arg_count);
                            }
                            let fn_frame = CallFrame {
                                chunk,
                                ip: 0,
                                base: callee_idx
                            };
                            self.frames.push(fn_frame);
                        },

                        _ => {
                            panic!("Only functions are callable")
                        }
                    }
                },

                OpCode::Return => {
                    let return_val = self.stack.pop().unwrap();
                    let frame = self.frames.pop().unwrap();

                    self.stack.truncate(frame.base);
                    self.stack.push(return_val);

                    if self.frames.is_empty() {
                        break;
                    }
                },

                _ => {
                    panic!();
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().unwrap();
        let byte = frame.chunk.code[frame.ip];
        frame.ip += 1;
        return byte;
    }

    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte() as usize;
        return self.frames.last().unwrap().chunk.constants[idx].clone();
    }
}