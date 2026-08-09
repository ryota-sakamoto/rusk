use core::panic;
use std::{collections::HashMap, fmt::Display};

use crate::ast::{ComparisonType, Function, Node, Program};

pub fn generate(program: &Program) {
    let mut generator = Generator::new(program);
    generator.parse();
    generator.generate();
}

struct Generator<'a> {
    string_map: HashMap<&'a str, String>,
    function_map: HashMap<&'a str, &'a str>,
    string_index: u64,
    program: &'a Program,
}

impl<'a> Generator<'a> {
    fn new(program: &'a Program) -> Self {
        Self {
            string_map: HashMap::new(),
            function_map: HashMap::new(),
            string_index: 0,
            program,
        }
    }

    fn parse(&mut self) {
        self.function_map.insert("printf", "i32");
        for f in &self.program.functions {
            self.parse_node(&f.body);
            self.function_map.insert(&f.name, &f.ty);
        }
    }

    fn parse_node(&mut self, node: &'a Node) {
        match node {
            Node::Call(_l, args) => {
                for v in args {
                    self.parse_node(v);
                }
            }
            Node::String(s) if !self.string_map.contains_key(s.as_str()) => {
                self.string_map
                    .insert(s, format!("@.str.{}", self.string_index));
                self.string_index += 1;
            }
            Node::Block(body) => {
                for v in body {
                    self.parse_node(v);
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
            let generator = GenerateFunction::new(f, &self.string_map, &self.function_map);
            generator.generate();
        }
    }
}

#[derive(Clone)]
pub enum Type {
    Int,
    Int8,
    Ptr(Box<Type>),
}

impl From<&str> for Type {
    fn from(value: &str) -> Self {
        match value {
            "i32" => Type::Int,
            "i8" => Type::Int8,
            _ => panic!("not supported: {value}"),
        }
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Type::Int => "i32",
                Type::Int8 => "i8",
                Type::Ptr(_) => "ptr",
            }
        )
    }
}

impl Type {
    fn inner(&self) -> &Self {
        match self {
            Type::Ptr(v) => v,
            _ => panic!("not ptr"),
        }
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
    map: HashMap<&'a str, Value>,
    string_map: &'a HashMap<&'a str, String>,
    function_map: &'a HashMap<&'a str, &'a str>,
    has_return: bool,
}

impl<'a> GenerateFunction<'a> {
    fn new(
        function: &'a Function,
        string_map: &'a HashMap<&'a str, String>,
        function_map: &'a HashMap<&'a str, &'a str>,
    ) -> Self {
        Self {
            function,
            index: 0,
            label: 0,
            map: HashMap::new(),
            string_map,
            function_map,
            has_return: false,
        }
    }

    fn is_main(&self) -> bool {
        self.function.name == "main"
    }

    fn generate(mut self) {
        let mut regs = Vec::new();
        let mut entry_instructions = Vec::new();

        for arg in self.function.args.iter() {
            let arg_reg = self.new_reg();
            regs.push(format!("{} %r{arg_reg}", arg.ty));

            let reg = self.new_reg();
            entry_instructions.push(format!("  %r{reg} = alloca {}", arg.ty));
            entry_instructions.push(format!("  store {} %r{arg_reg}, ptr %r{reg}", arg.ty));
            self.map.insert(
                &arg.name,
                Value {
                    name: format!("%r{reg}"),
                    ty: Type::Ptr(Box::new(Type::from(arg.ty.as_str()))),
                },
            );
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
        for v in entry_instructions {
            println!("{v}");
        }
        self.generate_node(&self.function.body);

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

        current
    }

    fn new_label(&mut self) -> u64 {
        let current = self.label;
        self.label += 1;

        current
    }

    fn generate_node(&mut self, node: &'a Node) -> Value {
        match node {
            Node::Add(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = add {} {}, {}", reg, ln.ty, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: ln.ty,
                }
            }
            Node::Sub(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = sub {} {}, {}", reg, ln.ty, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: ln.ty,
                }
            }
            Node::Mul(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = mul {} {}, {}", reg, ln.ty, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: ln.ty,
                }
            }
            Node::Div(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = sdiv {} {}, {}", reg, ln.ty, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: ln.ty,
                }
            }
            Node::Num(n) => Value {
                name: n.to_string(),
                ty: Type::Int,
            },
            Node::String(s) => {
                let name = self.string_map.get(s.as_str()).unwrap();
                Value {
                    name: name.clone(),
                    ty: Type::Ptr(Box::new(Type::Int)),
                }
            }
            Node::Ret(n) => {
                self.has_return = true;
                let ret = self.generate_node(n);
                println!("  ret {} {}", ret.ty, ret.name);
                Value {
                    name: String::new(),
                    ty: ret.ty,
                }
            }
            Node::Call(name, args) => {
                let mut call_args = Vec::new();

                for arg in args {
                    let ret = self.generate_node(arg);
                    call_args.push(ret);
                }

                let fn_ty = self.function_map[name.as_str()];

                let reg = self.new_reg();
                if !call_args.is_empty() {
                    println!(
                        "  %r{} = call {} @{}({})",
                        reg,
                        fn_ty,
                        name,
                        call_args
                            .iter()
                            .map(|reg| format!("{} {}", reg.ty, reg.name))
                            .collect::<Vec<String>>()
                            .join(", ")
                    );
                } else {
                    println!("  %r{} = call {} @{}()", reg, fn_ty, name);
                }

                Value {
                    name: format!("%r{reg}"),
                    ty: Type::from(fn_ty),
                }
            }
            Node::Let(name, right) => {
                let reg = self.new_reg();
                let r = self.generate_node(right);
                println!("  %r{reg} = alloca i32");
                println!("  store i32 {}, ptr %r{}", r.name, reg);

                self.map.insert(
                    name,
                    Value {
                        name: format!("%r{reg}"),
                        ty: Type::Ptr(Box::new(Type::Int)),
                    },
                );

                r
            }
            Node::RLet(name) => {
                let reg = self.new_reg();
                let r = self.map.get(name.as_str()).unwrap();
                let inner_ty = r.ty.inner();
                println!("  %r{reg} = load {}, ptr {}", inner_ty, r.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: inner_ty.clone(),
                }
            }
            Node::Assign(name, r) => {
                let rn = self.generate_node(r);
                let l = self.map.get(name.as_str()).unwrap();
                println!("  store {} {}, ptr {}", rn.ty, rn.name, l.name);

                rn
            }
            Node::Comparison(t, l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!(
                    "  %r{} = icmp {} i32 {}, {}",
                    reg,
                    match t {
                        ComparisonType::Eq => "eq",
                        ComparisonType::Ne => "ne",
                        ComparisonType::Gt => "sgt",
                        ComparisonType::Ge => "sge",
                        ComparisonType::Lt => "slt",
                        ComparisonType::Le => "sle",
                    },
                    ln.name,
                    rn.name
                );

                Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                }
            }
            Node::And(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = and i1 {}, {}", reg, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                }
            }
            Node::Or(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = or i1 {}, {}", reg, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: Type::Int,
                }
            }
            Node::If(l, body, ebody) => {
                let ln = self.generate_node(l);
                let label = self.new_label();

                let else_label = if ebody.is_some() {
                    format!("elseif_{label}")
                } else {
                    format!("else_{label}")
                };

                println!(
                    "  br i1 {}, label %if_{label}, label %{else_label}",
                    ln.name
                );

                println!("  if_{label}:");
                self.generate_node(body);
                println!("  br label %else_{label}");

                if let Some(ebody) = ebody {
                    println!("  elseif_{label}:");
                    self.generate_node(ebody);
                    println!("  br label %else_{label}");
                }

                println!("  else_{label}:");

                Value {
                    name: String::new(),
                    ty: Type::Int,
                }
            }
            Node::Block(body) => {
                for node in body {
                    self.generate_node(node);
                }

                Value {
                    name: String::new(),
                    ty: Type::Int,
                }
            }
        }
    }
}
