use std::rc::Rc;
use crate::chunk::*;
use crate::expr::*;
use crate::stmt::Stmt;
use crate::value::*;
use crate::opcode::*;

pub struct Local {
    name: String,
    depth: u32
}
pub struct Compiler {
    pub chunk: Chunk,
    scope_depth: u32,
    locals: Vec<Local>
}

impl Compiler {
    fn new() -> Compiler {
        Compiler{
            chunk: Chunk::new(),
            scope_depth: 0,
            locals: Vec::new()
        }
    }

    fn compile(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(val) => {
                let index = self.chunk.add_constant(Value::Number(*val));
                self.chunk.write_opcode(OpCode::Constant, 0);
                self.chunk.write(index.try_into().unwrap(), 0);
            },

            Expr::Nil => {
                self.chunk.write_opcode(OpCode::Nil, 0);
            },

            Expr::Bool(val) => {
                if *val == true { 
                    self.chunk.write_opcode(OpCode::True, 0)
                } else { 
                    self.chunk.write_opcode(OpCode::False, 0) 
                }
            },

            Expr::Unary { operator, right } => {
                self.compile(&right);

                match operator {
                    UnaryOp::Negate => self.chunk.write_opcode(OpCode::Negate, 0),
                    UnaryOp::Not => self.chunk.write_opcode(OpCode::Not, 0),
                }
            },

            Expr::Binary { left, operator, right } => {
                self.compile(&left);
                self.compile(&right);

                match operator {
                    BinaryOp::Add => self.chunk.write_opcode(OpCode::Add, 0),
                    BinaryOp::Subtract => self.chunk.write_opcode(OpCode::Subtract, 0),
                    BinaryOp::Multiply => self.chunk.write_opcode(OpCode::Multiply, 0),
                    BinaryOp::Divide => self.chunk.write_opcode(OpCode::Divide, 0),

                    BinaryOp::Equal => self.chunk.write_opcode(OpCode::Equal, 0),
                    BinaryOp::NotEqual => {
                        self.chunk.write_opcode(OpCode::Equal, 0);
                        self.chunk.write_opcode(OpCode::Not, 0);
                    },

                    BinaryOp::Greater => self.chunk.write_opcode(OpCode::Greater, 0),
                    BinaryOp::GreaterEqual => {
                        self.chunk.write_opcode(OpCode::Less, 0);
                        self.chunk.write_opcode(OpCode::Not, 0);
                    },

                    BinaryOp::Less => self.chunk.write_opcode(OpCode::Less, 0),
                    BinaryOp::LessEqual => {
                        self.chunk.write_opcode(OpCode::Greater, 0);
                        self.chunk.write_opcode(OpCode::Not, 0);
                    }
                }
            },

            Expr::Grouping(val) => {
                self.compile(&val);
            },

            Expr::String(str) => {
                let index = self.chunk.add_constant(Value::String(str.to_string()));
                self.chunk.write_opcode(OpCode::Constant, 0);
                self.chunk.write(index.try_into().unwrap(), 0);
            },

            Expr::Identifier(name) => {
                for (index, local) in self.locals.iter().enumerate().rev() {
                    if local.name == *name {
                        self.chunk.write_opcode(OpCode::GetLocal, 0);
                        self.chunk.write(index.try_into().unwrap(), 0);
                        return;
                    }
                }
                let index = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write_opcode(OpCode::GetGlobal, 0);
                self.chunk.write(index.try_into().unwrap(), 0);
            },

            Expr::Assignment { name, value } => {
                self.compile(value);

                for (index, local) in self.locals.iter().enumerate().rev() {
                    if local.name == *name {
                        self.chunk.write_opcode(OpCode::SetLocal, 0);
                        self.chunk.write(index.try_into().unwrap(), 0);
                        return
                    }
                }
                let index = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write_opcode(OpCode::SetGlobal, 0);
                self.chunk.write(index.try_into().unwrap(), 0);
            },

            Expr::Call { callee, arguments } => {
                self.compile(callee);
                for argument in arguments.iter() {
                    self.compile(argument);
                }
                self.chunk.write_opcode(OpCode::Call, 0);
                self.chunk.write(arguments.len().try_into().unwrap(), 0);
            },
        }
    }

    fn compile_stmt(&mut self, stmt:&Stmt) {
        match stmt {
            Stmt::Let { name, value } => {
                match value {
                    Some(expr) => {
                        self.compile(expr);
                    }
                    None => {
                        self.chunk.write_opcode(OpCode::Nil, 0);
                    }
                }

                if self.scope_depth == 0 {
                    let index = self.chunk.add_constant(Value::String(name.clone()));
                    self.chunk.write_opcode(OpCode::DefineGlobal, 0);
                    self.chunk.write(index.try_into().unwrap(), 0);
                } else {
                    let local = Local {
                        name: name.clone(),
                        depth: self.scope_depth
                    };
                    self.locals.push(local);
                }
            },

            Stmt::Block(statements) => {
                self.scope_depth += 1;
                for statement in statements {
                    self.compile_stmt(statement);
                }
                self.scope_depth -= 1;
                while let Some(local) = self.locals.last() {
                    if local.depth <= self.scope_depth {
                        break;
                    }
                    self.locals.pop();
                    self.chunk.write_opcode(OpCode::Pop, 0);
                }
            },

            Stmt::Fun { name, parameters, body } => {
                let mut fun_compiler = Compiler::new();
                fun_compiler.scope_depth += 1;

                let local = Local {
                    name: name.clone(),
                    depth: fun_compiler.scope_depth,
                };
                fun_compiler.locals.push(local);

                for param in parameters.iter() {
                    let local = Local{
                        name: param.clone(),
                        depth: fun_compiler.scope_depth
                    };
                    fun_compiler.locals.push(local);
                }

                for body_stmt in body.iter() {
                    fun_compiler.compile_stmt(body_stmt);
                }

                let value = Rc::new(fun_compiler.chunk);
                let index = self.chunk.add_constant(Value::Fun(value));
                self.chunk.write_opcode(OpCode::Constant, 0);
                self.chunk.write(index.try_into().unwrap(), 0);

                if self.scope_depth == 0 {
                    let ind = self.chunk.add_constant(Value::String(name.clone()));
                    self.chunk.write_opcode(OpCode::DefineGlobal, 0);
                    self.chunk.write(ind.try_into().unwrap(), 0);
                } else {
                    let local = Local {
                        name: name.clone(),
                        depth: self.scope_depth
                    };
                    self.locals.push(local);
                }
            },

            Stmt::If { condition, then_branch, else_branch } => {
                self.compile(condition);
                let then_offset = self.emit_jump(OpCode::JumpIfFalse);

                self.chunk.write_opcode(OpCode::Pop, 0);
                for statement in then_branch {
                    self.compile_stmt(statement);
                }
                let else_offset = self.emit_jump(OpCode::Jump);

                self.patch_jump(then_offset);
                self.chunk.write_opcode(OpCode::Pop, 0);
                
                if let Some(else_branch) = else_branch {
                    for statement in else_branch {
                        self.compile_stmt(statement);
                    }
                }
                self.patch_jump(else_offset);
            },

            Stmt::While { condition, body } => {
                let loop_start = self.chunk.code.len();

                self.compile(condition);
                let false_offset = self.emit_jump(OpCode::JumpIfFalse);

                self.chunk.write_opcode(OpCode::Pop, 0);
                for statement in body {
                    self.compile_stmt(statement);
                }

                self.emit_loop(loop_start);
                self.patch_jump(false_offset);
                self.chunk.write_opcode(OpCode::Pop, 0);
            },

            Stmt::Expression(expr) => {
                self.compile(expr);
                self.chunk.write_opcode(OpCode::Pop, 0);
            },

            Stmt::Return(expr) => {
                match expr {
                    Some(expr) => self.compile(expr),
                    None => self.chunk.write_opcode(OpCode::Nil, 0),
                }
                self.chunk.write_opcode(OpCode::Return, 0);
            },

            _ => panic!()
        }
    }

    fn emit_jump(&mut self, op: OpCode) -> usize {
        self.chunk.write_opcode(op, 0);
        let idx = self.chunk.code.len();        
        self.chunk.write(0, 0);
        self.chunk.write(0, 0);
        return idx;
    }

    fn emit_loop(&mut self, loop_start: usize) {
        self.chunk.write_opcode(OpCode::Loop, 0);

        let offset = self.chunk.code.len() + 2 - loop_start;
        self.chunk.write((offset / 256).try_into().unwrap(), 0);
        self.chunk.write((offset % 256).try_into().unwrap(), 0);
    }

    fn patch_jump(&mut self, offset: usize) {
        let jump = self.chunk.code.len() - offset -2;

        self.chunk.code[offset] = (jump / 256).try_into().unwrap();  
        self.chunk.code[offset + 1] = (jump % 256).try_into().unwrap();
    }
}

#[test]
fn compiles_number_literal() {
    let mut compiler = Compiler::new();
    compiler.compile(&Expr::Number(5.0));

    assert_eq!(
        compiler.chunk.code,
        vec![
            OpCode::Constant as u8,
            0,
        ]
    );

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(5.0)
        ]
    );
}

#[test]
fn compiles_unary() {
    let mut compiler = Compiler::new();
    compiler.compile(
        &Expr::Unary {
            operator: UnaryOp::Negate,
            right: Box::new(Expr::Number(5.0)),
        }
    );

    assert_eq!(
        compiler.chunk.code,
        vec![
            OpCode::Constant as u8,
            0,
            OpCode::Negate as u8,
        ]
    );

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(5.0)
        ]
    );
}

#[test]
fn compiles_let_with_value() {
    let mut compiler = Compiler::new();

    compiler.compile_stmt(&Stmt::Let {
        name: "x".to_string(),
        value: Some(Expr::Number(42.0)),
    });

    assert_eq!(
        compiler.chunk.code,
        vec![
            OpCode::Constant as u8,
            0,
            OpCode::DefineGlobal as u8,
            1,
        ]
    );

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(42.0),
            Value::String("x".to_string()),
        ]
    );
}

#[test]
fn compiles_local_let() {
    let mut compiler = Compiler::new();

    compiler.compile_stmt(&Stmt::Block(vec![
        Stmt::Let {
            name: "x".to_string(),
            value: Some(Expr::Number(42.0)),
        },
    ]));

    assert_eq!(
        compiler.chunk.code,
        vec![
            OpCode::Constant as u8,
            0,
            OpCode::Pop as u8,
        ]
    );

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(42.0),
        ]
    );

    assert!(compiler.locals.is_empty());
}

#[test]
fn compiles_global_assignment() {
    let mut compiler = Compiler::new();

    compiler.compile(&Expr::Assignment {
        name: "x".to_string(),
        value: Box::new(Expr::Number(20.0)),
    });

    assert_eq!(
        compiler.chunk.code,
        vec![
            OpCode::Constant as u8,
            0,
            OpCode::SetGlobal as u8,
            1,
        ]
    );

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(20.0),
            Value::String("x".to_string()),
        ]
    );
}

#[test]
fn compiles_function() {
    let mut compiler = Compiler::new();

    compiler.compile_stmt(&Stmt::Fun {
        name: "foo".to_string(),
        parameters: vec![],
        body: vec![],
    });

    assert_eq!(
        compiler.chunk.code,
        vec![
            OpCode::Constant as u8,
            0,
            OpCode::DefineGlobal as u8,
            1,
        ]
    );

    assert_eq!(compiler.chunk.constants.len(), 2);

    assert_eq!(
        compiler.chunk.constants[1],
        Value::String("foo".to_string())
    );

    match &compiler.chunk.constants[0] {
        Value::Fun(chunk) => {
            assert!(chunk.code.is_empty());
            assert!(chunk.constants.is_empty());
        }
        _ => panic!("expected function"),
    }
}

#[test]
fn compiles_local_assignment() {
    let mut compiler = Compiler::new();

    compiler.scope_depth = 1;

    compiler.locals.push(Local {
        name: "x".to_string(),
        depth: 1,
    });

    compiler.compile(&Expr::Assignment {
        name: "x".to_string(),
        value: Box::new(Expr::Number(20.0)),
    });

    assert_eq!(
        compiler.chunk.code,
        vec![
            OpCode::Constant as u8,
            0,
            OpCode::SetLocal as u8,
            0,
        ]
    );

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(20.0),
        ]
    );
}

#[test]
fn compiles_while_with_local_assignment() {
    let mut compiler = Compiler::new();

    compiler.compile_stmt(&Stmt::Block(vec![
        Stmt::Let {
            name: "x".to_string(),
            value: Some(Expr::Number(0.0)),
        },

        Stmt::While {
            condition: Expr::Binary {
                left: Box::new(Expr::Identifier("x".to_string())),
                operator: BinaryOp::Less,
                right: Box::new(Expr::Number(10.0)),
            },

            body: vec![
                Stmt::Expression(
                    Expr::Assignment {
                        name: "x".to_string(),
                        value: Box::new(
                            Expr::Binary {
                                left: Box::new(
                                    Expr::Identifier("x".to_string())
                                ),
                                operator: BinaryOp::Add,
                                right: Box::new(Expr::Number(1.0)),
                            }
                        ),
                    }
                )
            ],
        },
    ]));

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(0.0),
            Value::Number(10.0),
            Value::Number(1.0),
        ]
    );

    assert!(compiler.locals.is_empty());
}

#[test]
fn compiles_function_with_if_and_returns() {
    let mut compiler = Compiler::new();

    compiler.compile_stmt(&Stmt::Fun {
        name: "abs".to_string(),

        parameters: vec![
            "x".to_string(),
        ],

        body: vec![
            Stmt::If {
                condition: Expr::Binary {
                    left: Box::new(
                        Expr::Identifier("x".to_string())
                    ),
                    operator: BinaryOp::Less,
                    right: Box::new(
                        Expr::Number(0.0)
                    ),
                },

                then_branch: vec![
                    Stmt::Return(Some(
                        Expr::Unary {
                            operator: UnaryOp::Negate,
                            right: Box::new(
                                Expr::Identifier("x".to_string())
                            ),
                        }
                    )),
                ],

                else_branch: Some(vec![
                    Stmt::Return(Some(
                        Expr::Identifier("x".to_string())
                    )),
                ]),
            },
        ],
    });

    match &compiler.chunk.constants[0] {
        Value::Fun(chunk) => {
            assert_eq!(
                chunk.constants,
                vec![
                    Value::Number(0.0),
                ]
            );

            // Don't assert the exact jump offsets yet.
            // First verify the important opcode structure.
            assert!(chunk.code.contains(&(OpCode::Less as u8)));
            assert!(chunk.code.contains(&(OpCode::JumpIfFalse as u8)));
            assert!(chunk.code.contains(&(OpCode::Negate as u8)));
            assert!(chunk.code.contains(&(OpCode::Return as u8)));
        }

        _ => panic!("expected function"),
    }
}

#[test]
fn compiles_while_with_if_else() {
    let mut compiler = Compiler::new();

    compiler.compile_stmt(&Stmt::Block(vec![
        Stmt::Let {
            name: "x".to_string(),
            value: Some(Expr::Number(0.0)),
        },

        Stmt::While {
            condition: Expr::Binary {
                left: Box::new(
                    Expr::Identifier("x".to_string())
                ),
                operator: BinaryOp::Less,
                right: Box::new(
                    Expr::Number(10.0)
                ),
            },

            body: vec![
                Stmt::If {
                    condition: Expr::Binary {
                        left: Box::new(
                            Expr::Identifier("x".to_string())
                        ),
                        operator: BinaryOp::Equal,
                        right: Box::new(
                            Expr::Number(5.0)
                        ),
                    },

                    then_branch: vec![
                        Stmt::Expression(
                            Expr::Assignment {
                                name: "x".to_string(),
                                value: Box::new(
                                    Expr::Number(10.0)
                                ),
                            }
                        ),
                    ],

                    else_branch: Some(vec![
                        Stmt::Expression(
                            Expr::Assignment {
                                name: "x".to_string(),
                                value: Box::new(
                                    Expr::Binary {
                                        left: Box::new(
                                            Expr::Identifier("x".to_string())
                                        ),
                                        operator: BinaryOp::Add,
                                        right: Box::new(
                                            Expr::Number(1.0)
                                        ),
                                    }
                                ),
                            }
                        ),
                    ]),
                },
            ],
        },
    ]));

    assert_eq!(
        compiler.chunk.constants,
        vec![
            Value::Number(0.0),
            Value::Number(10.0),
            Value::Number(5.0),
            Value::Number(10.0),
            Value::Number(1.0),
        ]
    );

    assert!(compiler.locals.is_empty());

    let code = &compiler.chunk.code;

    assert!(code.contains(&(OpCode::JumpIfFalse as u8)));
    assert!(code.contains(&(OpCode::Jump as u8)));
    assert!(code.contains(&(OpCode::Loop as u8)));

    assert!(code.contains(&(OpCode::GetLocal as u8)));
    assert!(code.contains(&(OpCode::SetLocal as u8)));

    assert!(code.contains(&(OpCode::Less as u8)));
    assert!(code.contains(&(OpCode::Equal as u8)));
    assert!(code.contains(&(OpCode::Add as u8)));
}