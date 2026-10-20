use std::{collections::BTreeSet, fs, path::PathBuf};
use verse_syntax::{
    Diagnostic, FileId, SourceFile, Span,
    diagnostic::{DEFINITIONS, Fact},
    explain,
    lexer::decode,
    parse,
};

#[test]
fn every_syntax_code_has_an_exact_golden_and_explanation() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/diagnostics");
    let cases: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(root.join("cases.json")).unwrap()).unwrap();
    let mut covered = BTreeSet::new();
    for case in cases.as_array().unwrap() {
        let code = case["code"].as_str().unwrap();
        covered.insert(code);
        let source = case["source"].as_str().unwrap_or("�");
        let file = SourceFile::new(FileId(0), "example.vs", source);
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
            parse(source, FileId(0)).diagnostics
        };
        let diagnostic = diagnostics
            .iter()
            .find(|d| d.code == code)
            .unwrap_or_else(|| panic!("{code}: not emitted: {diagnostics:?}"));
        let rendered = diagnostic.render(&file);
        assert_eq!(
            rendered,
            fs::read_to_string(root.join(format!("{code}.txt"))).unwrap(),
            "{code}"
        );
        let definition = DEFINITIONS.iter().find(|d| d.code == code).unwrap();
        let templates = format!(
            "{} {} {}",
            definition.message, definition.label, definition.help
        );
        for (key, fact) in &diagnostic.facts {
            assert!(
                templates.contains(&format!("{{{key}}}")),
                "unused fact {code}.{key}"
            );
            if matches!(key.as_str(), "n" | "m" | "k") {
                assert!(matches!(fact, Fact::Number(_)));
            }
        }
        for template in [definition.message, definition.label, definition.help] {
            let mut remaining = template;
            while let Some(start) = remaining.find('{') {
                remaining = &remaining[start + 1..];
                let Some(end) = remaining.find('}') else {
                    break;
                };
                let key = &remaining[..end];
                if !key.is_empty() && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                    assert!(
                        diagnostic.facts.contains_key(key),
                        "missing fact {code}.{key}"
                    );
                }
                remaining = &remaining[end + 1..];
            }
        }
        let text = explain(code).unwrap();
        assert!(text.lines().count() <= 25);
        for heading in [
            "What happened",
            "Why Verse works this way",
            "Ways to fix it",
            "Example",
        ] {
            assert!(text.contains(heading));
        }
        assert!(
            parse(definition.example, FileId(0)).is_valid(),
            "{code} explain example does not parse"
        );
        for help in &diagnostic.help {
            if help.edits.is_empty() {
                continue;
            }
            let mut fixed = source.to_owned();
            let mut edits = help.edits.clone();
            edits.sort_by_key(|e| std::cmp::Reverse(e.span.start));
            for edit in edits {
                assert_eq!(edit.span.file_id, FileId(0));
                fixed.replace_range(
                    edit.span.start as usize..edit.span.end as usize,
                    &edit.replacement,
                );
            }
            assert!(
                parse(&fixed, FileId(0)).is_valid(),
                "{code}: edit does not fix syntax: {fixed:?}"
            );
        }
    }
    assert_eq!(covered, DEFINITIONS.iter().map(|d| d.code).collect());
}

#[test]
fn fact_values_are_not_reinterpreted_as_templates() {
    let diagnostic = Diagnostic::new(
        "V0201",
        Span::default(),
        [("found", "{what}".into()), ("what", "an expression".into())],
    );
    assert_eq!(diagnostic.primary.label, "found {what}");
}
