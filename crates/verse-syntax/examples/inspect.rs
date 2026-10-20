//! Development inspection, not a syntax-only implementation of `verse check`.
use std::{env, fs};
use verse_syntax::{FileId, SourceFile, parse};
fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.first().is_some_and(|a| a == "--corpus") {
        let examples: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&args[1]).unwrap()).unwrap();
        let mut failed = 0;
        for example in examples.as_array().unwrap() {
            if example["kind"] == "fragment" {
                continue;
            }
            let parsed = parse(example["source"].as_str().unwrap(), FileId(0));
            if !parsed.is_valid() {
                failed += 1;
                println!(
                    "{} {}",
                    example["id"],
                    parsed
                        .diagnostics
                        .iter()
                        .map(|d| d.code.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                );
                let source = SourceFile::new(
                    FileId(0),
                    example["path"].as_str().unwrap(),
                    example["source"].as_str().unwrap(),
                );
                for diagnostic in parsed.diagnostics {
                    print!("{}", diagnostic.render(&source));
                }
            }
        }
        println!("{failed} examples with diagnostics");
    } else {
        let source = fs::read_to_string(&args[0]).unwrap();
        let parsed = parse(&source, FileId(0));
        println!("{:#?}", parsed.syntax());
        let source = SourceFile::new(FileId(0), &args[0], source);
        for d in parsed.diagnostics {
            print!("{}", d.render(&source));
        }
    }
}
