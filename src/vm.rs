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

pub struct CallFrame {
    chunk: Rc<Chunk>,
    ip: usize,
    base: usize
}

fn print(args: &[Value]) -> Value {
    let prt: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", prt.join(" "));
    Value::Nil
}

impl VM {
    pub fn new() -> VM {
        VM {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) {
        let chunk = Rc::new(chunk);
        let callframe = CallFrame {
            chunk,
            ip: 0,
            base: 0,
        };
        self.frames.push(callframe);
        self.globals.insert("print".to_string(), Value::NativeFn(print));

        loop {
            let instruction = self.read_byte();
            let opcode = byte_to_opcode(instruction).unwrap();

            match opcode {
                OpCode::True => {
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
                        _ => panic!("Operand must be a number"),
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
                            self.stack.push(Value::Number(a + b));
                        },
                        _ => panic!("Operands must be numbers"),
                    }
                },

                OpCode::Subtract => {
                    let op2 = self.stack.pop().unwrap();
                    let op1 = self.stack.pop().unwrap();

                    match (op1, op2) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a - b));
                        },
                        _ => panic!("Operands must be numbers"),
                    }
                },

                OpCode::Multiply => {
                    let op2 = self.stack.pop().unwrap();
                    let op1 = self.stack.pop().unwrap();

                    match (op1, op2) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Number(a * b));
                        },
                        _ => panic!("Operands must be numbers"),
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
                            self.stack.push(Value::Number(a / b));
                        },
                        _ => panic!("Operands must be numbers"),
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
                            self.stack.push(Value::Bool(a > b));
                        },
                        _ => panic!("Operands must be numbers"),
                    }
                },

                OpCode::Less => {
                    let right = self.stack.pop().unwrap();
                    let left = self.stack.pop().unwrap();

                    match (left, right) {
                        (Value::Number(a), Value::Number(b)) => {
                            self.stack.push(Value::Bool(a < b));
                        },
                        _ => panic!("Operands must be numbers"),
                    }
                },

                OpCode::Constant => {
                    let val = self.read_constant();
                    self.stack.push(val);
                },

                OpCode::DefineGlobal => {
                    let name = self.read_string();
                    let value = self.stack.pop().unwrap();
                    self.globals.insert(name, value);
                },

                OpCode::GetGlobal => {
                    let name = self.read_string();
                    match self.globals.get(&name) {
                        Some(x) => self.stack.push(x.clone()),
                        None => panic!("Undefined variable"),
                    }
                },

                OpCode::SetGlobal => {
                    let name = self.read_string();
                    let value = self.stack.last().unwrap().clone();

                    match self.globals.contains_key(&name) {
                        true => { self.globals.insert(name, value); },
                        false => panic!("Undefined Variable"),
                    };
                },

                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    let frame = self.frames.last().unwrap();
                    let val = self.stack[frame.base + slot].clone();
                    self.stack.push(val);
                },

                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let frame = self.frames.last().unwrap();
                    let value = self.stack.last().unwrap().clone();
                    self.stack[frame.base + slot] = value;
                },

                OpCode::Jump => {
                    let offset = self.read_u16() as usize;
                    let frame = self.frames.last_mut().unwrap();
                    frame.ip += offset;
                },

                OpCode::JumpIfFalse => {
                    let offset = self.read_u16() as usize;
                    let condition = self.stack.last().unwrap().clone();
                    if condition.is_falsy() {
                        let frame = self.frames.last_mut().unwrap();
                        frame.ip += offset;
                    }
                },

                OpCode::Loop => {
                    let offset = self.read_u16() as usize;
                    let frame = self.frames.last_mut().unwrap();
                    frame.ip -= offset;
                },

                OpCode::Call => {
                    let arg_count = self.read_byte() as usize;
                    let callee_idx = self.stack.len() - arg_count - 1;
                    let callee = self.stack[callee_idx].clone();

                    match callee {
                        Value::Fun(chunk) => {
                            if arg_count != chunk.arity {
                                panic!("Expected {} arguments but received {}", chunk.arity, arg_count);
                            }
                            let fn_frame = CallFrame {
                                chunk,
                                ip: 0,
                                base: callee_idx,
                            };
                            self.frames.push(fn_frame);
                        },

                        Value::NativeFn(f) => {
                            let args = &self.stack[callee_idx + 1..];
                            let result = f(args);

                            self.stack.truncate(callee_idx);
                            self.stack.push(result);
                        },

                        _ => panic!("Only functions are callable"),
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
                }
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().unwrap();
        let byte = frame.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    fn read_u16(&mut self) -> u16 {
        let high = self.read_byte() as u16;
        let low = self.read_byte() as u16;
        (high << 8) | low
    }

    fn read_constant(&mut self) -> Value {
        let idx = self.read_byte() as usize;
        self.frames.last().unwrap().chunk.constants[idx].clone()
    }

    fn read_string(&mut self) -> String {
        match self.read_constant() {
            Value::String(s) => s,
            _ => panic!("Expected string constant"),
        }
    }
}