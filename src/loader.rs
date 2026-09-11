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
    std_dir: PathBuf,
    resolved_mod: HashSet<String>,
}

impl Loader {
    pub fn new(base_dir: PathBuf, std_dir: PathBuf) -> Self {
        Self {
            base_dir,
            std_dir,
            resolved_mod: HashSet::new(),
        }
    }

    pub fn load(&mut self, file_path: &Path) -> Program {
        let mut program = self._load(file_path, None);

        let p = self.load_file(self.std_dir.join("cmp.rs"), Some("std::cmp".to_string()));
        program.nodes.extend(p.nodes);

        program
    }

    fn _load(&mut self, file_path: &Path, mod_name: Option<String>) -> Program {
        let original_file = Path::new(&file_path);

        let file_name = self.base_dir.join(original_file);
        let mut program = self.load_file(file_name, mod_name);

        let mut mods = self.get_mods(&program.nodes);
        while let Some(m) = mods.pop() {
            if !self.resolved_mod.insert(m.clone()) {
                continue;
            }

            let mod_program = self._load(Path::new(&format!("{m}.rs")), Some(m));
            program.nodes.extend(mod_program.nodes);
            mods.extend(self.get_mods(&program.nodes));
        }

        program
    }

    fn load_file(&self, file_name: PathBuf, mod_name: Option<String>) -> Program {
        let p = fs::read_to_string(file_name).unwrap();

        let tokens = token::tokenize(&p);
        let mut parser = ast::Parser::new(&tokens, mod_name);
        parser.program()
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
