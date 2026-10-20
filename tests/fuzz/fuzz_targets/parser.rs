#![no_main]
use libfuzzer_sys::fuzz_target;
use verse_syntax::{FileId, parse};
fuzz_target!(|source: &str| {
    let parsed = parse(source, FileId(0));
    assert_eq!(parsed.syntax().text().to_string(), source);
});
