use std::{env, fs, hint::black_box, time::Instant};
use verse_syntax::FileId;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let source = fs::read_to_string(&args[1]).unwrap();
    let start = Instant::now();
    match args[0].as_str() {
        "lex" => {
            let result = verse_syntax::lex(black_box(&source), FileId(0));
            assert!(result.diagnostics.is_empty());
            black_box(result);
        }
        "parse" => {
            let result = verse_syntax::parse(black_box(&source), FileId(0));
            assert!(result.is_valid(), "{:?}", result.diagnostics);
            black_box(result);
        }
        "ast" => {
            let result = verse_syntax::parse(black_box(&source), FileId(0));
            assert!(result.is_valid());
            black_box(verse_syntax::ast::Module::lower_parse(&result, FileId(0)));
        }
        "format" => {
            black_box(verse_fmt::format(black_box(&source), FileId(0)).unwrap());
        }
        _ => panic!("unknown stage"),
    }
    println!(
        "{}",
        serde_json::json!({"stage": args[0], "bytes": source.len(), "lines": source.lines().count(), "elapsed_ns": start.elapsed().as_nanos()})
    );
}
