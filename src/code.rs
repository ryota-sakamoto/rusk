use crate::ast::Node;

pub fn generate(node: &Node, index: u64) -> u64 {
    match node {
        Node::ADD(l, r) => {
            let ln = generate(l, index);
            let rn = generate(r, ln + 1);
            println!("  %{} = add i32 %{}, %{}", rn + 1, ln, rn);

            return rn + 1;
        }
        Node::SUB(l, r) => {
            let ln = generate(l, index);
            let rn = generate(r, ln + 1);
            println!("  %{} = sub i32 %{}, %{}", rn + 1, ln, rn);

            return rn + 1;
        }
        Node::MUL(l, r) => {
            let ln = generate(l, index);
            let rn = generate(r, ln + 1);
            println!("  %{} = mul i32 %{}, %{}", rn + 1, ln, rn);

            return rn + 1;
        }
        Node::DIV(l, r) => {
            let ln = generate(l, index);
            let rn = generate(r, ln + 1);
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
