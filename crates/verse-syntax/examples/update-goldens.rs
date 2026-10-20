//! Explicit developer action. Tests never update expected output.
use std::{fs, path::PathBuf};
use verse_syntax::{Diagnostic, FileId, SourceFile, Span, lex, lexer::decode, parse};
fn main() {
    assert!(
        std::env::args().any(|a| a == "--bless"),
        "pass --bless after reviewing the diagnostic definitions"
    );
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/diagnostics");
    let cases: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("cases.json")).unwrap()).unwrap();
    for case in cases.as_array().unwrap() {
        let code = case["code"].as_str().unwrap();
        let text = case["source"].as_str().unwrap_or("�");
        let diagnostics = if let Some(stage) = case["internal"].as_str() {
            vec![Diagnostic::new(
                "V9901",
                Span::default(),
                [("stage", stage.into())],
            )]
        } else if let Some(command) = case["command"].as_str() {
            vec![Diagnostic::new(
                "V1101",
                Span::new(FileId(0), 0, command.len()),
                [("word", command.into())],
            )]
        } else if let Some(bytes) = case["bytes"].as_array() {
            vec![
                *decode(
                    &bytes
                        .iter()
                        .map(|v| v.as_u64().unwrap() as u8)
                        .collect::<Vec<_>>(),
                    FileId(0),
                )
                .unwrap_err(),
            ]
        } else {
            parse(text, FileId(0)).diagnostics
        };
        let d = diagnostics
            .iter()
            .find(|d| d.code == code)
            .unwrap_or_else(|| panic!("{code}: not emitted: {diagnostics:?}"));
        fs::write(
            root.join(format!("{code}.txt")),
            d.render(&SourceFile::new(FileId(0), "example.vs", text)),
        )
        .unwrap();
    }
    for entry in fs::read_dir(root.join("../syntax")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "vs") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let tokens = lex(&source, FileId(0))
            .tokens
            .iter()
            .map(|t| {
                format!(
                    "{:?} {}..{} {:?}\n",
                    t.kind,
                    t.span.start,
                    t.span.end,
                    t.text(&source)
                )
            })
            .collect::<String>();
        fs::write(path.with_extension("tokens"), tokens).unwrap();
        fs::write(
            path.with_extension("cst"),
            format!("{:#?}\n", parse(&source, FileId(0)).syntax()),
        )
        .unwrap();
    }
}
