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
    println!("define i32 @{}() {{", function.name);

    let mut index = 1;
    for node in function.body.iter() {
        index = generate_node(node, index) + 1;
    }
    println!("}}");
}

fn generate_node(node: &Node, index: u64) -> u64 {
    match node {
        Node::ADD(l, r) => {
            let ln = generate_node(l, index);
            let rn = generate_node(r, ln + 1);
            println!("  %{} = add i32 %{}, %{}", rn + 1, ln, rn);

            return rn + 1;
        }
        Node::SUB(l, r) => {
            let ln = generate_node(l, index);
            let rn = generate_node(r, ln + 1);
            println!("  %{} = sub i32 %{}, %{}", rn + 1, ln, rn);

            return rn + 1;
        }
        Node::MUL(l, r) => {
            let ln = generate_node(l, index);
            let rn = generate_node(r, ln + 1);
            println!("  %{} = mul i32 %{}, %{}", rn + 1, ln, rn);

            return rn + 1;
        }
        Node::DIV(l, r) => {
            let ln = generate_node(l, index);
            let rn = generate_node(r, ln + 1);
            println!("  %{} = sdiv i32 %{}, %{}", rn + 1, ln, rn);

            return rn + 1;
        }
        Node::NUM(n) => {
            println!("  %{} = alloca i32", index);
            println!("  store i32 {}, ptr %{}", n, index);
            println!("  %{} = load i32, ptr %{}", index + 1, index);
            return index + 1;
        }
        Node::RET(n) => {
            let ret = generate_node(n, index);
            println!("  ret i32 %{}", ret);
            return 0;
        }
        Node::CALL(name, args) => {
            let mut call_args = Vec::new();

            let mut next_index = index;
            for arg in args {
                let ret = generate_node(arg, next_index);
                call_args.push(ret);
                next_index = ret + 1;
            }

            if call_args.len() > 0 {
                println!(
                    "  %{} = call i32 @{}(ptr @.str, i32 %{})",
                    next_index, name, call_args[0]
                );
            } else {
                println!("  %{} = call i32 @{}()", next_index, name);
            }

            return next_index;
        }
    }
}
