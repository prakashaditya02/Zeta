use std::env;
use std::fs;
use std::io;
use std::io::*;

mod token;
mod lexer;
mod stmt;
mod expr;
mod parser;
mod precedence;
mod opcode;
mod value;
mod compiler;
mod chunk;
mod vm;


fn interpret(source: &str, vm: &mut vm::VM) {
    let mut lexer = lexer::Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();

    let mut parser = parser::Parser::new(tokens);
    let stmts = parser.parse_prog();

    let mut compiler = compiler::Compiler::new();
    compiler.compile_program(&stmts);

    vm.interpret(compiler.chunk);
}

fn input_is_complete(src: &str) -> bool {
    let mut brace = 0;
    let mut paren = 0;
    let mut in_string = false;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if in_string {
            if c == '"' {
                in_string = false;
            }
            continue;
        }
        if c == '"' {
            in_string = true;
            continue;
        }
        if c == '/' && chars.peek() == Some(&'/') {
            break;
        }
        if c == '{' {
            brace += 1;
        } else if c == '}' {
            brace -= 1;
        } else if c == '(' {
            paren += 1;
        } else if c == ')' {
            paren -= 1;
        }
    }
    if in_string {
        return false;
    }
    return brace <= 0 && paren <= 0;
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut vm = vm::VM::new();
    if args.len() > 1 {
        let source = fs::read_to_string(&args[1]).unwrap();
        interpret(&source, &mut vm);
    } else {
        let mut input = String::new();
        let mut buf = String::new();
        loop {
            if buf.is_empty() {
                print!(">> ");
            } else {
                print!(".. ");
            }
            io::stdout().flush().unwrap();
            input.clear();
            match io::stdin().read_line(&mut input) {
                Ok(0) => break,
                Ok(_) => {},
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }

            if buf.is_empty() {
                let trimmed = input.trim();
                if trimmed.is_empty() {
                    continue;
                } else if (trimmed == "Exit") || (trimmed == "exit") {
                    break;
                }
            }
            buf.push_str(&input);

            if !input_is_complete(&buf) {
                continue;
            }
            let src = buf.trim().to_string();
            buf.clear();
            if src.is_empty() {
                continue;
            }
            interpret(&src, &mut vm);
        }
    }
}
