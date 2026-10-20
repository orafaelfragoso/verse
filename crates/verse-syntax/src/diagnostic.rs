use crate::{FileId, Kind, SourceFile, Span, lexer::lex};
use serde::Serialize;
use std::{collections::BTreeMap, fmt::Write};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum Fact {
    Text(String),
    Number(u64),
    Position(Span),
}
impl From<&str> for Fact {
    fn from(v: &str) -> Self {
        Self::Text(v.into())
    }
}
impl From<String> for Fact {
    fn from(v: String) -> Self {
        Self::Text(v)
    }
}
impl From<usize> for Fact {
    fn from(v: usize) -> Self {
        Self::Number(v as u64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Label {
    pub span: Span,
    pub label: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Edit {
    pub span: Span,
    pub replacement: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Suggestion {
    pub message: String,
    pub edits: Vec<Edit>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub code: String,
    pub severity: Severity,
    pub message: String,
    pub primary: Label,
    pub secondary: Vec<Label>,
    pub rule: String,
    pub help: Vec<Suggestion>,
    pub facts: BTreeMap<String, Fact>,
}

pub struct Definition {
    pub code: &'static str,
    pub message: &'static str,
    pub label: &'static str,
    pub help: &'static str,
    pub rule: &'static str,
    pub example: &'static str,
    pub reason: &'static str,
    pub what_happened: &'static str,
}

macro_rules! definitions {
    ($(($code:literal, $message:literal, $label:literal, $help:literal, $rule:literal, $example:literal, $reason:literal, $what_happened:literal)),* $(,)?) => {
        pub static DEFINITIONS: &[Definition] = &[$(Definition { code: $code, message: $message, label: $label, help: $help, rule: $rule, example: $example, reason: $reason, what_happened: $what_happened }),*];
    }
}
definitions![
    (
        "V0101",
        "a tab is not allowed in indentation",
        "this line starts with a tab",
        "indent with four spaces (applied by `verse fmt`)",
        "§3.3",
        "if true\n    pass",
        "Spaces make block boundaries unambiguous.",
        "Indentation is measured in spaces, and this line begins with a tab."
    ),
    (
        "V0102",
        "this line is indented by {n} spaces, not a multiple of 4",
        "expected {m} or {k} spaces",
        "indent with {h} spaces",
        "§3.3",
        "if true\n    pass",
        "Block lines share an indentation; canonical indentation is four spaces.",
        "This line is indented a number of spaces that is not a multiple of four."
    ),
    (
        "V0103",
        "this text is never closed",
        "the text starts here",
        "add the closing `\"`",
        "§3.7",
        "text = \"hello\"",
        "A literal needs a closing quote to delimit its contents.",
        "A text or character literal reaches the end of the line without its closing quote."
    ),
    (
        "V0104",
        "`\\{c}` is not an escape",
        "not one of `\\n \\t \\r \\\\ \\\" \\{ \\} \\0 \\u{…}`",
        "write `\\\\` for a backslash",
        "§3.7",
        "path = r\"C:\\files\"",
        "Escapes have explicit spellings; raw text needs no escapes.",
        "A backslash is followed by something that is not an escape."
    ),
    (
        "V0105",
        "a multi-line text must start on its own line",
        "the opening `\"\"\"` ends its line, but this one is followed by text",
        "move the text to the next line",
        "§3.7",
        "text = \"\"\"\n    hello\n    \"\"\"",
        "Triple-quoted text uses the closing indentation to strip indentation.",
        "The opening triple quotes are followed by text on the same line, so the literal has no clear start."
    ),
    (
        "V0106",
        "this interpolation is nested too deeply",
        "text inside `{…}` inside `{…}`",
        "bind the inner text to a name first",
        "§3.7",
        "inner = \"hello\"\ntext = \"{inner}\"",
        "One nested text level keeps interpolation readable.",
        "A text literal inside an interpolation contains another interpolation, one level too deep."
    ),
    (
        "V0107",
        "`{text}` is not a number",
        "cannot read this as {kind}",
        "write digits, with `_` between groups",
        "§3.7",
        "number = 1_000",
        "Numeric literals must follow their base and exponent grammar.",
        "Text in a number position cannot be read as a numeric literal."
    ),
    (
        "V0108",
        "the number {text} does not fit in `Int`",
        "the largest `Int` is 9,223,372,036,854,775,807",
        "use a `Float`, or a sized type from `math`",
        "§4.1",
        "number = 9223372036854775807",
        "Int is a signed 64-bit integer on every target.",
        "An integer literal exceeds what a 64-bit signed Int can hold."
    ),
    (
        "V0109",
        "this file is not valid UTF-8",
        "the first bad byte is here",
        "save the file as UTF-8",
        "§3.1",
        "text = \"é\"",
        "Every Verse source file is UTF-8.",
        "The file is not valid UTF-8."
    ),
    (
        "V0110",
        "`{name}` does not follow {style} naming",
        "rename this {kind}",
        "use {style} for {kind}s",
        "§3.5",
        "user_name = 1",
        "Names are lowercase or PascalCase according to what they name.",
        "A name uses a capitalization that does not match what it names."
    ),
    (
        "V0111",
        "a bare carriage return is not a line ending",
        "write `\\n` or `\\r\\n` here",
        "replace this byte with a newline",
        "§3.1",
        "first = 1\nsecond = 2",
        "Every line ends with a newline or a carriage-return-newline pair.",
        "A carriage return appears without a following newline."
    ),
    (
        "V0112",
        "NUL is not allowed in source",
        "remove this byte",
        "delete the NUL byte",
        "§3.1",
        "text = \"hello\"",
        "Source files contain text, not binary control characters.",
        "A NUL byte appears in the source."
    ),
    (
        "V0113",
        "the file starts with a byte-order mark",
        "remove the BOM",
        "save the file as UTF-8 without a BOM",
        "§3.1",
        "text = \"hello\"",
        "A BOM is metadata, not part of the program.",
        "The file begins with a byte-order mark."
    ),
    (
        "V0114",
        "a character literal must contain exactly one character",
        "this literal has {n} characters",
        "write one character, or use a text literal",
        "§3.7",
        "letter = 'a'",
        "A Char is one Unicode scalar value.",
        "A character literal holds more or fewer than one character."
    ),
    (
        "V0201",
        "expected {what} here",
        "found {found}",
        "",
        "§3",
        "fn example()\n    pass",
        "Declarations and expressions have explicit grammatical boundaries.",
        "The grammar expects one thing here, and the source has something else."
    ),
    (
        "V0202",
        "this block is empty",
        "a block needs at least one line",
        "write `pass`, or remove the line",
        "§6.6",
        "if true\n    pass",
        "An intentionally empty block is written pass.",
        "A block has no lines between its header and its dedent."
    ),
    (
        "V0203",
        "`{kw}` is a keyword and cannot be a name",
        "rename it",
        "try `{kw}_` or a more specific name",
        "§3.6",
        "fn example()\n    pass",
        "Keywords identify language constructs, not user bindings.",
        "A keyword appears where a name is required."
    ),
    (
        "V0206",
        "`from` must come last in an import",
        "the item list goes first",
        "write `import {names} from {module}`",
        "§12.2",
        "import read_text from fs",
        "Imports always start with import; from distinguishes item imports.",
        "An import puts `from` before its item list, which is not a valid import order."
    ),
    (
        "V0207",
        "an import must be at the top of the file",
        "imports come right after the doc comment",
        "move this line up",
        "§12.2",
        "import fs\ntext = \"hello\"",
        "A module's dependencies are visible at the start of the file.",
        "An import appears after other declarations, below the top of the file."
    ),
    (
        "V0208",
        "a long import needs parentheses",
        "the list continues on the next line",
        "wrap the names in `( … )`",
        "§12.2",
        "import (\n    read_text,\n    write_text,\n) from fs",
        "Only parenthesized import lists may span physical lines.",
        "An import list continues onto the next line without parentheses."
    ),
    (
        "V0209",
        "`{op}` cannot be in the right operand of `otherwise` without parentheses",
        "`otherwise` binds more loosely than `{op}`",
        "write `({left} otherwise {right}) {op} {rest}` or `{left} otherwise ({right} {op} {rest})`",
        "§3.8.2",
        "value = count otherwise (0 + 1)",
        "Restricted fallback operands prevent surprising precedence.",
        "An operator appears inside the right operand of `otherwise` without parentheses."
    ),
    (
        "V0210",
        "`{}` does not say whether it is a map or a set",
        "this could be an empty map or an empty set",
        "write `{:}` for an empty map, or `set{}` for an empty set",
        "§3.7",
        "settings = {:}",
        "Maps and sets have distinct empty literal spellings.",
        "An empty brace pair does not say whether it is a map or a set."
    ),
    (
        "V1101",
        "no command `{word}`; to run a file, write `{word}.vs`",
        "",
        "run `verse` with no arguments to see the commands",
        "19 §19.2.1",
        "print(\"hello\")",
        "A command name and a source path are distinguished by their spelling.",
        "The command line names a command that `verse` does not have."
    ),
    (
        "V9901",
        "the compiler failed in {stage}; this is a bug in Verse, not in your program",
        "",
        "run `verse check --report` and send the file",
        "02 §2.2",
        "print(\"hello\")",
        "A stage must establish its output invariants. This failure is an implementation bug.",
        "A compiler stage failed to uphold its output contract."
    ),
];

impl Diagnostic {
    pub fn new(
        code: &str,
        span: Span,
        facts: impl IntoIterator<Item = (&'static str, Fact)>,
    ) -> Self {
        let definition = DEFINITIONS
            .iter()
            .find(|d| d.code == code)
            .expect("undefined diagnostic");
        let facts: BTreeMap<String, Fact> = facts.into_iter().map(|(k, v)| (k.into(), v)).collect();
        let expand = |template: &str| {
            let mut s = String::new();
            let mut remaining = template;
            while let Some(start) = remaining.find('{') {
                s.push_str(&remaining[..start]);
                remaining = &remaining[start..];
                let Some(end) = remaining.find('}') else {
                    break;
                };
                match facts.get(&remaining[1..end]) {
                    Some(Fact::Text(value)) => s.push_str(value),
                    Some(Fact::Number(value)) => s.push_str(&value.to_string()),
                    Some(Fact::Position(value)) => {
                        let _ = write!(s, "{}:{}", value.start, value.end);
                    }
                    None => s.push_str(&remaining[..=end]),
                }
                remaining = &remaining[end + 1..];
            }
            s.push_str(remaining);
            s
        };
        Self {
            code: code.into(),
            severity: Severity::Error,
            message: expand(definition.message),
            primary: Label {
                span,
                label: expand(definition.label),
            },
            secondary: vec![],
            rule: definition.rule.into(),
            help: if definition.help.is_empty() {
                vec![]
            } else {
                vec![Suggestion {
                    message: expand(definition.help),
                    edits: vec![],
                }]
            },
            facts,
        }
    }

    pub fn simple(code: &str, span: Span) -> Self {
        Self::new(code, span, [])
    }

    pub fn with_edit(mut self, span: Span, replacement: impl Into<String>) -> Self {
        if let Some(help) = self.help.first_mut() {
            help.edits.push(Edit {
                span,
                replacement: replacement.into(),
            });
        }
        self
    }

    pub fn render(&self, source: &SourceFile) -> String {
        let (line, col) = source.position(self.primary.span.start);
        let (end_line, end_col) = source.position(self.primary.span.end);
        let width = line.to_string().len();
        let severity = match self.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
        };
        // A span that ends on a later line marks the rest of its first line.
        let caret_len = if end_line == line {
            end_col.saturating_sub(col).max(1)
        } else {
            source
                .line(line)
                .chars()
                .count()
                .saturating_sub(col - 1)
                .max(1)
        };
        let label = if self.primary.label.is_empty() {
            String::new()
        } else {
            format!(" {}", self.primary.label)
        };
        let mut out = format!(
            "{severity}: {}\n  --> {}:{line}:{col}\n{:width$} |\n{line:width$} | {}\n{:width$} | {}{}{}\n",
            self.message,
            source.name,
            "",
            source.line(line),
            "",
            " ".repeat(col - 1),
            "^".repeat(caret_len),
            label,
        );
        for label in &self.secondary {
            let (l, c) = source.position(label.span.start);
            let _ = writeln!(out, "  --> {}:{l}:{c}: {}", source.name, label.label);
        }
        let _ = writeln!(out, "rule: {}", self.rule);
        for help in &self.help {
            let _ = writeln!(out, "help: {}", help.message);
        }
        let _ = writeln!(
            out,
            "explain: {} (run `verse explain {}`)",
            self.code, self.code
        );
        out
    }
}

/// Naming conventions are checked without rewriting the author's identifiers.
/// A foreign function's C name is not Verse code, so only its `as` alias is
/// checked.
pub fn naming_diagnostics(source: &str, file: FileId) -> Vec<Diagnostic> {
    let tokens = lex(source, file).tokens;
    let code: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| !token.kind.is_trivia() && !token.kind.is_layout())
        .map(|(index, _)| index)
        .collect();
    let mut out = Vec::new();
    for (position, index) in code.iter().copied().enumerate() {
        let token = &tokens[index];
        let text = token.text(source);
        let style = match token.kind {
            Kind::Ident if text.chars().any(|c| c.is_uppercase()) => "snake_case",
            Kind::TypeIdent if text.contains('_') => "PascalCase",
            _ => continue,
        };
        let previous = position
            .checked_sub(1)
            .map(|position| tokens[code[position]].text(source));
        let next = code
            .get(position + 1)
            .map(|index| tokens[*index].text(source));
        if previous == Some("fn") && next == Some("as") {
            continue;
        }
        out.push(Diagnostic::new(
            "V0110",
            token.span,
            [
                ("name", text.to_owned().into()),
                ("style", style.into()),
                (
                    "kind",
                    if token.kind == Kind::TypeIdent {
                        "type"
                    } else {
                        "name"
                    }
                    .into(),
                ),
            ],
        ));
    }
    out
}

pub fn explain(code: &str) -> Option<String> {
    let d = DEFINITIONS.iter().find(|d| d.code == code)?;
    Some(format!(
        "{}  {}\n\nWhat happened\n{}\n\nWhy Verse works this way\n{} ({})\n\nWays to fix it\n1. {}\n\nExample\n{}\n",
        d.code,
        d.message,
        d.what_happened,
        d.reason,
        d.rule,
        if d.help.is_empty() {
            "Use the syntax shown in the example."
        } else {
            d.help
        },
        d.example
    ))
}
