#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::Path,
        process::{Command, Stdio},
    };

    fn run_and_assert(path: &Path, expected: i32) {
        let dir = env::temp_dir();

        let mut ll_path = dir.clone();
        ll_path.push("test.ll");

        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", &path.to_string_lossy()])
            .stderr(Stdio::inherit())
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

    fn collect_rs_files(dir: &Path, files: &mut Vec<std::path::PathBuf>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_rs_files(&path, files);
                } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
    }

    #[test]
    fn test_return_numbers() {
        let cases_dir = Path::new("tests/cases");
        let mut entries = Vec::new();
        collect_rs_files(cases_dir, &mut entries);
        entries.sort();

        for path in entries {
            let content = fs::read_to_string(&path).unwrap();

            let first_line = content.lines().next().unwrap();
            let expected: i32 = first_line
                .strip_prefix("// EXPECTED: ")
                .unwrap()
                .parse()
                .unwrap();

            println!("run {:?}", path);
            run_and_assert(&path, expected);
        }
    }
}
