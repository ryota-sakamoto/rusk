use std::{
    env::{args, current_dir},
    path::Path,
};

mod ast;
mod code;
mod hir;
mod loader;
mod scope;
mod semantic;
mod token;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("args should be specified.");
    }

    let mut current = current_dir().unwrap();
    current.push("std");

    let original_path = Path::new(&args[1]);
    let original_file = Path::new(original_path.file_name().unwrap());
    let base_dir = original_path.parent().unwrap().to_path_buf();

    let mut l = loader::Loader::new(base_dir, current);
    let program = l.load(original_file);
    let hir_program = semantic::analyze(&program);
    code::generate(&hir_program);
}
