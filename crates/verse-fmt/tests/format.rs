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

#[test]
fn formatter_formats_every_conformance_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/conformance");
    let mut count = 0;
    for entry in std::fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "vs") {
            continue;
        }
        count += 1;
        let source = std::fs::read_to_string(&path).unwrap();
        let formatted = verse_fmt::format(&source, FileId(0))
            .unwrap_or_else(|d| panic!("{}: {d:?}", path.display()));
        assert_eq!(
            verse_fmt::format(&formatted, FileId(0)).unwrap(),
            formatted,
            "{} not idempotent",
            path.display()
        );
        assert_eq!(
            canonical_structure(&source),
            canonical_structure(&formatted),
            "{} changed structure",
            path.display()
        );
    }
    assert!(count > 0, "no conformance fixtures found");
}

#[test]
fn block_lambda_continuation_headers_indent_from_their_column() {
    assert_eq!(
        verify("r = items\n    .map(x ->\n        x + 1\nprint(r)\n"),
        "r = items\n    .map(x ->\n        x + 1\nprint(r)\n"
    );
    assert_eq!(
        verify("outer = xs.map(\n    x ->\n        x + 1\n)\n"),
        "outer = xs.map(\n    x ->\n        x + 1\n)\n"
    );
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
        canonical_structure(source),
        canonical_structure(&formatted),
        "structure changed:\n{source}\n=>\n{formatted}"
    );
    formatted
}

fn canonical_structure(source: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let parsed = parse(source, FileId(0));
    let root = parsed.syntax();
    let mut items = Vec::new();
    let mut imports = Vec::new();
    for child in root.children() {
        if child.kind() == verse_syntax::Kind::Import {
            imports.push(fingerprint(&child));
        } else {
            items.push(fingerprint(&child));
        }
    }
    imports.sort();
    let mut comments: Vec<_> = root
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| {
            matches!(
                token.kind(),
                verse_syntax::Kind::Comment | verse_syntax::Kind::DocComment
            )
        })
        .map(|token| token.text().trim_end().to_owned())
        .collect();
    comments.sort();
    (items, imports, comments)
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
    assert_eq!(
        verse_fmt::format("if true\n   a = 1\n   b = 2\n", FileId(0)).unwrap(),
        "if true\n    a = 1\n    b = 2\n"
    );
    assert_eq!(
        verse_fmt::format("if true\n    if false\n         pass\n", FileId(0)).unwrap(),
        "if true\n    if false\n        pass\n"
    );
    assert_eq!(
        verse_fmt::format(
            "if true\n    if false\n        a = 1\n     b = 2\n",
            FileId(0)
        )
        .unwrap(),
        "if true\n    if false\n        a = 1\n    b = 2\n"
    );
    // Two spaces could mean the block at four or a dedent to zero.
    assert!(verse_fmt::format("if true\n    pass\n  other = 1\n", FileId(0)).is_err());
    // Three spaces from an eight-space block is closer to four, but the lexer
    // would pop both levels; that is not an unambiguous repair.
    assert!(
        verse_fmt::format(
            "if true\n    if false\n        a = 1\n   b = 2\n",
            FileId(0)
        )
        .is_err()
    );
}

#[test]
fn canonical_spacing_is_structural() {
    let cases = [
        ("a=xs [0]\n", "a = xs[0]\n"),
        ("a=settings [\"timeout\"]\n", "a = settings[\"timeout\"]\n"),
        ("a=1 ..= 10\n", "a = 1..=10\n"),
        ("a=[0, .. rest]\n", "a = [0, ..rest]\n"),
        ("a=xs [2 ..]\n", "a = xs[2..]\n"),
        ("a=foo(1,(2, 3))\n", "a = foo(1, (2, 3))\n"),
        ("a=if c then(1) else(2)\n", "a = if c then (1) else (2)\n"),
        ("a=p is(1, 2)\n", "a = p is (1, 2)\n"),
        ("a=b **(c)\n", "a = b ** (c)\n"),
        (
            "a=match x\n    (1, 2) -> 3\n",
            "a = match x\n    (1, 2) -> 3\n",
        ),
        (
            "fn f(x:(Int, Int))\n    pass\n",
            "fn f(x: (Int, Int))\n    pass\n",
        ),
        ("Circle(r) = shape\n", "Circle(r) = shape\n"),
    ];
    for (source, expected) in cases {
        assert_eq!(verify(source), expected, "source: {source}");
    }
}

#[test]
fn multiline_text_is_never_sorted_like_imports() {
    assert_eq!(
        verify("import z\nimport a\ntext = \"\"\"\nimport zebra\nimport alpaca\n\"\"\"\n"),
        "import a\nimport z\ntext = \"\"\"\nimport zebra\nimport alpaca\n\"\"\"\n"
    );
}

#[test]
fn single_line_lists_do_not_gain_trailing_commas() {
    assert_eq!(verify("x = foo(1, 2) # keep\n"), "x = foo(1, 2)  # keep\n");
    // A text token's own newline does not make its list multiline.
    assert_eq!(verify("x = [\"a\\nb\"]\n"), "x = [\"a\\nb\"]\n");
}

#[test]
fn a_block_lambda_argument_does_not_gain_a_trailing_comma() {
    let source =
        "fn generated(values)\n    result = values.map(value ->\n        value + 1\n    result\n";
    assert_eq!(verify(source), source);
}

#[test]
fn a_chain_cannot_follow_a_dedent_closed_lambda() {
    assert!(
        verse_fmt::format("r = xs.map(x ->\n    x + 1\n).filter(y -> y)\n", FileId(0)).is_err()
    );
}

#[test]
fn comments_before_a_declaration_stay_with_it() {
    assert_eq!(
        verify("fn a()\n    pass\n# Describes b.\nfn b()\n    pass\n"),
        "fn a()\n    pass\n\n# Describes b.\nfn b()\n    pass\n"
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
fn import_groups_stay_separated_and_sort_within_each_group() {
    let source = "import z\nimport a\n\nimport y\nimport b\nx = 1\n";
    assert_eq!(
        verify(source),
        "import a\nimport z\n\nimport b\nimport y\nx = 1\n"
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
        assert_eq!(
            canonical_structure(source),
            canonical_structure(&formatted),
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

    #[test]
    fn generated_programs_with_imports_comments_and_text_preserve_structure(
        comment in "[a-z ]{0,16}",
        value in 0u16..1000
    ) {
        let source = format!(
            "# {comment}\nimport z\nimport a\n\nfn f(name: Text): Text\n    # inner {value}\n    value =\n        \"\"\"\n        import zebra\n        import alpaca\n        \"\"\"\n    value\n"
        );
        verify(&source);
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
