use std::{collections::HashSet, env::args, fs, path::Path};

use crate::ast::Program;

mod ast;
mod code;
mod hir;
mod semantic;
mod token;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("args should be specified.");
    }

    let original_path = Path::new(&args[1]);
    let base_dir = original_path.parent().unwrap();
    let original_file = Path::new(original_path.file_name().unwrap());

    let mut program = new_program(base_dir, original_file, None);
    let mut mods = program.mods.clone();
    let mut resolved = HashSet::new();
    while let Some(m) = mods.pop() {
        if !resolved.insert(m.clone()) {
            continue;
        }

        let mod_program = new_program(base_dir, Path::new(&format!("{m}.rs")), Some(m.clone()));
        program.functions.extend(mod_program.functions);
        mods.extend(mod_program.mods.clone());
    }

    let hir_program = semantic::analyze(&program);
    code::generate(&hir_program);
}

fn new_program(base: &Path, p: &Path, mod_name: Option<String>) -> Program {
    let file_name = base.join(p);
    let p = fs::read_to_string(file_name).unwrap();

    let tokens = token::tokenize(&p);
    let mut parser = ast::Parser::new(&tokens, mod_name);
    parser.program()
}
