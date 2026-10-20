use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base = if Path::new("/tmp/opencode").is_dir() {
            PathBuf::from("/tmp/opencode")
        } else {
            std::env::temp_dir()
        };
        let path = base.join(format!(
            "verse-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_verse"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn fmt_check_does_not_write_and_format_is_idempotent() {
    let temp = Scratch::new();
    let path = temp.0.join("example.vs");
    fs::write(&path, "x=1+2\n").unwrap();
    let path = path.to_str().unwrap();
    assert_eq!(run(&["fmt", path, "--check"]).status.code(), Some(1));
    assert_eq!(fs::read_to_string(path).unwrap(), "x=1+2\n");
    assert!(run(&["fmt", path]).status.success());
    assert_eq!(fs::read_to_string(path).unwrap(), "x = 1 + 2\n");
    assert!(run(&["fmt", "--check", path]).status.success());
}

#[test]
fn syntax_errors_are_structured_and_never_overwrite_source() {
    let temp = Scratch::new();
    let path = temp.0.join("bad.vs");
    fs::write(&path, "x = {}\n").unwrap();
    let output = run(&["fmt", path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let d: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(d["code"], "V0210");
    assert!(d["primary"]["span"].is_object());
    assert_eq!(fs::read_to_string(path).unwrap(), "x = {}\n");
}

#[test]
fn folder_validation_precedes_any_write() {
    let temp = Scratch::new();
    fs::write(temp.0.join("a.vs"), "x=1\n").unwrap();
    fs::write(temp.0.join("b.vs"), "x = )\n").unwrap();
    assert_eq!(
        run(&["fmt", temp.0.to_str().unwrap()]).status.code(),
        Some(2)
    );
    assert_eq!(fs::read_to_string(temp.0.join("a.vs")).unwrap(), "x=1\n");
}

#[test]
fn malformed_utf8_does_not_panic() {
    let temp = Scratch::new();
    let path = temp.0.join("bad.vs");
    fs::write(&path, [255]).unwrap();
    let output = run(&["fmt", path.to_str().unwrap(), "--json"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout).unwrap().contains("V0109"));
}

#[test]
fn explain_and_unknown_command() {
    let output = run(&["explain", "V0101"]);
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("What happened")
    );
    let output = run(&["unknown", "--json"]);
    assert_eq!(output.status.code(), Some(2));
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["code"], "V1101");
    assert_eq!(run(&["check"]).status.code(), Some(2));
}

#[test]
fn metadata_and_usage_errors_honor_json() {
    for args in [
        vec!["--json"],
        vec!["--version", "--json"],
        vec!["explain", "--json"],
        vec!["fmt", "--bad-option", "--json"],
    ] {
        let output = run(&args);
        assert!(
            serde_json::from_slice::<serde_json::Value>(&output.stdout).is_ok(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn check_reports_fixable_indentation_and_naming() {
    let temp = Scratch::new();
    let indented = temp.0.join("indent.vs");
    fs::write(&indented, "if true\n   pass\n").unwrap();
    let output = run(&["fmt", indented.to_str().unwrap(), "--check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8(output.stderr).unwrap().contains("V0102"));
    assert_eq!(
        fs::read_to_string(&indented).unwrap(),
        "if true\n   pass\n",
        "check must not rewrite"
    );
    let named = temp.0.join("named.vs");
    fs::write(&named, "userName = 1\n").unwrap();
    let output = run(&["fmt", named.to_str().unwrap(), "--check"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stderr).unwrap().contains("V0110"));
}

#[test]
fn fmt_rejects_a_file_that_is_not_verse_source() {
    let temp = Scratch::new();
    let path = temp.0.join("notes.txt");
    fs::write(&path, "x = 1\n").unwrap();
    assert_eq!(run(&["fmt", path.to_str().unwrap()]).status.code(), Some(2));
    assert_eq!(fs::read_to_string(&path).unwrap(), "x = 1\n");
}

#[test]
fn formatting_a_directory_skips_files_that_are_not_verse_source() {
    let temp = Scratch::new();
    let source = temp.0.join("example.vs");
    let notes = temp.0.join("notes.txt");
    fs::write(&source, "x=1+2\n").unwrap();
    fs::write(&notes, "not verse\n").unwrap();
    assert!(run(&["fmt", temp.0.to_str().unwrap()]).status.success());
    assert_eq!(fs::read_to_string(&source).unwrap(), "x = 1 + 2\n");
    assert_eq!(fs::read_to_string(&notes).unwrap(), "not verse\n");
    assert!(fs::read_dir(&temp.0).unwrap().all(|entry| {
        !entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".tmp")
    }));
}
