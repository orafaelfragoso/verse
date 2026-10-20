use std::{path::PathBuf, process::Command};
use verse_syntax::{FileId, Kind, SourceFile, ast::Module, lex, parse};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn named_rule_fixtures() {
    for entry in std::fs::read_dir(root().join("tests/conformance")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "vs") {
            valid(&std::fs::read_to_string(&path).unwrap());
        }
    }
}

#[test]
fn specification_examples() {
    let output = Command::new("python3")
        .arg(root().join("tools/extract-examples"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let examples: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut failures = Vec::new();
    let mut count = 0;
    for example in examples.as_array().unwrap() {
        if example["kind"] == "fragment" {
            continue;
        }
        count += 1;
        let source = example["source"].as_str().unwrap();
        let parsed = parse(source, FileId(0));
        assert_eq!(
            parsed.syntax().text().to_string(),
            source,
            "{} lost source text",
            example["id"]
        );
        let mut codes: Vec<_> = parsed.diagnostics.iter().map(|d| d.code.as_str()).collect();
        codes.sort();
        codes.dedup();
        let mut expected: Vec<_> = example["codes"].as_array().map_or_else(Vec::new, |a| {
            a.iter().map(|v| v.as_str().unwrap()).collect()
        });
        expected.sort();
        if codes != expected {
            let file = SourceFile::new(FileId(0), example["id"].as_str().unwrap(), source);
            failures.push(format!(
                "{} expected {expected:?}, got {codes:?}\n{}",
                example["id"],
                parsed
                    .diagnostics
                    .iter()
                    .map(|d| d.render(&file))
                    .collect::<String>()
            ));
        }
    }
    assert!(count > 100);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn valid(source: &str) -> verse_syntax::Parse {
    let p = parse(source, FileId(0));
    assert!(p.is_valid(), "{source}\n{:#?}", p.diagnostics);
    assert_eq!(p.syntax().text().to_string(), source);
    p
}

#[test]
fn s3_1_crlf_and_unicode_positions() {
    valid("fn café(数量)\r\n    数量\r\n");
    let file = SourceFile::new(FileId(3), "unicode.vs", "é\r\n数量\n");
    assert_eq!(file.position(2), (1, 2));
    assert_eq!(file.position(4), (2, 1));
    assert_eq!(file.position(7), (2, 2));
    assert_eq!(file.line(1), "é");
}

#[test]
fn s3_5_unicode_name_categories() {
    let tokens = lex("Éclair ǅelta 数量 _hidden", FileId(0));
    let kinds: Vec<_> = tokens
        .tokens
        .iter()
        .filter(|t| matches!(t.kind, Kind::Ident | Kind::TypeIdent))
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        [Kind::TypeIdent, Kind::TypeIdent, Kind::Ident, Kind::Ident]
    );
}

#[test]
fn s3_7_tuple_positions_and_numeric_methods() {
    let l = lex(
        "t.0.1\n10.seconds()\n1e-9\n-9223372036854775808\n",
        FileId(0),
    );
    assert!(l.diagnostics.is_empty(), "{:?}", l.diagnostics);
    let numbers: Vec<_> = l
        .tokens
        .iter()
        .filter(|t| matches!(t.kind, Kind::Int | Kind::Float))
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        numbers,
        [Kind::Int, Kind::Int, Kind::Int, Kind::Float, Kind::Int]
    );
}

#[test]
fn s3_8_precedence() {
    let p = valid("x = -2 ** 3 ** 2\ny = try a.b(c)\nz = not a < b and c\n");
    let text = format!("{:#?}", p.syntax());
    assert!(text.contains("Unary"));
    assert!(text.contains("Binary"));
    let unary = p
        .syntax()
        .descendants()
        .find(|n| n.kind() == Kind::Unary)
        .unwrap();
    assert_eq!(unary.children().next().unwrap().kind(), Kind::Binary);
    let power = unary.children().next().unwrap();
    assert_eq!(power.children().last().unwrap().kind(), Kind::Binary);
}

#[test]
fn s5_4_block_lambdas_close_on_dedent() {
    valid(
        "labels = orders.map(order ->\n    status = if order.paid then \"paid\" else \"due\"\n    status\nprint(labels)\n",
    );
    valid("labels = orders.map(order ->\n    order.name\n)\nprint(labels)\n");
    valid("outer = xs.map(x ->\n    inner = ys.map(y ->\n        x + y\n    inner\nprint(outer)\n");
    valid("outer = xs.map(x ->\n    ys.map(y ->\n        x + y\nprint(outer)\n");
    valid("outer = xs.map(x ->\n    ys.map(y ->\n        x + y\n");
    valid("outer = xs.map(\n    x ->\n        x + 1\n)\n");
}

#[test]
fn s3_7_interpolation_colon_and_nested_text() {
    valid("a = \"{r\"{not_interpolated}\"}\"\n");
    valid(
        "a = \"{f(x: 1)} { {\"a\": 1}[\"a\"] } {price:.2}\"\nb = \"{email otherwise \"none\"}\"\n",
    );
    let p = parse("a = \"{1 + }\"\n", FileId(2));
    assert!(!p.is_valid());
    assert_eq!(p.diagnostics[0].primary.span.file_id, FileId(2));
}

#[test]
fn interpolation_expressions_reach_the_arena_with_absolute_spans() {
    let source = "text = \"{first + 1} {second.name}\"\n";
    let parsed = valid(source);
    assert_eq!(parsed.interpolations.len(), 2);
    let module = Module::lower_parse(&parsed, FileId(4));
    let literal = module
        .nodes
        .iter()
        .find(|n| n.kind == Kind::Literal && n.span.start < 10 && n.span.end > 20)
        .unwrap();
    assert_eq!(literal.children.len(), 2);
    for &id in &module.edges[literal.children.clone()] {
        let expression = module.node(id);
        assert_eq!(expression.span.file_id, FileId(4));
        assert!(
            expression.span.start > literal.span.start && expression.span.end < literal.span.end
        );
    }
    assert!(
        module
            .tokens
            .iter()
            .any(|token| module.symbols.resolve(token.symbol) == "first")
    );
}

#[test]
fn recovery_preserves_following_declaration_and_source() {
    let text = "x = )\nfn good()\n    pass\n";
    let p = parse(text, FileId(0));
    assert!(!p.is_valid());
    assert_eq!(p.syntax().text().to_string(), text);
    assert!(p.syntax().descendants().any(|n| n.kind() == Kind::Error));
    assert!(p.syntax().descendants().any(|n| n.kind() == Kind::Function));
}

#[test]
fn arena_ids_and_interning_are_stable_for_the_same_module() {
    let p = valid("x = 1\ny = x\n");
    let a = Module::lower(&p.syntax(), FileId(7));
    let b = Module::lower(&p.syntax(), FileId(7));
    assert_eq!(a.root, b.root);
    assert_eq!(a.nodes.len(), b.nodes.len());
    assert_eq!(a.edges, b.edges);
    assert_eq!(a.tokens, b.tokens);
    assert!(a.tokens.iter().all(|token| !token.kind.is_trivia()
        && !token.kind.is_layout()
        && token.kind != Kind::DocComment));
    for (a, b) in a.nodes.iter().zip(&b.nodes) {
        assert_eq!(a.span, b.span);
        assert_eq!(a.children, b.children);
        assert_eq!(a.tokens, b.tokens);
    }
    let mut interner = verse_syntax::ast::Interner::default();
    let x = interner.intern("x");
    assert_eq!(x, interner.intern("x"));
    assert_eq!(interner.resolve(x), "x");
}

#[test]
fn deeply_nested_valid_syntax_is_not_rejected_by_an_arbitrary_limit() {
    let depth = 20_000;
    let nested = format!("value = {}1{}\n", "(".repeat(depth), ")".repeat(depth));
    let parsed = valid(&nested);
    let arena = Module::lower(&parsed.syntax(), FileId(0));
    assert!(arena.nodes.len() > depth);
    assert!(
        !verse_syntax::tree::fingerprint(&parsed.syntax()).is_empty(),
        "iterative fingerprint must handle the same depth"
    );
    drop(arena);
    drop(parsed);
    let unary = format!("value = {}1\n", "-".repeat(depth));
    let parsed = valid(&unary);
    let arena = Module::lower(&parsed.syntax(), FileId(0));
    assert!(arena.nodes.len() > depth);
    drop(arena);
    drop(parsed);
    let ty = format!(
        "alias Nested = {}Int{}\n",
        "List<".repeat(1_000),
        ">".repeat(1_000)
    );
    valid(&ty);
}

#[test]
fn unclosed_brackets_recover_at_the_next_declaration() {
    let text = "x = (3,\nfn good()\n    pass\n";
    let parsed = parse(text, FileId(0));
    assert!(!parsed.is_valid());
    assert_eq!(parsed.syntax().text().to_string(), text);
    assert!(
        parsed
            .syntax()
            .children()
            .any(|node| node.kind() == Kind::Function)
    );
}

#[test]
fn ast_tokens_keep_absolute_spans() {
    let parsed = valid("text = \"{first + 1}\"\n");
    let module = Module::lower_parse(&parsed, FileId(3));
    let token = module
        .tokens
        .iter()
        .find(|token| module.symbols.resolve(token.symbol) == "first")
        .unwrap();
    assert_eq!(token.span.file_id, FileId(3));
    assert!(token.span.start >= 9 && token.span.end <= 20);
}

proptest::proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(256))]
    #[test]
    fn arbitrary_utf8_never_panics_and_round_trips(source in "(?s).{0,512}") {
        let p = parse(&source, FileId(0));
        proptest::prop_assert_eq!(p.syntax().text().to_string(), source.clone());
        let l = lex(&source, FileId(0));
        proptest::prop_assert_eq!(l.tokens.iter().map(|t| t.text(&source)).collect::<String>(), source);
        let mut balance = 0isize;
        for token in l.tokens {
            if token.kind == Kind::Indent { balance += 1; }
            else if token.kind == Kind::Dedent { balance -= 1; }
            proptest::prop_assert!(balance >= 0);
        }
        proptest::prop_assert_eq!(balance, 0);
    }
}

#[test]
fn numeric_separators_follow_the_base() {
    valid("value = 0xDEAD_BEEF\n");
    valid("value = 1_000_000\n");
    assert!(!parse("value = 0x_DEAD\n", FileId(0)).is_valid());
    assert!(!parse("value = 1e_9\n", FileId(0)).is_valid());
}

#[test]
fn character_escapes_and_width_are_checked() {
    valid("letter = '\\n'\nzero = '\\0'\nsmile = '\\u{1F600}'\n");
    let parsed = parse("letter = 'ab'\n", FileId(0));
    assert!(parsed.diagnostics.iter().any(|d| d.code == "V0114"));
    assert!(
        !parse("letter = '\\q'\n", FileId(0))
            .diagnostics
            .iter()
            .all(|d| d.code != "V0104")
    );
}

#[test]
fn one_unterminated_interpolation_does_not_duplicate_the_same_error() {
    let parsed = parse("text = \"{value\n}\"\n", FileId(0));
    let at_close = parsed
        .diagnostics
        .iter()
        .filter(|d| d.code == "V0201" && d.primary.span.start == 15)
        .count();
    assert_eq!(at_close, 1);
}

#[test]
fn only_a_leading_bom_is_reported_as_a_byte_order_mark() {
    let leading = parse("\u{feff}x = 1\n", FileId(0));
    assert!(leading.diagnostics.iter().any(|d| d.code == "V0113"));
    let middle = parse("x = 1\n\u{feff}\n", FileId(0));
    assert!(middle.diagnostics.iter().all(|d| d.code != "V0113"));
}
