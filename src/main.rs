use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("args should be specified.");
    }

    let p = &args[1];

    println!("define i32 @main(i32, i8**) {{");
    println!("  %3 = alloca i32");
    println!("  store i32 {}, ptr %3", p);
    println!("  %4 = load i32, ptr %3");
    println!("  ret i32 %4");
    println!("}}");
}
