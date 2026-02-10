use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn bin_path() -> String {
    env!("CARGO_BIN_EXE_treestat").to_string()
}

fn make_temp_dir() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("treestat-test-{nanos}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, content).unwrap();
}

#[test]
fn text_tree_output_for_rust() {
    let root = make_temp_dir();
    write(&root.join("src/main.rs"), "fn main(){}\n");
    write(&root.join("src/nested/lib.rs"), "pub fn a(){}\n");
    write(&root.join("tests/test.rs"), "#[test] fn t(){}\n");
    write(&root.join("README.md"), "x\n");

    let out = Command::new(bin_path())
        .arg(&root)
        .arg("--lang")
        .arg("rust")
        .arg("--count-mode")
        .arg("tree")
        .output()
        .unwrap();

    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("Total matching files: 3"));
    assert!(s.contains("src/ (2 files)"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn json_output_shape() {
    let root = make_temp_dir();
    write(&root.join("pkg/a.py"), "print(1)\n");

    let out = Command::new(bin_path())
        .arg(&root)
        .arg("--lang")
        .arg("python")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(out.status.success());
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains("\"total_files\":1"));
    assert!(s.contains("\"count_mode\":\"tree\""));
    assert!(s.contains("\"children\":"));

    let _ = fs::remove_dir_all(root);
}
