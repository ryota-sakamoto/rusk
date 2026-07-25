use std::env::args;

mod ast;
mod code;
mod token;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("args should be specified.");
    }

    let p = &args[1];

    let tokens = token::tokenize(p);
    let mut parser = ast::Parser::new(&tokens);
    let program = parser.program();
    code::generate(&program);
}
