#![no_main]
use libfuzzer_sys::fuzz_target;
use verse_syntax::{FileId, Kind, lex};
fuzz_target!(|source: &str| {
    let lexed = lex(source, FileId(0));
    assert_eq!(
        lexed
            .tokens
            .iter()
            .map(|t| t.text(source))
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
