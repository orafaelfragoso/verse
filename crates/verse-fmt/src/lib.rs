//! Canonical CST-based formatting. The public formatter has no style settings.
use std::collections::BTreeSet;
use verse_syntax::{Diagnostic, FileId, Kind, SyntaxNode, parse};

pub fn format(source: &str, file: FileId) -> Result<String, Vec<Diagnostic>> {
    let parsed = parse(source, file);
    if parsed
        .diagnostics
        .iter()
        .any(|d| !matches!(d.code.as_str(), "V0101" | "V0102"))
    {
        return Err(parsed.diagnostics);
    }
    let mut current = source.to_owned();
    let original_diagnostics = parsed.diagnostics.clone();
    let mut parsed = parsed;
    // Wrapping a chain can make an enclosing list multiline; settle its commas
    // in the same invocation, rather than requiring the author to format twice.
    for _ in 0..8 {
        let formatted = format_syntax(&parsed.syntax());
        if formatted == current {
            return Ok(formatted);
        }
        current = formatted;
        parsed = parse(&current, file);
        if !parsed.is_valid() {
            if !original_diagnostics.is_empty() {
                return Err(original_diagnostics);
            }
            // A valid input whose canonical form does not parse is a formatter
            // bug, never an empty diagnostic list.
            break;
        }
    }
    Err(vec![Diagnostic::new(
        "V9901",
        verse_syntax::Span::new(file, 0, 0),
        [("stage", "formatting".into())],
    )])
}

fn format_syntax(root: &SyntaxNode) -> String {
    let mut comma_after = BTreeSet::new();
    let mut declaration_starts = BTreeSet::new();
    let mut chain_breaks = BTreeSet::new();
    fn chain_dots(node: &SyntaxNode, dots: &mut Vec<u32>) {
        let mut node = node.clone();
        while matches!(node.kind(), Kind::Call | Kind::FieldAccess | Kind::Index) {
            if node.kind() == Kind::FieldAccess
                && let Some(dot) = node
                    .children_with_tokens()
                    .filter_map(|e| e.into_token())
                    .find(|t| t.text() == ".")
            {
                dots.push(u32::from(dot.text_range().start()));
            }
            let Some(first) = node.children().next() else {
                break;
            };
            node = first;
        }
        dots.reverse();
    }
    for node in root.descendants() {
        if matches!(node.kind(), Kind::Call | Kind::FieldAccess)
            && !node
                .parent()
                .is_some_and(|p| matches!(p.kind(), Kind::Call | Kind::FieldAccess | Kind::Index))
            && node.text().len() > 88.into()
        {
            let mut dots = vec![];
            chain_dots(&node, &mut dots);
            if dots.len() >= 2 {
                chain_breaks.extend(dots);
            }
        }
        if node.parent().is_some_and(|p| p.kind() == Kind::File)
            && matches!(
                node.kind(),
                Kind::Function
                    | Kind::TypeDeclaration
                    | Kind::Trait
                    | Kind::Impl
                    | Kind::Extern
                    | Kind::Test
                    | Kind::Alias
            )
            && let Some(token) = node
                .descendants_with_tokens()
                .filter_map(|e| e.into_token())
                .find(|t| !t.kind().is_trivia() && !t.kind().is_layout())
        {
            declaration_starts.insert(u32::from(token.text_range().start()));
        }
        if matches!(
            node.kind(),
            Kind::Arguments
                | Kind::Parameters
                | Kind::List
                | Kind::Map
                | Kind::Set
                | Kind::Tuple
                | Kind::Import
        ) && node.text().to_string().contains('\n')
        {
            let tokens: Vec<_> = node
                .descendants_with_tokens()
                .filter_map(|e| e.into_token())
                .filter(|t| !t.kind().is_trivia() && !t.kind().is_layout())
                .collect();
            if tokens.len() >= 3 {
                let closing_index = if node.kind() == Kind::Import {
                    tokens
                        .iter()
                        .rposition(|t| t.text() == ")")
                        .unwrap_or(tokens.len() - 1)
                } else {
                    tokens.len() - 1
                };
                let close = &tokens[closing_index];
                let previous = &tokens[closing_index - 1];
                if matches!(close.text(), ")" | "]" | "}")
                    && !matches!(previous.text(), "," | ":" | "(" | "[" | "{")
                {
                    comma_after.insert(u32::from(previous.text_range().start()));
                }
            }
        }
    }
    let mut out = String::new();
    let mut line = String::new();
    let mut indent = 0usize;
    let mut physical_indent = 0usize;
    let mut previous = String::new();
    let mut at_line_start = true;
    let mut pending_blank = false;
    let mut continuation = false;
    let mut explicit_close = false;
    let mut previous_tight = false;
    let mut last_kind = None;
    let flush = |out: &mut String, line: &mut String| {
        if !line.trim().is_empty() {
            out.push_str(line.trim_end());
            out.push('\n');
        }
        line.clear();
    };
    for token in root
        .descendants_with_tokens()
        .filter_map(|e| e.into_token())
    {
        let text = token.text();
        let kind = token.kind();
        match kind {
            Kind::Indent => {
                indent += 1;
                continue;
            }
            Kind::Dedent => {
                indent = indent.saturating_sub(1);
                continue;
            }
            Kind::ImplicitClose => {
                explicit_close = true;
                continue;
            }
            Kind::Eof => continue,
            Kind::Whitespace | Kind::Blank | Kind::Newline => {
                if text.contains(['\r', '\n']) || kind == Kind::Newline {
                    if !at_line_start {
                        flush(&mut out, &mut line);
                    } else if kind == Kind::Blank {
                        pending_blank = true;
                    }
                    continuation = kind == Kind::Whitespace;
                    at_line_start = true;
                    physical_indent = 0;
                    if !continuation {
                        previous.clear();
                    }
                } else if at_line_start {
                    physical_indent += text
                        .chars()
                        .map(|c| if c == '\t' { 4 } else { 1 })
                        .sum::<usize>();
                }
                continue;
            }
            _ => {}
        }
        if explicit_close {
            if text == ")" {
                explicit_close = false;
                continue;
            }
            explicit_close = false;
        }
        if chain_breaks.contains(&u32::from(token.text_range().start())) && !at_line_start {
            flush(&mut out, &mut line);
            at_line_start = true;
            continuation = true;
            physical_indent = (indent + 1) * 4;
        }
        if at_line_start {
            let declaration = declaration_starts.contains(&u32::from(token.text_range().start()));
            if (pending_blank || declaration) && !out.is_empty() && !out.ends_with("\n\n") {
                out.push('\n');
            }
            pending_blank = false;
            let width = if continuation && !matches!(text, ")" | "]" | "}") {
                physical_indent.max((indent + 1) * 4)
            } else {
                indent * 4
            };
            line.push_str(&" ".repeat(width));
            previous.clear();
            at_line_start = false;
        }
        let generic = (matches!(text, "<" | ">") || previous == "<")
            && token.parent_ancestors().any(|n| {
                matches!(
                    n.kind(),
                    Kind::Type | Kind::Generics | Kind::GenericParameter
                )
            });
        let no_before = matches!(text, "," | ":" | ")" | "]" | "}" | "." | "?")
            || (generic && matches!(text, "<" | ">"));
        let no_after =
            matches!(previous.as_str(), "(" | "[" | "{" | ".") || (generic && previous == "<");
        let call = text == "("
            && !matches!(
                previous.as_str(),
                "import"
                    | "if"
                    | "while"
                    | "for"
                    | "return"
                    | "assert"
                    | "in"
                    | "otherwise"
                    | "="
                    | "->"
                    | "+"
                    | "-"
                    | "*"
                    | "/"
                    | "and"
                    | "or"
                    | "not"
                    | "try"
                    | "spawn"
            );
        let set = text == "{" && previous == "set";
        let comment = kind == Kind::Comment;
        if !previous.is_empty()
            && !no_before
            && !no_after
            && !call
            && !set
            && !previous_tight
            && !line.ends_with(' ')
        {
            line.push_str(if comment { "  " } else { " " });
        }
        line.push_str(text);
        if comma_after.contains(&u32::from(token.text_range().start())) {
            line.push(',');
        }
        previous_tight = text == "-" && token.parent().is_some_and(|n| n.kind() == Kind::Unary);
        previous = text.into();
        last_kind = Some(kind);
    }
    flush(&mut out, &mut line);
    // A module doc comment needs its separating blank line to stay a module
    // doc comment when no declaration follows it.
    if pending_blank && last_kind == Some(Kind::DocComment) && out.ends_with('\n') {
        out.push('\n');
    }
    sort_imports(&out)
}

fn sort_imports(source: &str) -> String {
    let mut out = String::new();
    let mut imports: Vec<String> = vec![];
    let flush = |out: &mut String, imports: &mut Vec<String>| {
        imports.sort();
        for import in imports.drain(..) {
            out.push_str(&import);
        }
    };
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next() {
        if line.starts_with("import ") && !line.contains('#') {
            let mut import = format!("{line}\n");
            if line.contains('(') && !line.contains(')') {
                while let Some(line) = lines.peek() {
                    let line = *line;
                    if !line.starts_with(' ') && !line.starts_with(')') {
                        break;
                    }
                    lines.next();
                    import.push_str(line);
                    import.push('\n');
                    if line.starts_with(')') {
                        break;
                    }
                }
            }
            imports.push(import);
        } else {
            flush(&mut out, &mut imports);
            out.push_str(line);
            out.push('\n');
        }
    }
    flush(&mut out, &mut imports);
    out
}

/// Naming conventions are checked without rewriting the author's identifiers.
pub fn naming_violations(source: &str, file: FileId) -> Vec<(verse_syntax::Span, String)> {
    verse_syntax::lex(source, file)
        .tokens
        .into_iter()
        .filter_map(|token| {
            let text = token.text(source);
            let bad = match token.kind {
                Kind::Ident => text.chars().any(|c| c.is_uppercase()),
                Kind::TypeIdent => text.contains('_'),
                _ => false,
            };
            bad.then(|| {
                (
                    token.span,
                    format!(
                        "`{text}` does not follow {} naming",
                        if token.kind == Kind::TypeIdent {
                            "PascalCase"
                        } else {
                            "snake_case"
                        }
                    ),
                )
            })
        })
        .collect()
}
