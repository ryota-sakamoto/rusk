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
    label: u64,
    map: HashMap<&'a str, u64>,
}

impl<'a> GenerateFunction<'a> {
    fn new(function: &'a Function) -> Self {
        Self {
            function,
            index: 0,
            label: 0,
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
                    .map(|reg| format!("i32 %r{}", reg))
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

    fn new_label(&mut self) -> u64 {
        let current = self.label;
        self.label += 1;

        return current;
    }

    fn generate_node(&mut self, node: &'a Node) -> u64 {
        match node {
            Node::ADD(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = add i32 %r{}, %r{}", reg, ln, rn);

                return reg;
            }
            Node::SUB(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = sub i32 %r{}, %r{}", reg, ln, rn);

                return reg;
            }
            Node::MUL(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = mul i32 %r{}, %r{}", reg, ln, rn);

                return reg;
            }
            Node::DIV(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = sdiv i32 %r{}, %r{}", reg, ln, rn);

                return reg;
            }
            Node::NUM(n) => {
                let reg = self.new_reg();
                println!("  %r{} = alloca i32", reg);
                println!("  store i32 {}, ptr %r{}", n, reg);

                let reg2 = self.new_reg();
                println!("  %r{} = load i32, ptr %r{}", reg2, reg);

                return reg2;
            }
            Node::RET(n) => {
                let ret = self.generate_node(n);
                println!("  ret i32 %r{}", ret);
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
                            "  %r{} = call i32 @{}(ptr @.str, i32 %r{})",
                            reg, name, call_args[0]
                        );
                    } else {
                        println!(
                            "  %r{} = call i32 @{}({})",
                            reg,
                            name,
                            call_args
                                .iter()
                                .map(|reg| format!("i32 %r{}", reg))
                                .collect::<Vec<String>>()
                                .join(", ")
                        );
                    }
                } else {
                    println!("  %r{} = call i32 @{}()", reg, name);
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
            Node::EQ(l, r) => {
                let ln = self.generate_node(l);
                let rn = self.generate_node(r);

                let reg = self.new_reg();
                println!("  %r{} = icmp eq i32 %r{}, %r{}", reg, ln, rn);

                return reg;
            }
            Node::IF(l, body, ebody) => {
                let ln = self.generate_node(l);
                let label = self.new_label();

                let else_label = if ebody.len() > 0 {
                    format!("elseif_{label}")
                } else {
                    format!("else_{label}")
                };

                println!("  br i1 %r{}, label %if_{label}, label %{else_label}", ln);

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

                return 0;
            }
        }
    }
}
