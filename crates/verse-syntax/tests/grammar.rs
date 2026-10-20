use verse_syntax::{FileId, Kind, parse};

#[test]
fn s17_grammar_production_fixtures() {
    let cases = [
        "import a as b\nimport a as b from m\nimport (a, B,) from m\n",
        "fn f<T: Show + Copy, U>(mut x: T, take y: U, flag = true): T? raises Error\n    x\n",
        "type Box<T>\n    ## Field doc\n    export value: T\n    count = 0\n",
        "type Shape = Circle(radius: Float)\n    | Square(side: Float)\n",
        "alias Handler = fn(mut x: Int, take Text): Int?? raises Error\n",
        "alias Pair = (first: Text, second: List<Int>)\n",
        "trait Read<T>: Show + Copy\n    fn get(self): T\n    fn label(self): Text\n        \"reader\"\n",
        "impl<T: Show> Show for Box<T>\n    export fn show(self): Text\n        \"box\"\n",
        "impl Unique for File\n",
        "extern c\n    ## Function doc\n    export fn Open as open(path: Bytes): Int\n",
        "outer: for (a, b) in pairs\n    while true\n        break outer if a == b\n        continue if false\n",
        "fn example(x)\n    return (if x then 1 else 2)\n",
        "fn smaller(a, b)\n    if a < b then a else b\n",
        "match x\n    Some(name:) if name == \"x\" -> pass\n    Record(body: b: Text) -> print(b)\n    [a, ..rest] -> a\n    (a, b) -> a + b\n    1..=3 -> 1\n    none? -> 0\n    x: Int | Float -> x\n    _ -> return 0\n",
        "match\n    x > 0 -> 1\n    x < 0 -> -1\n    _ -> 0\n",
        "x = Channel.new<Int>(capacity: 10)\ny = a.b < c > (d)\n",
        "x = [0, ..rest]\ny = {..defaults, \"key\": 1}\nz = set{..xs, 1}\n",
        "x = () -> 1\ny = (a: Int, b: Int) -> a + b\nz = xs.map(.name.upper())\n",
        "x = match value\n    A -> (n -> n * 2)\n    _ -> (n -> -n)\n",
        "x = if flag\n    1\nelse\n    2\n",
        "a = nums[1..]\nb = nums[..3]\nc = nums[..=3]\nd = nums[1..=3]\n",
        "a = nums[if flag then 1 else 2]\nb = lookup[a < b]\n",
        "Circle(r) = shape otherwise return 0.0\nmut (a, b) = pair\n",
    ];
    for source in cases {
        let parsed = parse(source, FileId(0));
        assert!(parsed.is_valid(), "{source}\n{:#?}", parsed.diagnostics);
        assert_eq!(parsed.syntax().text().to_string(), source);
    }
}

#[test]
fn s17_reject_invalid_grammar_not_just_unbalanced_delimiters() {
    let cases = [
        ("if x = 1\n    pass\n", "V0201"),
        ("fn f()\n", "V0202"),
        ("if true\n    # only a comment\n", "V0202"),
        ("x = {}\n", "V0210"),
        ("x = 10.\n", "V0107"),
        ("fn if()\n    pass\n", "V0203"),
        ("x = 1\nimport fs\n", "V0207"),
        ("from fs import read_text\n", "V0206"),
        ("import read_text,\n    write_text from fs\n", "V0208"),
        ("x = value otherwise 0 + 1\n", "V0209"),
        ("x = value otherwise (return 1) + 2\n", "V0209"),
        ("x = [1, 2\n", "V0201"),
        ("for x in 1..\n    pass\n", "V0201"),
        ("x = user?\n", "V0201"),
        ("f(x) = 1\n", "V0201"),
        ("match\n    true -> 1\n", "V0201"),
        ("x = a && b\n", "V0201"),
        ("x = [..]\n", "V0201"),
        ("x = \"{1 + }\"\n", "V0201"),
        ("import a, b\n", "V0201"),
        ("trait T\n    x = 1\n", "V0201"),
        ("fn f()\n    type T\n        x: Int\n", "V0201"),
        ("x = ()\n", "V0201"),
        ("x = (name: 1)\n", "V0201"),
        ("users.filter(.age >= 18)\n", "V0201"),
        ("x = a..b..c\n", "V0201"),
        ("x = 1 + not true\n", "V0201"),
        ("x = value otherwise try f()\n", "V0201"),
    ];
    for (source, code) in cases {
        let parsed = parse(source, FileId(0));
        assert!(
            parsed.diagnostics.iter().any(|d| d.code == code),
            "accepted {source:?}, expected {code}: {:?}",
            parsed.diagnostics
        );
        assert_eq!(parsed.syntax().text().to_string(), source);
    }
}

#[test]
fn s3_8_comparisons_and_associated_generics_have_different_trees() {
    let p = parse("x = a.b < c > (d)\n", FileId(0));
    assert_eq!(
        p.syntax()
            .descendants()
            .filter(|n| n.kind() == Kind::Comparison)
            .count(),
        2
    );
    let p = parse("x = Channel.new<Int>(capacity: 1)\n", FileId(0));
    assert_eq!(
        p.syntax()
            .descendants()
            .filter(|n| n.kind() == Kind::Comparison)
            .count(),
        0
    );
}
