use std::env::args;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("args should be specified.");
    }

    let num: u64 = args[1].parse().unwrap();

    println!("define i32 @main(i32, i8**) {{");
    println!("  ret i32 {}", num);
    println!("}}");
}
