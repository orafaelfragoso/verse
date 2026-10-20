#![no_main]
use libfuzzer_sys::fuzz_target;
use verse_syntax::{FileId, Kind, lex, lexer::decode};

fuzz_target!(|data: &[u8]| {
    let source = decode(data, FileId(0))
        .map(str::to_owned)
        .unwrap_or_else(|_| String::from_utf8_lossy(data).into_owned());
    let lexed = lex(&source, FileId(0));
    assert_eq!(
        lexed
            .tokens
            .iter()
            .map(|t| t.text(&source))
            .collect::<String>(),
        source
    );
    let mut balance = 0;
    for token in lexed.tokens {
        match token.kind {
            Kind::Indent => balance += 1,
            Kind::Dedent => balance -= 1,
            _ => {}
        }
        assert!(balance >= 0);
    }
    assert_eq!(balance, 0);
});
