#[cfg(test)]
mod tests {
    use std::{collections::linked_list, env, fs, path::Path, process::Command};

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
        let cases_dir = Path::new("tests/cases");
        let mut entries: Vec<_> = fs::read_dir(cases_dir)
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        entries.sort_by_key(|dir| dir.path());

        for entry in entries {
            let content = fs::read_to_string(entry.path()).unwrap();

            let first_line = content.lines().next().unwrap();
            let expected: i32 = first_line
                .strip_prefix("// EXPECTED: ")
                .unwrap()
                .parse()
                .unwrap();

            println!("run {:?}", entry.path());
            run_and_assert(&content, expected);
        }
    }
}
