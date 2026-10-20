use crate::{Diagnostic, FileId, Kind, Span};
use unicode_general_category::{GeneralCategory, get_general_category};

pub const KEYWORDS: &[&str] = &[
    "alias",
    "and",
    "as",
    "assert",
    "break",
    "continue",
    "else",
    "export",
    "extern",
    "false",
    "fn",
    "for",
    "from",
    "if",
    "impl",
    "import",
    "in",
    "is",
    "match",
    "mut",
    "none",
    "not",
    "or",
    "otherwise",
    "pass",
    "raises",
    "return",
    "self",
    "Self",
    "set",
    "spawn",
    "take",
    "test",
    "then",
    "trait",
    "true",
    "try",
    "type",
    "while",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub span: Span,
}
impl Token {
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        &source[self.span.start as usize..self.span.end as usize]
    }
}

#[derive(Debug, Clone)]
pub struct Interpolation {
    pub expression: Span,
}

#[derive(Debug, Clone)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
    pub interpolations: Vec<Interpolation>,
}

pub fn lex(source: &str, file: FileId) -> Lexed {
    let mut scanner = Scanner {
        source,
        file,
        pos: 0,
        tokens: vec![],
        diagnostics: vec![],
        interpolations: vec![],
    };
    scanner.scan();
    let tokens = layout(source, file, &scanner.tokens, &mut scanner.diagnostics);
    Lexed {
        tokens,
        diagnostics: scanner.diagnostics,
        interpolations: scanner.interpolations,
    }
}

pub fn decode(bytes: &[u8], file: FileId) -> Result<&str, Box<Diagnostic>> {
    std::str::from_utf8(bytes).map_err(|e| {
        Box::new(Diagnostic::simple(
            "V0109",
            Span::new(
                file,
                e.valid_up_to(),
                e.valid_up_to() + e.error_len().unwrap_or(1),
            ),
        ))
    })
}

struct Scanner<'a> {
    source: &'a str,
    file: FileId,
    pos: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
    interpolations: Vec<Interpolation>,
}

fn letter(c: char) -> bool {
    matches!(
        get_general_category(c),
        GeneralCategory::UppercaseLetter
            | GeneralCategory::LowercaseLetter
            | GeneralCategory::TitlecaseLetter
            | GeneralCategory::ModifierLetter
            | GeneralCategory::OtherLetter
    )
}
fn digit(c: char) -> bool {
    get_general_category(c) == GeneralCategory::DecimalNumber
}
fn upper(c: char) -> bool {
    matches!(
        get_general_category(c),
        GeneralCategory::UppercaseLetter | GeneralCategory::TitlecaseLetter
    )
}

impl Scanner<'_> {
    fn span(&self, start: usize) -> Span {
        Span::new(self.file, start, self.pos)
    }
    fn peek(&self) -> char {
        self.source[self.pos..].chars().next().unwrap_or('\0')
    }
    fn starts(&self, s: &str) -> bool {
        self.source[self.pos..].starts_with(s)
    }
    fn advance(&mut self) -> char {
        let c = self.peek();
        if self.pos < self.source.len() {
            self.pos += c.len_utf8();
        }
        c
    }
    fn error(&mut self, code: &str, start: usize) {
        self.diagnostics
            .push(Diagnostic::simple(code, self.span(start)));
    }

    fn scan(&mut self) {
        while self.pos < self.source.len() {
            let start = self.pos;
            let c = self.peek();
            let kind = match c {
                ' ' | '\t' => {
                    while matches!(self.peek(), ' ' | '\t') {
                        self.advance();
                    }
                    Kind::Whitespace
                }
                '\r' | '\n' => {
                    self.advance();
                    if c == '\r' && self.peek() == '\n' {
                        self.advance();
                    } else if c == '\r' {
                        self.diagnostics
                            .push(Diagnostic::simple("V0111", self.span(start)));
                        self.tokens.push(Token {
                            kind: Kind::ErrorToken,
                            span: self.span(start),
                        });
                        continue;
                    }
                    Kind::Newline
                }
                '\0' => {
                    self.advance();
                    self.diagnostics
                        .push(Diagnostic::simple("V0112", self.span(start)));
                    Kind::ErrorToken
                }
                '\u{feff}' => {
                    self.advance();
                    if start == 0 {
                        self.diagnostics
                            .push(Diagnostic::simple("V0113", self.span(start)));
                    }
                    Kind::ErrorToken
                }
                '#' => {
                    while self.pos < self.source.len() && !matches!(self.peek(), '\r' | '\n') {
                        self.advance();
                    }
                    if self.source[start..].starts_with("##") {
                        Kind::DocComment
                    } else {
                        Kind::Comment
                    }
                }
                'r' | 'b' if self.source[self.pos + 1..].starts_with('"') => {
                    self.advance();
                    self.string(c == 'r', c == 'b', 0);
                    if c == 'r' { Kind::RawText } else { Kind::Bytes }
                }
                '"' => {
                    self.string(false, false, 0);
                    Kind::Text
                }
                '\'' => {
                    if self.character() {
                        Kind::Char
                    } else {
                        Kind::ErrorToken
                    }
                }
                '0'..='9' => self.number(start),
                '_' => {
                    self.identifier();
                    if KEYWORDS.contains(&&self.source[start..self.pos]) {
                        Kind::Keyword
                    } else {
                        Kind::Ident
                    }
                }
                c if letter(c) => {
                    self.identifier();
                    let name = &self.source[start..self.pos];
                    if KEYWORDS.contains(&name) {
                        Kind::Keyword
                    } else if upper(c) {
                        Kind::TypeIdent
                    } else {
                        Kind::Ident
                    }
                }
                _ => {
                    let symbol = [
                        "..=", "**", "->", "..", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=",
                        "%=",
                    ]
                    .into_iter()
                    .find(|s| self.starts(s));
                    if let Some(symbol) = symbol {
                        self.pos += symbol.len();
                        Kind::Symbol
                    } else {
                        self.advance();
                        if "()[]{}.,:?|+-*/%=<>".contains(c) {
                            Kind::Symbol
                        } else {
                            Kind::ErrorToken
                        }
                    }
                }
            };
            self.tokens.push(Token {
                kind,
                span: self.span(start),
            });
        }
    }

    fn identifier(&mut self) {
        self.advance();
        while self.pos < self.source.len()
            && (letter(self.peek()) || digit(self.peek()) || self.peek() == '_')
        {
            self.advance();
        }
    }

    fn number(&mut self, start: usize) -> Kind {
        let after_dot = self
            .tokens
            .iter()
            .rev()
            .find(|t| !t.kind.is_trivia())
            .is_some_and(|t| t.text(self.source) == ".");
        let mut kind = Kind::Int;
        let mut base = 10;
        if self.starts("0x") || self.starts("0o") || self.starts("0b") {
            base = match self.source.as_bytes()[self.pos + 1] {
                b'x' => 16,
                b'o' => 8,
                _ => 2,
            };
            self.pos += 2;
            while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
                self.advance();
            }
        } else {
            while self.peek().is_ascii_digit() || self.peek() == '_' {
                self.advance();
            }
            if !after_dot
                && self.peek() == '.'
                && !self.starts("..")
                && self.source[self.pos + 1..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
            {
                kind = Kind::Float;
                self.advance();
                while self.peek().is_ascii_digit() || self.peek() == '_' {
                    self.advance();
                }
            }
            if !after_dot && matches!(self.peek(), 'e' | 'E') {
                kind = Kind::Float;
                self.advance();
                if matches!(self.peek(), '+' | '-') {
                    self.advance();
                }
                while self.peek().is_ascii_digit() || self.peek() == '_' {
                    self.advance();
                }
            }
            if letter(self.peek()) {
                while letter(self.peek()) || digit(self.peek()) || self.peek() == '_' {
                    self.advance();
                }
            }
        }
        let text = &self.source[start..self.pos];
        let cleaned = text.replace('_', "");
        let digits = if base == 10 {
            cleaned.as_str()
        } else {
            &cleaned[2..]
        };
        let valid = if kind == Kind::Float {
            cleaned.parse::<f64>().is_ok()
        } else {
            !digits.is_empty() && digits.chars().all(|c| c.is_digit(base))
        };
        let bad_separator = text.ends_with('_')
            || text.contains("__")
            || (base == 10
                && (text.contains("._")
                    || text.contains("_e")
                    || text.contains("_E")
                    || text.contains("e_")
                    || text.contains("E_")))
            || (base != 10 && text.as_bytes().get(2) == Some(&b'_'));
        let trailing_dot = !after_dot
            && self.peek() == '.'
            && !self.starts("..")
            && self.source[self.pos + 1..]
                .chars()
                .next()
                .is_none_or(|c| !letter(c) && c != '_');
        if !valid || bad_separator || trailing_dot {
            self.diagnostics.push(Diagnostic::new(
                "V0107",
                self.span(start),
                [
                    ("text", text.into()),
                    (
                        "kind",
                        if kind == Kind::Float { "Float" } else { "Int" }.into(),
                    ),
                ],
            ));
        } else if kind == Kind::Int {
            // A leading `-` negates only when it is a prefix operator, not a
            // binary one: `-9223372036854775808` is the minimum Int, while
            // `1 - 9223372036854775808` overflows.
            let mut trailing = self.tokens.iter().rev().filter(|t| !t.kind.is_trivia());
            let minus = trailing.next().is_some_and(|t| t.text(self.source) == "-");
            let operand = trailing.next().is_some_and(|t| {
                matches!(
                    t.kind,
                    Kind::Ident
                        | Kind::TypeIdent
                        | Kind::Int
                        | Kind::Float
                        | Kind::Text
                        | Kind::RawText
                        | Kind::Char
                        | Kind::Bytes
                ) || matches!(
                    t.text(self.source),
                    ")" | "]" | "}" | "true" | "false" | "none" | "self" | "Self"
                )
            });
            let negative = minus && !operand;
            let max = i64::MAX as u128 + u128::from(negative);
            if u128::from_str_radix(digits, base).map_or(true, |n| n > max) {
                self.diagnostics.push(Diagnostic::new(
                    "V0108",
                    self.span(start),
                    [("text", text.into())],
                ));
            }
        }
        kind
    }

    fn escape(&mut self) {
        let start = self.pos;
        self.advance();
        // A backslash before a newline is not an escape. Leave the newline for
        // the enclosing single-line text so it reports the text is unterminated
        // instead of swallowing the next physical line.
        if matches!(self.peek(), '\r' | '\n') {
            let c = if self.peek() == '\r' { "\\r" } else { "\\n" };
            self.diagnostics.push(
                Diagnostic::new("V0104", self.span(start), [("c", c.into())])
                    .with_edit(Span::new(self.file, start, start), "\\"),
            );
            return;
        }
        let c = self.advance();
        if c == 'u' && self.peek() == '{' {
            self.advance();
            let digits = self.pos;
            while self.peek().is_ascii_hexdigit() {
                self.advance();
            }
            let value = u32::from_str_radix(&self.source[digits..self.pos], 16)
                .ok()
                .and_then(char::from_u32);
            if self.peek() == '}' && value.is_some() {
                self.advance();
                return;
            }
        } else if "ntr\\\"{}0".contains(c) && c != '\0' {
            return;
        }
        self.diagnostics.push(
            Diagnostic::new("V0104", self.span(start), [("c", c.to_string().into())])
                .with_edit(Span::new(self.file, start, start), "\\"),
        );
    }

    fn character(&mut self) -> bool {
        let start = self.pos;
        self.advance();
        let mut count = 0;
        while self.pos < self.source.len() && !matches!(self.peek(), '\'' | '\r' | '\n') {
            if self.peek() == '\\' {
                self.escape();
            } else {
                self.advance();
            }
            count += 1;
        }
        if self.peek() == '\'' {
            self.advance();
            if count == 1 {
                true
            } else {
                self.diagnostics.push(Diagnostic::new(
                    "V0114",
                    self.span(start),
                    [("n", count.into())],
                ));
                false
            }
        } else {
            self.error("V0103", start);
            false
        }
    }

    fn string(&mut self, raw: bool, bytes: bool, nesting: usize) {
        let start = self.pos;
        let triple = self.starts("\"\"\"");
        let delimiter = if triple { "\"\"\"" } else { "\"" };
        self.pos += delimiter.len();
        if triple && !matches!(self.peek(), '\r' | '\n') {
            self.error("V0105", start);
        }
        while self.pos < self.source.len() {
            if self.starts(delimiter) {
                self.pos += delimiter.len();
                return;
            }
            if !triple && matches!(self.peek(), '\r' | '\n') {
                self.diagnostics.push(
                    Diagnostic::simple("V0103", self.span(start))
                        .with_edit(Span::new(self.file, self.pos, self.pos), delimiter),
                );
                return;
            }
            if !raw && self.peek() == '\\' {
                self.escape();
            } else if !raw && !bytes && self.peek() == '{' {
                let brace = self.pos;
                self.advance();
                if nesting > 0 {
                    self.error("V0106", brace);
                    // Bound recursive scanning even for malicious nested literals.
                    while self.pos < self.source.len() && !matches!(self.peek(), '}' | '\r' | '\n')
                    {
                        self.advance();
                    }
                    if self.peek() == '}' {
                        self.advance();
                    }
                } else if !self.interpolation(triple) {
                    if self.pos < self.source.len() {
                        continue;
                    }
                    return;
                }
            } else {
                self.advance();
            }
        }
        self.diagnostics.push(
            Diagnostic::simple("V0103", self.span(start))
                .with_edit(Span::new(self.file, self.pos, self.pos), delimiter),
        );
    }

    fn interpolation(&mut self, triple: bool) -> bool {
        let start = self.pos;
        let mut brackets = Vec::new();
        while self.pos < self.source.len() {
            if !triple && matches!(self.peek(), '\r' | '\n') {
                // The enclosing text owns the diagnostic for an unterminated
                // interpolation. Do not report it once per line.
                return false;
            }
            match self.peek() {
                'r' | 'b' if self.source[self.pos + 1..].starts_with('"') => {
                    let prefix = self.advance();
                    self.string(prefix == 'r', prefix == 'b', 1);
                }
                '"' => self.string(false, false, 1),
                '#' => {
                    while self.pos < self.source.len() && !matches!(self.peek(), '\r' | '\n') {
                        self.advance();
                    }
                }
                '\'' => {
                    self.character();
                }
                '(' | '[' | '{' => {
                    brackets.push(self.advance());
                }
                ')' | ']' => {
                    brackets.pop();
                    self.advance();
                }
                '}' if !brackets.is_empty() => {
                    brackets.pop();
                    self.advance();
                }
                ':' if brackets.is_empty() => {
                    self.interpolations.push(Interpolation {
                        expression: Span::new(self.file, start, self.pos),
                    });
                    while self.pos < self.source.len() && self.peek() != '}' {
                        if !triple && matches!(self.peek(), '\r' | '\n') {
                            return false;
                        }
                        self.advance();
                    }
                    if self.peek() == '}' {
                        self.advance();
                        return true;
                    }
                    self.error("V0103", start);
                    return false;
                }
                '}' => {
                    self.interpolations.push(Interpolation {
                        expression: Span::new(self.file, start, self.pos),
                    });
                    self.advance();
                    return true;
                }
                _ => {
                    self.advance();
                }
            }
        }
        self.error("V0103", start);
        false
    }
}

fn binary(text: &str) -> bool {
    matches!(
        text,
        "+" | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            | "=="
            | "!="
            | "<"
            | "<="
            | ">"
            | ">="
            | "and"
            | "or"
            | "otherwise"
            | "in"
            | "is"
            | "="
            | "|"
            | ".."
            | "..="
    )
}

/// Insert layout without losing physical whitespace. A block lambda temporarily
/// establishes a layout floor inside its call's opening parenthesis.
fn layout(
    source: &str,
    file: FileId,
    raw: &[Token],
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<Token> {
    let mut out = Vec::with_capacity(raw.len() + raw.len() / 8);
    let mut next_code = vec![None; raw.len() + 1];
    for i in (0..raw.len()).rev() {
        next_code[i] = if raw[i].kind.is_trivia() || raw[i].kind == Kind::Newline {
            next_code[i + 1]
        } else {
            Some(i)
        };
    }
    let mut indents = vec![0usize];
    let mut brackets: Vec<(&str, usize)> = vec![];
    let mut lambdas: Vec<(usize, usize, bool)> = vec![]; // header indentation, layout floor, implicit close
    let mut line_start = true;
    let mut continuation = false;
    let mut indentation = 0;
    let mut previous = "";
    let mut line_number = 0;
    let mut line_has_code = false;
    let mut skip_explicit = 0;
    let mut closed_explicit_line = false;
    let mut angle_depth = 0usize;
    let mut previous_type = false;
    let mut generic_close = false;
    for (index, &token) in raw.iter().enumerate() {
        let text = token.text(source);
        if line_start && token.kind == Kind::Whitespace {
            indentation = text.chars().map(|c| if c == '\t' { 4 } else { 1 }).sum();
            if text.contains('\t') {
                diagnostics.push(
                    Diagnostic::simple("V0101", token.span)
                        .with_edit(token.span, " ".repeat(indentation)),
                );
            }
            out.push(token);
            continue;
        }
        if token.kind == Kind::Newline {
            let next_index = next_code[index + 1];
            let next_text = next_index.map_or("", |i| raw[i].text(source));
            let floor = lambdas.last().map_or(0, |(_, i, _)| i + 1);
            let arrow_block = line_has_code && previous == "->";
            if arrow_block && !brackets.is_empty() {
                let implicit = brackets
                    .iter()
                    .rposition(|(s, line)| *s == "(" && *line == line_number);
                lambdas.push((
                    indentation,
                    implicit.unwrap_or(brackets.len() - 1),
                    implicit.is_some(),
                ));
            }
            let can_end = matches!(previous, ")" | "]" | "}")
                || previous
                    .chars()
                    .last()
                    .is_some_and(|c| c.is_alphanumeric() || "_'\"?".contains(c));
            let next_indent = raw
                .get(index + 1)
                .filter(|token| token.kind == Kind::Whitespace)
                .map_or(0, |token| {
                    token
                        .text(source)
                        .chars()
                        .map(|c| if c == '\t' { 4 } else { 1 })
                        .sum()
                });
            // An unclosed bracket must not turn every later line into the same
            // logical line. A top-level declaration is a reliable recovery
            // point, so end the continuation and let the parser report the
            // missing close at the declaration boundary. A `fn` at column zero
            // can instead be a function type inside a bracketed type, so only a
            // `fn` that is not followed by `(` is a declaration.
            let recover = next_indent == 0
                && !brackets.is_empty()
                && next_index.is_some_and(|i| {
                    let text = raw[i].text(source);
                    if text == "fn" {
                        !next_code[i + 1].is_some_and(|j| raw[j].text(source) == "(")
                    } else {
                        matches!(
                            text,
                            "type"
                                | "alias"
                                | "trait"
                                | "impl"
                                | "extern"
                                | "test"
                                | "import"
                                | "export"
                        )
                    }
                });
            if recover {
                brackets.clear();
            }
            continuation = !recover
                && !arrow_block
                && (brackets.len() > floor
                    || (binary(previous) && !generic_close)
                    || (can_end && next_text == ".")
                    || next_text == "|");
            let kind = if !line_has_code {
                Kind::Blank
            } else if continuation {
                Kind::Whitespace
            } else {
                Kind::Newline
            };
            out.push(Token { kind, ..token });
            if !continuation && line_has_code {
                previous = "";
            }
            line_start = true;
            indentation = 0;
            line_has_code = false;
            closed_explicit_line = false;
            line_number += 1;
            continue;
        }
        if token.kind == Kind::Comment {
            out.push(token);
            continue;
        }
        if token.kind == Kind::DocComment && !brackets.is_empty() {
            // A doc comment inside an open bracket is an ordinary comment, not
            // documentation for a following item, so it is trivia there.
            out.push(Token {
                kind: Kind::Comment,
                ..token
            });
            continue;
        }
        if line_start {
            // The dedent closes lambda calls before checking ordinary bracket continuation.
            while lambdas
                .last()
                .is_some_and(|(indent, _, _)| indentation <= *indent)
            {
                let (header_indent, bracket_index, implicit) = lambdas.pop().unwrap();
                while indents.len() > 1 && *indents.last().unwrap() > header_indent {
                    indents.pop();
                    out.push(Token {
                        kind: Kind::Dedent,
                        span: Span::new(file, token.span.start as usize, token.span.start as usize),
                    });
                }
                if implicit {
                    brackets.truncate(bracket_index);
                    out.push(Token {
                        kind: Kind::ImplicitClose,
                        span: Span::new(file, token.span.start as usize, token.span.start as usize),
                    });
                    if text == ")" {
                        skip_explicit += 1;
                        closed_explicit_line = true;
                    }
                }
            }
            if !continuation {
                let current = *indents.last().unwrap();
                let levels = indents.clone();
                // A line deeper than the current block opens exactly one new
                // level, regardless of how many extra spaces it has. A dedent
                // pops to the greatest open level at or below its column.
                let target_index = if indentation > current {
                    levels.len()
                } else {
                    levels
                        .iter()
                        .rposition(|level| *level <= indentation)
                        .unwrap_or(0)
                };
                let canonical_target = target_index * 4;
                let max_lower = levels
                    .iter()
                    .copied()
                    .filter(|level| *level < indentation)
                    .max();
                let distance = |level: usize| level.abs_diff(indentation);
                let nearest_distance = levels.iter().map(|level| distance(*level)).min();
                let nearest: Vec<_> = levels
                    .iter()
                    .copied()
                    .filter(|level| Some(distance(*level)) == nearest_distance)
                    .collect();
                let fixable = indentation > current
                    || levels.contains(&indentation)
                    || (nearest.len() == 1 && Some(nearest[0]) == max_lower);
                if indentation % 4 != 0 {
                    diagnostics.push(Diagnostic::new(
                        "V0102",
                        token.span,
                        [
                            ("n", indentation.into()),
                            ("m", (indentation / 4 * 4).into()),
                            ("k", (indentation / 4 * 4 + 4).into()),
                            ("h", canonical_target.into()),
                            ("fixable", if fixable { "yes" } else { "no" }.into()),
                        ],
                    ));
                }
                if indentation > current {
                    indents.push(indentation);
                    out.push(Token {
                        kind: Kind::Indent,
                        span: Span::new(file, token.span.start as usize, token.span.start as usize),
                    });
                } else {
                    while indents.len() > 1 && indentation < *indents.last().unwrap() {
                        indents.pop();
                        out.push(Token {
                            kind: Kind::Dedent,
                            span: Span::new(
                                file,
                                token.span.start as usize,
                                token.span.start as usize,
                            ),
                        });
                    }
                    if indentation % 4 == 0 && indentation != *indents.last().unwrap() {
                        let lower = levels
                            .iter()
                            .copied()
                            .rev()
                            .find(|level| *level < indentation)
                            .unwrap_or(0);
                        let upper = levels
                            .iter()
                            .copied()
                            .find(|level| *level > indentation)
                            .unwrap_or(indentation + 4);
                        diagnostics.push(Diagnostic::new(
                            "V0102",
                            token.span,
                            [
                                ("n", indentation.into()),
                                ("m", lower.into()),
                                ("k", upper.into()),
                                ("h", canonical_target.into()),
                                ("fixable", "no".into()),
                            ],
                        ));
                    }
                }
            }
            line_start = false;
        }
        if token.kind != Kind::Whitespace {
            line_has_code = true;
            if text == ")" && skip_explicit > 0 {
                skip_explicit -= 1;
                out.push(Token {
                    kind: Kind::Whitespace,
                    ..token
                });
                continue;
            }
            if closed_explicit_line {
                closed_explicit_line = false;
                diagnostics.push(Diagnostic::new(
                    "V0201",
                    token.span,
                    [
                        ("what", "end of line".into()),
                        ("found", text.to_owned().into()),
                    ],
                ));
            }
            match text {
                "(" | "[" | "{" => brackets.push((text, line_number)),
                ")" | "]" | "}" => {
                    brackets.pop();
                }
                _ => {}
            }
            generic_close = text == ">" && angle_depth > 0;
            if text == "<" && (previous_type || angle_depth > 0) {
                angle_depth += 1;
            }
            if generic_close {
                angle_depth -= 1;
            }
            previous_type = token.kind == Kind::TypeIdent;
            previous = text;
        }
        out.push(token);
    }
    let eof = Span::new(file, source.len(), source.len());
    if !previous.is_empty() {
        out.push(Token {
            kind: Kind::Newline,
            span: eof,
        });
    }
    for (header_indent, _, implicit) in lambdas.into_iter().rev() {
        while indents.len() > 1 && *indents.last().unwrap() > header_indent {
            indents.pop();
            out.push(Token {
                kind: Kind::Dedent,
                span: eof,
            });
        }
        if implicit {
            out.push(Token {
                kind: Kind::ImplicitClose,
                span: eof,
            });
        }
    }
    while indents.len() > 1 {
        indents.pop();
        out.push(Token {
            kind: Kind::Dedent,
            span: eof,
        });
    }
    out.push(Token {
        kind: Kind::Eof,
        span: eof,
    });
    out
}
