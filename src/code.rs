use crate::ast::{Node, Program};

pub fn generate(program: &Program) {
    println!(r#"@.str = private unnamed_addr constant [3 x i8] c"%d\00""#);
    println!("declare i32 @printf(ptr, ...)");
    println!("define i32 @main(i32, i8**) {{");

    let ret = generate_node(&program.functions[0].body[0], 3);

    println!("  call i32 (ptr, ...) @printf(ptr @.str, i32 %{})", ret);
    println!("  ret i32 0");
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
    }
}
