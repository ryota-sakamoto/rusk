use std::env::args;

mod ast;
mod code;
mod token;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        panic!("args should be specified.");
    }

    let p = &args[1];

    let tokens = token::tokenize(p);
    let mut parser = ast::Parser::new(&tokens);
    let program = parser.program();
    code::generate(&program);
}

#[cfg(test)]
mod tests {
    use std::{env, fs, process::Command};

    fn run_and_assert(input: &str, expected: i32) {
        let dir = env::temp_dir();

        let mut ll_path = dir.clone();
        ll_path.push("test.ll");

        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", input])
            .output()
            .expect("Failed to execute compiler");

        assert!(output.status.success());

        let ll_code = String::from_utf8(output.stdout).unwrap();
        fs::write(&ll_path, ll_code).expect("Failed to write .ll file");

        let mut exe_path = dir.clone();
        exe_path.push("test.out");

        let clang_status = Command::new("clang")
            .arg(&ll_path)
            .arg("-o")
            .arg(&exe_path)
            .status()
            .expect("Failed to run clang");
        assert!(clang_status.success());

        let run_output = Command::new(&exe_path)
            .output()
            .expect("Failed to run compiled binary");

        assert!(run_output.status.success());

        let run_stdout = String::from_utf8(run_output.stdout).unwrap();
        let a: i32 = run_stdout.parse().unwrap();
        assert_eq!(a, expected);
    }

    #[test]
    fn test_return_numbers() {
        run_and_assert("fn main() { return 0; }", 0);
        run_and_assert("fn main() { return 42; }", 42);
        run_and_assert("fn main() { return 12+5-1; }", 16);
        run_and_assert("fn main() { return 33*4+8; }", 140);
        run_and_assert("fn main() { return 28+4*8-12/2; }", 54);
        run_and_assert("fn main() { return 12*(4+3)-3; }", 81);
        run_and_assert("fn main() { return 22*-5+49; }", -61);
    }
}
