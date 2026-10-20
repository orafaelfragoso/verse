#![no_main]
use libfuzzer_sys::fuzz_target;
use verse_syntax::{FileId, Kind, parse, tree::fingerprint};
fuzz_target!(|source: &str| {
    let original = parse(source, FileId(0));
    if let Ok(formatted) = verse_fmt::format(source, FileId(0)) {
        assert_eq!(verse_fmt::format(&formatted, FileId(0)).unwrap(), formatted);
        let structure = |s: &str| {
            parse(s, FileId(0))
                .syntax()
                .children()
                .filter(|n| n.kind() != Kind::Import)
                .map(|n| fingerprint(&n))
                .collect::<Vec<_>>()
        };
        assert_eq!(structure(source), structure(&formatted));
    } else {
        assert!(
            !original.is_valid(),
            "formatter rejected valid source: {source}"
        );
    }
});
