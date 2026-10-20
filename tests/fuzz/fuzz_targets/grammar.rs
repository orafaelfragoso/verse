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

fuzz_target!(|data: &[u8]| {
    let expression = expression(&mut data.iter(), 6);
    let source = format!(
        "## Generated syntax.\n\nfn generated(value)\n    result = {expression}\n    if flag\n        print(result)\n    result\n"
    );
    let parsed = parse(&source, FileId(0));
    assert!(parsed.is_valid(), "{source}\n{:?}", parsed.diagnostics);
    assert_eq!(parsed.syntax().text().to_string(), source);
    let formatted = verse_fmt::format(&source, FileId(0)).unwrap();
    assert_eq!(verse_fmt::format(&formatted, FileId(0)).unwrap(), formatted);
    assert_eq!(
        verse_syntax::tree::fingerprint(&parsed.syntax()),
        verse_syntax::tree::fingerprint(&parse(&formatted, FileId(0)).syntax())
    );
});
