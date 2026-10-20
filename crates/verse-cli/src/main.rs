use std::{
    env, fs,
    io::Write,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
    process::ExitCode,
};
use verse_syntax::{Diagnostic, FileId, SourceFile, Span, lexer::decode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args().skip(1).collect();
    let json = args.iter().any(|a| a == "--json");
    // A panic is a compiler bug, not a source error. Report it as V9901 and
    // leave the reserved status 3. Stack overflows are prevented by the parser
    // and formatter rather than caught here.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = catch_unwind(AssertUnwindSafe(|| run(args)));
    std::panic::set_hook(hook);
    match result {
        Ok(Ok(code)) => ExitCode::from(code),
        Ok(Err(message)) => {
            if json {
                println!(
                    "{}",
                    serde_json::json!({"kind": "command-error", "message": message})
                );
            } else {
                eprintln!("{message}");
            }
            ExitCode::from(2)
        }
        Err(_) => {
            let source = SourceFile::new(FileId(0), "command line", "");
            let diagnostic =
                Diagnostic::new("V9901", Span::default(), [("stage", "command".into())]);
            if json {
                println!("{}", diagnostic_json(&diagnostic, &source));
            } else {
                eprint!("{}", diagnostic.render(&source));
            }
            ExitCode::from(3)
        }
    }
}

fn run(mut args: Vec<String>) -> Result<u8, String> {
    let json = args.iter().any(|a| a == "--json");
    args.retain(|a| a != "--json");
    if args.is_empty() || args == ["--help"] || args == ["-h"] {
        if json {
            println!(
                "{}",
                serde_json::json!({"version": env!("CARGO_PKG_VERSION"), "milestone": "M0", "commands": ["fmt", "explain", "bench"]})
            );
        } else {
            println!(
                "Verse 0.1 — M0 foundations\n\nCommands:\n  fmt [path] [--check]    Format Verse source\n  explain <code>          Explain a diagnostic\n  bench                   Run compiler benchmarks\n\nExecution and semantic checking are not available in M0."
            );
        }
        return Ok(0);
    }
    if args == ["--version"] {
        if json {
            println!(
                "{}",
                serde_json::json!({"version": env!("CARGO_PKG_VERSION")})
            );
        } else {
            println!("verse {}", env!("CARGO_PKG_VERSION"));
        }
        return Ok(0);
    }
    match args[0].as_str() {
        "explain" => {
            if args.len() != 2 {
                return Err("usage: verse explain <code>".into());
            }
            let text = verse_syntax::explain(&args[1])
                .ok_or_else(|| format!("no implemented explanation for {}", args[1]))?;
            if json {
                println!(
                    "{}",
                    serde_json::json!({"code": args[1], "explanation": text})
                );
            } else {
                print!("{text}");
            }
            Ok(0)
        }
        "fmt" => {
            let check = args.iter().any(|a| a == "--check");
            let paths: Vec<_> = args[1..]
                .iter()
                .filter(|a| a.as_str() != "--check")
                .collect();
            if paths.len() > 1 || paths.first().is_some_and(|p| p.starts_with('-')) {
                return Err("usage: verse fmt [path] [--check] [--json]".into());
            }
            let root = Path::new(paths.first().map_or(".", |p| p.as_str()));
            let mut files = vec![];
            collect_sources(root, &mut files).map_err(|e| format!("{}: {e}", root.display()))?;
            files.sort();
            let mut failed = false;
            let mut internal_failure = false;
            // Validate every input before writing any of them.
            let mut changes = vec![];
            for (index, path) in files.iter().enumerate() {
                let bytes = fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
                let id = FileId(index as u32);
                let text = match decode(&bytes, id) {
                    Ok(text) => text,
                    Err(d) => {
                        failed = true;
                        let source = SourceFile::new(
                            id,
                            path.display().to_string(),
                            String::from_utf8_lossy(&bytes).into_owned(),
                        );
                        if json {
                            println!("{}", diagnostic_json(&d, &source));
                        } else {
                            eprint!("{}", d.render(&source));
                        }
                        continue;
                    }
                };
                match verse_fmt::format(text, id) {
                    Ok(formatted) => {
                        if check {
                            let source = SourceFile::new(id, path.display().to_string(), text);
                            let parsed = verse_syntax::parse(text, id);
                            for d in parsed
                                .diagnostics
                                .iter()
                                .filter(|d| matches!(d.code.as_str(), "V0101" | "V0102"))
                            {
                                if json {
                                    println!("{}", diagnostic_json(d, &source));
                                } else {
                                    eprint!("{}", d.render(&source));
                                }
                            }
                            for d in verse_syntax::diagnostic::naming_diagnostics(text, id) {
                                failed = true;
                                if json {
                                    println!("{}", diagnostic_json(&d, &source));
                                } else {
                                    eprint!("{}", d.render(&source));
                                }
                            }
                        }
                        if formatted != text {
                            changes.push((path, formatted));
                        }
                    }
                    Err(diagnostics) => {
                        failed = true;
                        internal_failure |= diagnostics.iter().any(|d| d.code.starts_with("V99"));
                        let source = SourceFile::new(id, path.display().to_string(), text);
                        for d in diagnostics {
                            if json {
                                println!("{}", diagnostic_json(&d, &source));
                            } else {
                                eprint!("{}", d.render(&source));
                            }
                        }
                    }
                }
            }
            if failed {
                return Ok(if internal_failure { 3 } else { 2 });
            }
            for (path, formatted) in &changes {
                if !check {
                    write_atomic(path, formatted)
                        .map_err(|e| format!("{}: {e}", path.display()))?;
                }
                if json {
                    println!(
                        "{}",
                        serde_json::json!({"path": path, "changed": true, "written": !check})
                    );
                } else if check {
                    println!("would format {}", path.display());
                }
            }
            Ok(u8::from(check && !changes.is_empty()))
        }
        "bench" => {
            let script = bench_script().ok_or_else(|| {
                "bench/compiler/run.py was not found; run from the Verse repository".to_owned()
            })?;
            let mut command = std::process::Command::new("python3");
            command.arg(script).args(&args[1..]);
            if json {
                command.arg("--json");
            }
            let status = command.status().map_err(|e| e.to_string())?;
            Ok(status.code().unwrap_or(2).clamp(0, 255) as u8)
        }
        "check" | "build" | "test" | "debug" | "lsp" => {
            Err(format!("`verse {}` is not implemented in M0", args[0]))
        }
        word if word.ends_with(".vs") || word.contains('/') || word == "." => {
            Err("running Verse programs is not implemented in M0".into())
        }
        word => {
            let source = SourceFile::new(FileId(0), "command line", word);
            let diagnostic = Diagnostic::new(
                "V1101",
                Span::new(FileId(0), 0, word.len()),
                [("word", word.into())],
            );
            if json {
                println!("{}", diagnostic_json(&diagnostic, &source));
            } else {
                eprint!("{}", diagnostic.render(&source));
            }
            Ok(2)
        }
    }
}

fn diagnostic_json(diagnostic: &Diagnostic, source: &SourceFile) -> serde_json::Value {
    let mut value = serde_json::to_value(diagnostic).unwrap();
    let (line, column) = source.position(diagnostic.primary.span.start);
    value["primary"]["position"] =
        serde_json::json!({"file": source.name, "line": line, "column": column});
    value
}

fn collect_sources(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if path.is_dir() {
        collect_directory(path, files)
    } else if path.extension().is_some_and(|s| s == "vs") {
        files.push(path.to_owned());
        Ok(())
    } else if !path.exists() {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "path does not exist",
        ))
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "not a Verse source file (expected a .vs path or a directory)",
        ))
    }
}

fn collect_directory(path: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_symlink() {
            continue;
        }
        if entry
            .file_name()
            .to_str()
            .is_some_and(|s| s.starts_with('.') || matches!(s, "target" | "vendor" | "dist"))
        {
            continue;
        }
        let path = entry.path();
        if path.is_dir() {
            collect_directory(&path, files)?;
        } else if path.extension().is_some_and(|s| s == "vs") {
            files.push(path);
        }
    }
    Ok(())
}

/// Write beside the target and rename, so an interrupted format can never
/// leave a half-written source file.
fn write_atomic(path: &Path, text: &str) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let name = path
        .file_name()
        .map_or_else(|| "source.vs".into(), |name| name.to_string_lossy());
    let temporary = parent.join(format!(".{name}.{}.tmp", std::process::id()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        if let Ok(metadata) = fs::metadata(path) {
            fs::set_permissions(&temporary, metadata.permissions())?;
        }
        fs::rename(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn bench_script() -> Option<PathBuf> {
    if let Ok(root) = env::var("VERSE_ROOT") {
        let candidate = PathBuf::from(root).join("bench/compiler/run.py");
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    let mut directory = env::current_dir().ok()?;
    loop {
        let candidate = directory.join("bench/compiler/run.py");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !directory.pop() {
            break;
        }
    }
    let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../bench/compiler/run.py");
    candidate.is_file().then_some(candidate)
}
