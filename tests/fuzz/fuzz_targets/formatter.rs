#![no_main]
use libfuzzer_sys::fuzz_target;
use verse_syntax::{FileId, Kind, lexer::decode, parse, tree::fingerprint};

fn structure(source: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let parsed = parse(source, FileId(0));
    let root = parsed.syntax();
    let mut items = Vec::new();
    let mut imports = Vec::new();
    for child in root.children() {
        if child.kind() == Kind::Import {
            imports.push(fingerprint(&child));
        } else {
            items.push(fingerprint(&child));
        }
    }
    imports.sort();
    let mut comments: Vec<_> = root
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| matches!(token.kind(), Kind::Comment | Kind::DocComment))
        .map(|token| token.text().trim_end().to_owned())
        .collect();
    comments.sort();
    (items, imports, comments)
}

fuzz_target!(|data: &[u8]| {
    let source = match decode(data, FileId(0)) {
        Ok(source) => source.to_owned(),
        Err(_) => String::from_utf8_lossy(data).into_owned(),
    };
    let original = parse(&source, FileId(0));
    if let Ok(formatted) = verse_fmt::format(&source, FileId(0)) {
        assert_eq!(verse_fmt::format(&formatted, FileId(0)).unwrap(), formatted);
        assert_eq!(
            structure(&source),
            structure(&formatted),
            "structure changed for {source:?}"
        );
    } else {
        assert!(
            !original.is_valid(),
            "formatter rejected valid source: {source:?}"
        );
    }
});
