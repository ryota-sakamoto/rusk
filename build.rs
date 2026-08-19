use std::{fs, path::Path};

fn main() {
    println!("cargo:rerun-if-changed=tests/cases");

    let dest_dir = Path::new("tests/generated");
    fs::create_dir_all(dest_dir).unwrap();

    let dest = dest_dir.join("tests.rs");
    let cases_dir = Path::new("tests/cases");

    let mut entries = Vec::new();
    collect_rs_files(cases_dir, &mut entries);
    entries.sort();

    let mut code = String::new();
    for path in entries {
        let content = fs::read_to_string(&path).unwrap();

        let first_line = content.lines().next().unwrap();
        let expected: Option<Result<i32, _>> =
            first_line.strip_prefix("// EXPECTED: ").map(|v| v.parse());

        if let Some(Ok(expected)) = expected {
            let path_str = path.to_str().unwrap();
            let test_name = path_str
                .strip_prefix("tests/cases/")
                .unwrap()
                .replace('/', "_")
                .replace(".rs", "");

            code.push_str(&format!(
                "#[test]\nfn test_{}() {{\n    run_and_assert(\"{}\", {expected});\n}}\n\n",
                test_name,
                path.to_str().unwrap(),
            ));
        }
    }

    fs::write(dest, code).unwrap();
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
