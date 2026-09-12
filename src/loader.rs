use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    ast::{self, Module, Node, Program},
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

        program
            .modules
            .push(self.load_file(self.std_dir.join("cmp.rs"), Some("std::cmp".to_string())));

        program
    }

    fn _load(&mut self, file_path: &Path, mod_name: Option<String>) -> Program {
        let original_file = Path::new(&file_path);

        let file_name = self.base_dir.join(original_file);
        let m = self.load_file(file_name, mod_name);

        let mut mods = self.get_mods(&m.nodes);
        let mut modules = vec![m];
        while let Some(m) = mods.pop() {
            if !self.resolved_mod.insert(m.clone()) {
                continue;
            }

            let file_name = self.base_dir.join(Path::new(&format!("{m}.rs")));
            let module = self.load_file(file_name, Some(m));
            mods.extend(self.get_mods(&module.nodes));
            modules.push(module);
        }

        Program { modules }
    }

    fn load_file(&self, file_name: PathBuf, mod_name: Option<String>) -> Module {
        let p = fs::read_to_string(file_name).unwrap();

        let tokens = token::tokenize(&p);
        let mut parser = ast::Parser::new(&tokens, mod_name.clone());
        parser.module(mod_name)
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
