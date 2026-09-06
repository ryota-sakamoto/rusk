use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    ast::{self, Node, Program},
    token,
};

pub struct Loader {
    base_dir: PathBuf,
    resolved_mod: HashSet<String>,
}

impl Loader {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            base_dir,
            resolved_mod: HashSet::new(),
        }
    }

    pub fn load(&mut self, file_path: &Path, mod_name: Option<String>) -> Program {
        let original_file = Path::new(&file_path);

        let file_name = self.base_dir.join(original_file);
        let p = fs::read_to_string(file_name).unwrap();

        let tokens = token::tokenize(&p);
        let mut parser = ast::Parser::new(&tokens, mod_name);
        let mut program = parser.program();

        let mut mods = self.get_mods(&program.nodes);
        while let Some(m) = mods.pop() {
            if !self.resolved_mod.insert(m.clone()) {
                continue;
            }

            let mod_program = self.load(Path::new(&format!("{m}.rs")), Some(m));
            program.functions.extend(mod_program.functions);
            mods.extend(self.get_mods(&program.nodes));
        }

        program
    }

    fn get_mods(&self, nodes: &Vec<Node>) -> Vec<String> {
        let mut mods = Vec::new();
        for node in nodes {
            if let Node::Mod(m) = node {
                mods.push(m.clone());
            }
        }

        mods
    }
}
