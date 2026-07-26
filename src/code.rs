use std::{collections::HashMap, fmt::Display};

use crate::ast::{Function, Node, Program};

pub fn generate(program: &Program) {
    let mut generator = Generator::new(program);
    generator.parse();
    generator.generate();
}

struct Generator<'a> {
    string_map: HashMap<&'a str, String>,
    string_index: u64,
    program: &'a Program,
}

impl<'a> Generator<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            string_map: HashMap::new(),
            string_index: 0,
            program,
        }
    }

    fn parse(&mut self) {
        for f in &self.program.functions {
            for node in &f.body {
                self.parse_node(node);
            }
        }
    }

    fn parse_node(&mut self, node: &'a Node) {
        match node {
            Node::CALL(_l, args) => {
                for v in args {
                    self.parse_node(v);
                }
            }
            Node::STRING(s) => {
                if !self.string_map.get(s.as_str()).is_some() {
                    self.string_map
                        .insert(s, format!("@.str.{}", self.string_index));
                    self.string_index += 1;
                }
            }
            _ => {}
        }
    }

    fn generate(&mut self) {
        for (k, v) in &self.string_map {
            println!(
                r#"{} = private unnamed_addr constant [{} x i8] c"{}\00""#,
                v,
                k.len() + 1,
                k,
            );
        }

        println!("declare i32 @printf(ptr, ...)");

        for f in self.program.functions.iter() {
            println!();
            let generator = GenerateFunction::new(f, &self.string_map);
            generator.generate();
        }
    }
}

pub enum Type {
    Int,
    Ptr,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Int => "i32",
                Type::Ptr => "ptr",
            }
        )
    }
}

struct Value {
    name: String,
    ty: Type,
}

struct GenerateFunction<'a> {
    function: &'a Function,
    index: u64,
    label: u64,
    map: HashMap<&'a str, String>,
    string_map: &'a HashMap<&'a str, String>,
    has_return: bool,
}

impl<'a> GenerateFunction<'a> {
    fn new(function: &'a Function, string_map: &'a HashMap<&'a str, String>) -> Self {
        Self {
            function,
            index: 0,
            label: 0,
            map: HashMap::new(),
            string_map,
            has_return: false,
        }
    }

    fn is_main(&self) -> bool {
        self.function.name == "main"
    }

    fn generate(mut self) {
        let mut regs = Vec::new();
        for arg in self.function.args.iter() {
            let reg = self.new_reg();
            self.map.insert(&arg.name, format!("%r{reg}"));
            regs.push(format!("{} %r{reg}", arg.ty));
        }

        println!(
            "define {} @{}({}) {{",
            if self.is_main() {
                "i32"
            } else {
                self.function.ty.as_str()
            },
            self.function.name,
            regs.into_iter().collect::<Vec<String>>().join(", "),
        );
        println!("entry:");

        for node in self.function.body.iter() {
            self.generate_node(node);
        }

        if !self.has_return {
            if self.function.ty != "void" {
                panic!("should have return");
            } else if self.is_main() {
                println!("  ret i32 0");
            } else {
                println!("  ret void");
            }
        }
        println!("}}");
    }

    fn new_reg(&mut self) -> u64 {
        let current = self.index;
        self.index += 1;

        return current;
    }

    fn new_label(&mut self) -> u64 {
        let current = self.label;
        self.label += 1;

        return current;
    }

    fn generate_node(&mut self, node: &'a Node) -> Value {
        match node {
            Node::ADD(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = add i32 {}, {}", reg, ln.name, rn.name);

                return Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                };
            }
            Node::SUB(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = sub i32 {}, {}", reg, ln.name, rn.name);

                return Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                };
            }
            Node::MUL(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = mul i32 {}, {}", reg, ln.name, rn.name);

                return Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                };
            }
            Node::DIV(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = sdiv i32 {}, {}", reg, ln.name, rn.name);

                return Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                };
            }
            Node::NUM(n) => {
                let reg = self.new_reg();
                println!("  %r{} = alloca i32", reg);
                println!("  store i32 {}, ptr %r{}", n, reg);

                let reg2 = self.new_reg();
                println!("  %r{} = load i32, ptr %r{}", reg2, reg);

                return Value {
                    name: format!("%r{reg2}"),
                    ty: Type::Int,
                };
            }
            Node::STRING(s) => {
                let name = self.string_map.get(s.as_str()).unwrap();
                return Value {
                    name: name.clone(),
                    ty: Type::Ptr,
                };
            }
            Node::RET(n) => {
                self.has_return = true;
                let ret = self.generate_node(n);
                println!("  ret {} {}", ret.ty, ret.name);
                return Value {
                    name: format!(""),
                    ty: Type::Int,
                };
            }
            Node::CALL(name, args) => {
                let mut call_args = Vec::new();

                for arg in args {
                    let ret = self.generate_node(arg);
                    call_args.push(ret);
                }

                let reg = self.new_reg();
                if call_args.len() > 0 {
                    println!(
                        "  %r{} = call i32 @{}({})",
                        reg,
                        name,
                        call_args
                            .iter()
                            .map(|reg| format!("{} {}", reg.ty, reg.name))
                            .collect::<Vec<String>>()
                            .join(", ")
                    );
                } else {
                    println!("  %r{} = call i32 @{}()", reg, name);
                }

                return Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                };
            }
            Node::LET(name, right) => {
                let r = self.generate_node(right);
                self.map.insert(name, r.name.clone());

                return r;
            }
            Node::RLET(name) => {
                let r = self.map.get(name.as_str()).unwrap();
                return Value {
                    name: r.clone(),
                    ty: Type::Int,
                };
            }
            Node::EQ(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = icmp eq i32 {}, {}", reg, ln.name, rn.name);

                return Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                };
            }
            Node::IF(l, body, ebody) => {
                let ln = self.generate_node(l);
                let label = self.new_label();

                let else_label = if ebody.len() > 0 {
                    format!("elseif_{label}")
                } else {
                    format!("else_{label}")
                };

                println!(
                    "  br i1 {}, label %if_{label}, label %{else_label}",
                    ln.name
                );

                println!("  if_{label}:");
                for f in body {
                    self.generate_node(f);
                }
                println!("  br label %else_{label}");

                if ebody.len() > 0 {
                    println!("  elseif_{label}:");
                    for f in ebody {
                        self.generate_node(f);
                    }
                }

                println!("  else_{label}:");

                return Value {
                    name: format!(""),
                    ty: Type::Int,
                };
            }
        }
    }
}
