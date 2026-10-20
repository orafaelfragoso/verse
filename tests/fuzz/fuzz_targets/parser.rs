#![no_main]
use libfuzzer_sys::fuzz_target;
use verse_syntax::{FileId, lexer::decode, parse};

fuzz_target!(|data: &[u8]| {
    let source = decode(data, FileId(0))
        .map(str::to_owned)
        .unwrap_or_else(|_| String::from_utf8_lossy(data).into_owned());
    let parsed = parse(&source, FileId(0));
    assert_eq!(parsed.syntax().text().to_string(), source);
});
