use core::panic;
use std::collections::HashMap;

use crate::ast::ComparisonType;
use crate::hir::{Function, Node, Program, Type};

pub fn generate(program: &Program) {
    let mut generator = Generator::new(program);
    generator.generate();
}

struct Generator<'a> {
    program: &'a Program,
}

impl<'a> Generator<'a> {
    fn new(program: &'a Program) -> Self {
        Self { program }
    }

    fn generate(&mut self) {
        for (k, v) in self.program.strings.iter().enumerate() {
            println!(
                r#"@.str.{} = private unnamed_addr constant [{} x i8] c"{}\00""#,
                k,
                v.len() + 1,
                v,
            );
        }

        for (k, v) in &self.program.struct_map {
            println!(
                "%{} = type {{{}}}",
                k,
                v.values()
                    .map(|field| format!("{}", field.ty))
                    .collect::<Vec<_>>()
                    .join(",")
            );
        }

        println!("declare i32 @printf(ptr, ...)");

        for f in self.program.functions.iter() {
            println!();
            let generator = GenerateFunction::new(f, &self.program.enum_map);
            generator.generate();
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
    enum_map: &'a HashMap<String, HashMap<String, usize>>,
    has_return: bool,
    terminated: bool,
    next_start_label: Option<String>,
    next_end_label: Option<String>,
}

impl<'a> GenerateFunction<'a> {
    fn new(function: &'a Function, enum_map: &'a HashMap<String, HashMap<String, usize>>) -> Self {
        Self {
            function,
            index: 0,
            label: 0,
            map: HashMap::new(),
            enum_map,
            has_return: false,
            terminated: false,
            next_start_label: None,
            next_end_label: None,
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
            let ty: Type = arg.ty.parse().unwrap();
            regs.push(format!("{ty} %r{arg_reg}"));

            let reg = self.new_reg();
            entry_instructions.push(format!("  %r{reg} = alloca {ty}"));
            entry_instructions.push(format!("  store {ty} %r{arg_reg}, ptr %r{reg}"));
            self.map.insert(
                &arg.name,
                Value {
                    name: format!("%r{reg}"),
                    ty: Type::Ptr(Box::new(ty)),
                },
            );
        }

        println!(
            "define {} @\"{}\"({}) {{",
            if self.is_main() {
                "i32"
            } else {
                self.function.ty.as_str()
            },
            self.function.full_name(),
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
            Node::Add(l, r, ty) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = add {} {}, {}", reg, ty, ln.name, rn.name);

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
            Node::String(id) => Value {
                name: format!("@.str.{}", id),
                ty: Type::Ptr(Box::new(Type::Int)),
            },
            Node::Bool(b) => Value {
                name: (*b as i32).to_string(),
                ty: Type::Bool,
            },
            Node::Ret(n) => {
                self.has_return = true;
                let ret = self.generate_node(n);
                self.terminated = true;
                println!("  ret {} {}", ret.ty, ret.name);
                Value {
                    name: String::new(),
                    ty: ret.ty,
                }
            }
            Node::Call(name, args, ty) => {
                let mut call_args = Vec::new();

                for arg in args {
                    let ret = self.generate_node(arg);
                    call_args.push(ret);
                }

                let reg = self.new_reg();
                if !call_args.is_empty() {
                    println!(
                        "  %r{} = call {} @\"{}\"({})",
                        reg,
                        ty,
                        name,
                        call_args
                            .iter()
                            .map(|reg| format!("{} {}", reg.ty, reg.name))
                            .collect::<Vec<String>>()
                            .join(", ")
                    );
                } else {
                    println!("  %r{} = call {} @\"{}\"()", reg, ty, name);
                }

                Value {
                    name: format!("%r{reg}"),
                    ty: ty.clone(),
                }
            }
            Node::Let(name, ty, right, _) => {
                let r = self.generate_node(right);

                let reg = self.new_reg();
                println!("  %r{reg} = alloca {ty}");
                println!("  store {ty} {}, ptr %r{}", r.name, reg);

                self.map.insert(
                    name,
                    Value {
                        name: format!("%r{reg}"),
                        ty: Type::Ptr(Box::new(ty.clone())),
                    },
                );

                r
            }
            Node::RLet(name, ty) => {
                let reg = self.new_reg();
                let r = self.map.get(name.as_str()).unwrap();

                println!("  %r{reg} = load {}, ptr {}", ty, r.name);
                Value {
                    name: format!("%r{reg}"),
                    ty: ty.clone(),
                }
            }
            Node::FieldAccess(node, index, ty) => {
                let r = self.generate_node(node);
                let reg = self.new_reg();

                println!("  %r{reg} = extractvalue {} {}, {}", r.ty, r.name, index);

                Value {
                    name: format!("%r{reg}"),
                    ty: ty.clone(),
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
            Node::And(l, r, ty) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = and {} {}, {}", reg, ty, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: ty.clone(),
                }
            }
            Node::Or(l, r, ty) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = or {} {}, {}", reg, ty, ln.name, rn.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: ty.clone(),
                }
            }
            Node::If(l, body, ebody) => {
                let ln = self.generate_node(l);
                let label = self.new_label();

                if ebody.is_some() {
                    println!(
                        "  br i1 {}, label %if_{label}, label %else_{label}",
                        ln.name
                    );
                } else {
                    println!(
                        "  br i1 {}, label %if_{label}, label %ifafter_{label}",
                        ln.name
                    );
                }

                self.terminated = false;
                println!("if_{label}:");
                self.generate_node(body);

                if !self.terminated {
                    println!("  br label %ifafter_{label}");
                }

                self.terminated = false;
                if let Some(ebody) = ebody {
                    println!("else_{label}:");
                    self.generate_node(ebody);
                    if !self.terminated {
                        println!("  br label %ifafter_{label}");
                    }
                }

                if !self.terminated {
                    println!("ifafter_{label}:");
                }

                Value {
                    name: String::new(),
                    ty: Type::Int,
                }
            }
            Node::While(l, body) => {
                println!("  ; while");
                let label = self.new_label();
                let cond_label = format!("cond_{label}");
                let while_label = format!("while_{label}");
                let whileend_label = format!("whileend_{label}");
                println!("  br label %{cond_label}");

                println!("{cond_label}:");
                let ln = self.generate_node(l);
                println!(
                    "  br i1 {}, label %{while_label}, label%{whileend_label}",
                    ln.name
                );

                println!("{while_label}:");
                self.next_start_label = Some(cond_label.clone());
                self.next_end_label = Some(whileend_label);
                self.generate_node(body);
                if !self.terminated {
                    println!("  br label %{cond_label}");
                }

                println!("whileend_{label}:");

                Value {
                    name: String::new(),
                    ty: Type::Int,
                }
            }
            Node::Break => {
                self.terminated = true;
                let label = self.next_end_label.clone().unwrap();
                println!("  br label %{label}");

                Value {
                    name: String::new(),
                    ty: Type::Int,
                }
            }
            Node::Continue => {
                self.terminated = true;
                let label = self.next_start_label.clone().unwrap();
                println!("  br label %{label}");

                Value {
                    name: String::new(),
                    ty: Type::Int,
                }
            }
            Node::Not(node) => {
                let ln = self.generate_node(node);
                let reg = self.new_reg();
                println!("  %r{} = xor i1 {}, 1", reg, ln.name);

                Value {
                    name: format!("%r{reg}"),
                    ty: Type::Bool,
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
            Node::Struct(name, args) => {
                let reg = self.new_reg();
                println!("  %r{reg} = alloca %{name}");

                for (index, n) in args {
                    let r = self.generate_node(n);
                    let field_reg = self.new_reg();
                    println!(
                        "  %r{field_reg} = getelementptr %{name}, ptr %r{reg}, i32 0, i32 {}",
                        index,
                    );
                    println!("  store {} {}, ptr %r{field_reg}", r.ty, r.name);
                }

                let val_reg = self.new_reg();
                println!("  %r{val_reg} = load %{name}, ptr %r{reg}");

                Value {
                    name: format!("%r{val_reg}"),
                    ty: Type::Struct(name.clone()),
                }
            }
            Node::Enum(name, variant) => {
                let variant_index = self.enum_map[name.as_str()][variant.as_str()];
                Value {
                    name: variant_index.to_string(),
                    ty: Type::Int,
                }
            }
            Node::Match(l, r) => {
                let ln = self.generate_node(l);

                let label = self.new_label();
                let switch_label = format!("switch_{label}_default");

                let mut switch_labels = Vec::new();
                for (index, (cond, _)) in r.iter().enumerate() {
                    let cond_value = self.generate_node(cond);

                    let switch_label = format!("switch_{label}_{index}");
                    switch_labels.push(format!(
                        "{} {}, label %{switch_label}",
                        cond_value.ty, cond_value.name
                    ));
                }

                println!(
                    "  switch {} {}, label %{switch_label} [{}]",
                    ln.ty,
                    ln.name,
                    switch_labels.join(" ")
                );

                println!("{switch_label}:");
                println!("  unreachable");

                for (index, (_, block)) in r.iter().enumerate() {
                    let switch_label = format!("switch_{label}_{index}");
                    println!("{switch_label}:");

                    self.generate_node(block);
                    println!("  br label %switch_{label}_after");
                }

                println!("switch_{label}_after:");

                // noop
                Value {
                    name: format!("testtest"),
                    ty: Type::Int,
                }
            }
            Node::Array(data, ty) => {
                let reg = self.new_reg();
                let array_ty = Type::Array(Box::new(ty.clone()), data.len());
                println!("  %r{reg} = alloca {}", array_ty);

                for (index, v) in data.iter().enumerate() {
                    let node = self.generate_node(v);
                    let field_reg = self.new_reg();

                    println!(
                        "  %r{field_reg} = getelementptr {}, ptr %r{reg}, i32 0, i32 {}",
                        array_ty, index,
                    );
                    println!("  store {} {}, ptr %r{field_reg}", node.ty, node.name);
                }

                let value_reg = self.new_reg();
                println!("  %r{value_reg} = load [{} x i32], ptr %r{reg}", data.len());

                Value {
                    name: format!("%r{value_reg}"),
                    ty: Type::Array(Box::new(Type::Int), data.len()),
                }
            }
            Node::ArrayAccess(node, index, ty) => {
                let array = self.generate_node(node);
                let index = self.generate_node(index);

                let reg = self.new_reg();
                println!(
                    "  %r{reg} = extractvalue {} {}, {}",
                    array.ty, array.name, index.name,
                );

                Value {
                    name: format!("%r{reg}"),
                    ty: ty.clone(),
                }
            }
        }
    }
}
