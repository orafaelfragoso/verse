use std::{fs, path::PathBuf};
use verse_syntax::{FileId, lex, parse};

#[test]
fn token_and_cst_snapshots() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/syntax");
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "vs") {
            continue;
        }
        let source = fs::read_to_string(&path).unwrap();
        let tokens = lex(&source, FileId(0))
            .tokens
            .iter()
            .map(|t| {
                format!(
                    "{:?} {}..{} {:?}\n",
                    t.kind,
                    t.span.start,
                    t.span.end,
                    t.text(&source)
                )
            })
            .collect::<String>();
        let cst = format!("{:#?}\n", parse(&source, FileId(0)).syntax());
        assert_eq!(
            tokens,
            fs::read_to_string(path.with_extension("tokens")).unwrap(),
            "{}",
            path.display()
        );
        assert_eq!(
            cst,
            fs::read_to_string(path.with_extension("cst")).unwrap(),
            "{}",
            path.display()
        );
    }
}
