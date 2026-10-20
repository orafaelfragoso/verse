match value
    Record(name:, body: text: Text) -> print(text)
    Some(x)? -> x
    [first, ..rest] -> first
    (a, b) -> a + b
    1..=3 -> 1
    _ -> 0
