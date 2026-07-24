use std::collections::HashMap;

use crate::ast::{Function, Node, Program};

pub fn generate(program: &Program) {
    println!(r#"@.str = private unnamed_addr constant [3 x i8] c"%d\00""#);
    println!("declare i32 @printf(ptr, ...)");
    println!();

    for f in program.functions.iter() {
        generate_function(f);
    }
}

fn generate_function(function: &Function) {
    let generator = GenerateFunction::new(function);
    generator.generate();
}

struct GenerateFunction<'a> {
    function: &'a Function,
    index: u64,
    map: HashMap<&'a str, u64>,
}

impl<'a> GenerateFunction<'a> {
    fn new(function: &'a Function) -> Self {
        Self {
            function,
            index: 0,
            map: HashMap::new(),
        }
    }

    fn generate(mut self) {
        let mut regs = Vec::new();
        for arg in self.function.args.iter() {
            let reg = self.new_reg();
            self.map.insert(arg, reg);
            regs.push(reg);
        }

        if regs.len() > 0 {
            println!(
                "define i32 @{}({}) {{",
                self.function.name,
                regs.iter()
                    .map(|reg| format!("i32 %{}", reg))
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        } else {
            println!("define i32 @{}() {{", self.function.name);
        }
        println!("entry:");

        for node in self.function.body.iter() {
            self.generate_node(node);
        }
        println!("}}");
    }

    fn new_reg(&mut self) -> u64 {
        let current = self.index;
        self.index += 1;

        return current;
    }

    fn generate_node(&mut self, node: &'a Node) -> u64 {
        match node {
            Node::ADD(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %{} = add i32 %{}, %{}", reg, ln, rn);

                return reg;
            }
            Node::SUB(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %{} = sub i32 %{}, %{}", reg, ln, rn);

                return reg;
            }
            Node::MUL(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %{} = mul i32 %{}, %{}", reg, ln, rn);

                return reg;
            }
            Node::DIV(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %{} = sdiv i32 %{}, %{}", reg, ln, rn);

                return reg;
            }
            Node::NUM(n) => {
                let reg = self.new_reg();
                println!("  %{} = alloca i32", reg);
                println!("  store i32 {}, ptr %{}", n, reg);

                let reg2 = self.new_reg();
                println!("  %{} = load i32, ptr %{}", reg2, reg);

                return reg2;
            }
            Node::RET(n) => {
                let ret = self.generate_node(n);
                println!("  ret i32 %{}", ret);
                return 0;
            }
            Node::CALL(name, args) => {
                let mut call_args = Vec::new();

                for arg in args {
                    let ret = self.generate_node(arg);
                    call_args.push(ret);
                }

                let reg = self.new_reg();
                if call_args.len() > 0 {
                    // TODO: fix after implement string
                    if name == "printf" {
                        println!(
                            "  %{} = call i32 @{}(ptr @.str, i32 %{})",
                            reg, name, call_args[0]
                        );
                    } else {
                        println!(
                            "  %{} = call i32 @{}({})",
                            reg,
                            name,
                            call_args
                                .iter()
                                .map(|reg| format!("i32 %{}", reg))
                                .collect::<Vec<String>>()
                                .join(", ")
                        );
                    }
                } else {
                    println!("  %{} = call i32 @{}()", reg, name);
                }

                return reg;
            }
            Node::LET(name, right) => {
                let r = self.generate_node(right);
                self.map.insert(name, r);

                return r;
            }
            Node::RLET(name) => {
                let r = self.map.get(name.as_str()).unwrap();
                return *r;
            }
        }
    }
}
