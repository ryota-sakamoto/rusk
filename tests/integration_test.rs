#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        process::{Command, Stdio},
    };

    fn run_and_assert(path: &str, expected: i32) {
        let dir = env::temp_dir();

        let mut ll_path = dir.clone();
        let file_name = path.replace("/", "_");
        ll_path.push(format!("{file_name}.ll"));

        let output = Command::new("cargo")
            .args(&["run", "--quiet", "--", path])
            .stderr(Stdio::inherit())
            .output()
            .expect("Failed to execute compiler");

        assert!(output.status.success());

        let ll_code = String::from_utf8(output.stdout).unwrap();
        fs::write(&ll_path, ll_code).expect("Failed to write .ll file");

        let mut exe_path = dir.clone();
        exe_path.push(format!("{file_name}.out"));

        let clang_status = Command::new("clang")
            .arg("-Wno-override-module")
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

    include!("generated/tests.rs");
}
