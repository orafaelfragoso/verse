use std::{env, fs, hint::black_box, time::Instant};
use verse_syntax::FileId;

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let source = fs::read_to_string(&args[1]).unwrap();
    let iterations: usize = args
        .get(2)
        .and_then(|value| value.parse().ok())
        .unwrap_or(1)
        .max(1);
    let parsed = if args[0] == "ast" {
        let parsed = verse_syntax::parse(black_box(&source), FileId(0));
        assert!(parsed.is_valid(), "{:?}", parsed.diagnostics);
        Some(parsed)
    } else {
        None
    };
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
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
                black_box(verse_syntax::ast::Module::lower_parse(
                    parsed.as_ref().unwrap(),
                    FileId(0),
                ));
            }
            "format" => {
                black_box(verse_fmt::format(black_box(&source), FileId(0)).unwrap());
            }
            _ => panic!("unknown stage"),
        }
        samples.push(start.elapsed().as_nanos());
    }
    println!(
        "{}",
        serde_json::json!({
            "stage": args[0],
            "bytes": source.len(),
            "lines": source.lines().count(),
            "iterations": iterations,
            "samples_ns": samples,
        })
    );
}
