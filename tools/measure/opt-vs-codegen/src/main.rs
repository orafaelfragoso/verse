fn main() { let r = regex::Regex::new(r"\d+").unwrap(); let v: serde_json::Value = serde_json::from_str("[1,2]").unwrap(); println!("{} {}", r.is_match("a1"), v); }
