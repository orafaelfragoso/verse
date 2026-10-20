#![no_main]
use libfuzzer_sys::fuzz_target;
use verse_syntax::{FileId, parse};

fn expression(bytes: &mut std::slice::Iter<'_, u8>, depth: usize) -> String {
    let byte = bytes.next().copied().unwrap_or(0);
    if depth == 0 || bytes.len() == 0 {
        return ["value", "1", "\"text\"", "none"][byte as usize % 4].into();
    }
    match byte % 8 {
        0 => format!(
            "({} + {})",
            expression(bytes, depth - 1),
            expression(bytes, depth - 1)
        ),
        1 => format!(
            "[{}, {}]",
            expression(bytes, depth - 1),
            expression(bytes, depth - 1)
        ),
        2 => format!(
            "{{{}: {}}}",
            expression(bytes, depth - 1),
            expression(bytes, depth - 1)
        ),
        3 => format!("f({})", expression(bytes, depth - 1)),
        4 => format!("({}).first().name", expression(bytes, depth - 1)),
        5 => format!("(x -> {})", expression(bytes, depth - 1)),
        6 => format!("(if flag then {} else none)", expression(bytes, depth - 1)),
        _ => format!(
            "({} otherwise ({}))",
            expression(bytes, depth - 1),
            expression(bytes, depth - 1)
        ),
    }
}

fn structure(source: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let parsed = parse(source, FileId(0));
    let root = parsed.syntax();
    let mut items = Vec::new();
    let mut imports = Vec::new();
    for child in root.children() {
        if child.kind() == verse_syntax::Kind::Import {
            imports.push(verse_syntax::tree::fingerprint(&child));
        } else {
            items.push(verse_syntax::tree::fingerprint(&child));
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

fuzz_target!(|data: &[u8]| {
    let expression = expression(&mut data.iter(), 6);
    let variant = data.first().copied().unwrap_or(0) % 3;
    let source = match variant {
        0 => format!(
            "## Generated syntax.\n\nfn generated(value)\n    result = {expression}\n    if flag\n        print(result)\n    result\n"
        ),
        1 => format!(
            "import z\nimport a\n\ntext =\n    \"\"\"\n    import zebra\n    import alpaca\n    \"\"\"\n\nfn generated(value)\n    result = {expression}\n    result\n"
        ),
        _ => format!(
            "fn generated(values)\n    result = values.map(value ->\n        value + {expression}\n    result\n"
        ),
    };
    let parsed = parse(&source, FileId(0));
    assert!(parsed.is_valid(), "{source}\n{:?}", parsed.diagnostics);
    assert_eq!(parsed.syntax().text().to_string(), source);
    let formatted = verse_fmt::format(&source, FileId(0))
        .unwrap_or_else(|diagnostics| panic!("formatter failed for {source:?}: {diagnostics:?}"));
    assert_eq!(verse_fmt::format(&formatted, FileId(0)).unwrap(), formatted);
    assert_eq!(structure(&source), structure(&formatted));
});
