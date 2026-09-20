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


fn interpret(source: &str) {
    let mut lexer = lexer::Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens();

    let mut parser = parser::Parser::new(tokens);
    let stmts = parser.parse_prog();

    let mut compiler = compiler::Compiler::new();
    compiler.compile_program(&stmts);

    let mut vm = vm::VM::new();
    vm.interpret(compiler.chunk);
}
fn main() {
    println!("Hello, world!");
}
