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
fn main() {
    let args: Vec<String> = env::args().collect();
    let mut vm = vm::VM::new();
    if args.len() > 1 {
        let source = fs::read_to_string(&args[1]).unwrap();
        interpret(&source, &mut vm);
    } else {
        let mut input = String::new();
        loop {
            print!(">> ");
            io::stdout().flush().unwrap();
            input.clear();
            io::stdin().read_line(&mut input).unwrap();

            input = input.trim().to_string();
            if input.is_empty() {
                continue;
            } else if (input == "Exit") || (input == "exit") {
                break;
            }
            interpret(&input, &mut vm);
        }
    }
}
