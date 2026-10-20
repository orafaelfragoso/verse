use std::{path::PathBuf, process::Command};
use verse_syntax::{FileId, parse, tree::fingerprint};

#[test]
fn formatter_goldens() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/format");
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        let Some(name) = path
            .file_name()
            .and_then(|p| p.to_str())
            .and_then(|s| s.strip_suffix(".input.vs"))
        else {
            continue;
        };
        let expected =
            std::fs::read_to_string(path.with_file_name(format!("{name}.expected.vs"))).unwrap();
        let source = std::fs::read_to_string(&path).unwrap();
        assert_eq!(
            verse_fmt::format(&source, FileId(0)).unwrap(),
            expected,
            "{}",
            path.display()
        );
        assert_eq!(verse_fmt::format(&expected, FileId(0)).unwrap(), expected);
    }
}

fn verify(source: &str) -> String {
    let formatted =
        verse_fmt::format(source, FileId(0)).unwrap_or_else(|d| panic!("input: {source}\n{d:#?}"));
    let parsed = parse(&formatted, FileId(0));
    assert!(
        parsed.is_valid(),
        "formatted: {formatted}\n{:#?}",
        parsed.diagnostics
    );
    assert_eq!(
        verse_fmt::format(&formatted, FileId(0)).unwrap(),
        formatted,
        "not idempotent: {source}"
    );
    assert_eq!(
        fingerprint(&parse(source, FileId(0)).syntax()),
        fingerprint(&parsed.syntax()),
        "structure changed:\n{source}\n=>\n{formatted}"
    );
    formatted
}

#[test]
fn canonical_spacing_and_annotations() {
    assert_eq!(
        verify("x=1+2\nfn double(n:Int):Int\n    n*2\n"),
        "x = 1 + 2\n\nfn double(n: Int): Int\n    n * 2\n"
    );
}

#[test]
fn formatter_repairs_noncanonical_indentation() {
    assert_eq!(
        verse_fmt::format("if true\n\tpass\n", FileId(0)).unwrap(),
        "if true\n    pass\n"
    );
    assert_eq!(
        verse_fmt::format("if true\n   pass\n", FileId(0)).unwrap(),
        "if true\n    pass\n"
    );
}

#[test]
fn significant_parentheses_and_comments_survive() {
    verify("# comment\na=(try lookup()).email\nb=(user.email).lower()\nc=x otherwise (1+2)\n");
}

#[test]
fn repeated_unary_minus_has_stable_spacing() {
    assert_eq!(verify("i----------impo"), "i - ---------impo\n");
}

#[test]
fn import_sorting_and_check_only_equivalence() {
    let source = "import z\nimport a\nx=1\n";
    let formatted = verse_fmt::format(source, FileId(0)).unwrap();
    assert_eq!(formatted, "import a\nimport z\nx = 1\n");
    assert_eq!(verse_fmt::format(&formatted, FileId(0)).unwrap(), formatted);
}

#[test]
fn long_chains_wrap_one_call_per_line() {
    let source = "result = users.filter(user -> user.active).map(user -> user.display_name).sort_by(.name).prefix(100).collect()\n";
    let formatted = verify(source);
    assert!(formatted.contains("\n    .filter("));
    assert!(formatted.contains("\n    .map("));
}

#[test]
fn wrapping_a_chain_also_formats_its_enclosing_list() {
    verify(
        "print(try response.body.a_very_long_method_name_that_makes_the_chain_long_enough_to_wrap_in_the_formatter())\n",
    );
    verify(
        "values = [(((((none otherwise (value)) otherwise (value)) otherwise (value)) + value)).first().name, value]\n",
    );
}

#[test]
fn multiline_imports_sort_as_whole_units() {
    let source = "import z\nimport (\n    a,\n    b,\n) from m\n";
    let formatted = verse_fmt::format(source, FileId(0)).unwrap();
    assert_eq!(formatted, "import (\n    a,\n    b,\n) from m\nimport z\n");
    assert_eq!(verse_fmt::format(&formatted, FileId(0)).unwrap(), formatted);
}

#[test]
fn multiline_lists_have_trailing_commas() {
    assert_eq!(
        verify("x = [\n    1,\n    2\n]\n"),
        "x = [\n    1,\n    2,\n]\n"
    );
}

#[test]
fn crlf_lists_format_in_one_pass_and_bare_cr_is_rejected() {
    assert_eq!(verify("f(\r\n    value)\r\n"), "f(\n    value,)\n");
    assert!(verse_fmt::format("\r\r2(\ry)", FileId(0)).is_err());
}

#[test]
fn trailing_commas_precede_inline_comments() {
    assert_eq!(
        verify("x = [\n    1 # keep this comment\n]\n"),
        "x = [\n    1,  # keep this comment\n]\n"
    );
}

#[test]
fn explicit_lambda_close_is_redundant() {
    let source = "result = xs.map(x ->\n    x + 1\n)\nprint(result)\n";
    let output = verify(source);
    assert!(!output.lines().any(|l| l.trim() == ")"));
}

#[test]
fn literals_preserve_their_contents() {
    verify("fn letter()\n    r\"\"\"\n    Dear reader,\n\n    Hello.\n    \"\"\"\n");
    verify("x = \"{email otherwise \"none\"}\"\ny = b\"a\\t\"\n");
}

#[test]
fn declaration_documentation_stays_attached() {
    assert_eq!(
        verify("## Document the function.\nfn f()\n    pass\n"),
        "## Document the function.\nfn f()\n    pass\n"
    );
    assert_eq!(
        verify("## Module documentation.\n\n## Function documentation.\nfn f()\n    pass\n"),
        "## Module documentation.\n\n## Function documentation.\nfn f()\n    pass\n"
    );
    assert_eq!(
        verify("## Module documentation.\n\n"),
        "## Module documentation.\n\n"
    );
}

#[test]
fn specification_examples_are_idempotent_and_preserve_structure() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let output = Command::new("python3")
        .arg(root.join("tools/extract-examples"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let examples: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for example in examples.as_array().unwrap() {
        if example["kind"] != "parse" {
            continue;
        }
        let source = example["source"].as_str().unwrap();
        let formatted = verse_fmt::format(source, FileId(0))
            .unwrap_or_else(|d| panic!("{}: {d:#?}", example["id"]));
        assert_eq!(
            verse_fmt::format(&formatted, FileId(0))
                .unwrap_or_else(|d| panic!("{} formatted:\n{formatted}\n{d:#?}", example["id"])),
            formatted,
            "{} not idempotent",
            example["id"]
        );
        // Import order is canonicalized, so compare each non-import item.
        let structures = |text: &str| {
            parse(text, FileId(0))
                .syntax()
                .children()
                .filter(|n| n.kind() != verse_syntax::Kind::Import)
                .map(|n| fingerprint(&n))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            structures(source),
            structures(&formatted),
            "{} changed structure:\n{formatted}",
            example["id"]
        );
    }
}

proptest::proptest! {
    #[test]
    fn generated_arithmetic_is_idempotent(a in -10000i32..10000, b in -10000i32..10000, c in -10000i32..10000, padding in 0usize..8) {
        let space = " ".repeat(padding);
        verify(&format!("answer{space}={space}({a}{space}+{space}{b}){space}*{space}{c}\n"));
    }

    #[test]
    fn generated_grammar_preserves_structure(expression in expression_strategy()) {
        verify(&format!("answer = {expression}\n"));
    }
}

fn expression_strategy() -> impl proptest::strategy::Strategy<Value = String> {
    use proptest::prelude::*;
    let leaf = prop_oneof![
        (-1000i32..1000).prop_map(|n| n.to_string()),
        Just("name".to_owned()),
        Just("\"text\"".to_owned()),
        Just("none".to_owned())
    ];
    leaf.prop_recursive(5, 64, 5, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("({a} + {b})")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("[{a}, {b}]")),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| format!("{{{a}: {b}}}")),
            inner.clone().prop_map(|a| format!("f({a})")),
            inner.clone().prop_map(|a| format!("({a}).first().name")),
            inner.clone().prop_map(|a| format!("(x -> {a})")),
            inner.prop_map(|a| format!("(if flag then {a} else none)")),
        ]
    })
}
