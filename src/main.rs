use std::{env::args, path::Path};

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

    let original_path = Path::new(&args[1]);
    let original_file = Path::new(original_path.file_name().unwrap());
    let base_dir = original_path.parent().unwrap().to_path_buf();

    let mut l = loader::Loader::new(base_dir);
    let program = l.load(original_file, None);
    let hir_program = semantic::analyze(&program);
    code::generate(&hir_program);
}
