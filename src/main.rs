use std::{env::args, fs};

mod ast;
mod code;
mod semantic;
mod token;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("args should be specified.");
    }

    let p = fs::read_to_string(&args[1]).unwrap();

    let tokens = token::tokenize(&p);
    let mut parser = ast::Parser::new(&tokens);
    let program = parser.program();
    semantic::analyze(&program);
    code::generate(&program);
}
