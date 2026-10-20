//! Canonical CST-based formatting. The public formatter has no style settings.
use std::collections::BTreeSet;
use verse_syntax::{Diagnostic, FileId, Kind, SyntaxNode, SyntaxToken, parse};

/// Formatting may repair tabs and unambiguous indentation mistakes. Anything
/// else is a source error and is reported instead of guessed at.
fn fixable(diagnostic: &Diagnostic) -> bool {
    match diagnostic.code.as_str() {
        "V0101" => true,
        "V0102" => diagnostic.facts.get("fixable").is_some_and(
            |fact| matches!(fact, verse_syntax::diagnostic::Fact::Text(text) if text == "yes"),
        ),
        _ => false,
    }
}

pub fn format(source: &str, file: FileId) -> Result<String, Vec<Diagnostic>> {
    let parsed = parse(source, file);
    if parsed.diagnostics.iter().any(|d| !fixable(d)) {
        return Err(parsed.diagnostics);
    }
    let original = structure(&parsed.syntax());
    let mut current = source.to_owned();
    let mut parsed = parsed;
    // Wrapping a chain can make an enclosing list multiline; settle its commas
    // in the same invocation, rather than requiring the author to format twice.
    for _ in 0..8 {
        let formatted = format_syntax(&parsed.syntax());
        if formatted == current {
            let reparsed = parse(&formatted, file);
            if !reparsed.is_valid() || structure(&reparsed.syntax()) != original {
                return Err(internal_failure(file));
            }
            return Ok(formatted);
        }
        current = formatted;
        parsed = parse(&current, file);
        if !parsed.is_valid() {
            // A valid input whose canonical form does not parse is a formatter
            // bug, never a source diagnostic.
            break;
        }
    }
    Err(internal_failure(file))
}

fn internal_failure(file: FileId) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        "V9901",
        verse_syntax::Span::new(file, 0, 0),
        [("stage", "formatting".into())],
    )]
}

/// Everything formatting must not change: declarations in order, imports as a
/// set, and every comment. Comparing this before and after catches a formatter
/// that silently rewrites a text literal or loses a comment.
fn structure(root: &SyntaxNode) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut items = Vec::new();
    let mut imports = Vec::new();
    for child in root.children() {
        if child.kind() == Kind::Import {
            imports.push(verse_syntax::tree::fingerprint(&child));
        } else {
            items.push(verse_syntax::tree::fingerprint(&child));
        }
    }
    imports.sort();
    let mut comments: Vec<_> = root
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| matches!(token.kind(), Kind::Comment | Kind::DocComment))
        .map(|token| token.text().trim_end().to_owned())
        .collect();
    comments.sort();
    (items, imports, comments)
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
        ) && node.descendants_with_tokens().any(|element| {
            element.as_token().is_some_and(|token| {
                !matches!(token.kind(), Kind::Text | Kind::RawText) && token.text().contains('\n')
            })
        }) {
            // The comma belongs to the last element, not to the closing
            // delimiter. Find that element as a direct child so a block lambda
            // ending with an implicit close cannot move the comma into its
            // body. A literal close token must exist; without one there is no
            // safe place for a trailing comma.
            let elements: Vec<_> = node.children_with_tokens().collect();
            let Some(close) = elements.iter().rposition(|element| {
                element
                    .as_token()
                    .is_some_and(|token| matches!(token.text(), ")" | "]" | "}"))
            }) else {
                continue;
            };
            let Some(previous) = elements[..close]
                .iter()
                .rev()
                .find(|element| !element.kind().is_trivia() && !element.kind().is_layout())
            else {
                continue;
            };
            let Some(previous) = previous.as_node() else {
                continue;
            };
            let end = u32::from(previous.text_range().end());
            let last = previous
                .descendants_with_tokens()
                .filter_map(|element| element.into_token())
                .filter(|token| !token.kind().is_trivia() && !token.kind().is_layout())
                .last();
            if last.is_some_and(|token| u32::from(token.text_range().end()) == end) {
                comma_after.insert(end);
            }
        }
    }
    // A comment directly above a declaration belongs to that declaration. The
    // blank line goes before the comment, not between the comment and the item.
    let all_tokens: Vec<_> = root
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .collect();
    let mut comment_before_declaration = BTreeSet::new();
    for (index, token) in all_tokens.iter().enumerate() {
        if !matches!(token.kind(), Kind::Comment | Kind::DocComment) {
            continue;
        }
        if let Some(next) = all_tokens[index + 1..]
            .iter()
            .find(|token| !token.kind().is_trivia() && !token.kind().is_layout())
            && declaration_starts.contains(&u32::from(next.text_range().start()))
        {
            comment_before_declaration.insert(u32::from(token.text_range().start()));
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
    let mut last_was_comment = false;
    let flush = |out: &mut String, line: &mut String| {
        if !line.trim().is_empty() {
            out.push_str(line.trim_end());
            out.push('\n');
        }
        line.clear();
    };
    for token in ordered_tokens(root) {
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
        let line_comment = at_line_start && matches!(kind, Kind::Comment | Kind::DocComment);
        if at_line_start {
            let declaration = declaration_starts.contains(&u32::from(token.text_range().start()))
                || (line_comment
                    && comment_before_declaration.contains(&u32::from(token.text_range().start())));
            let attached_to_previous_comment = declaration && last_was_comment && !pending_blank;
            if (pending_blank || declaration)
                && !attached_to_previous_comment
                && !out.is_empty()
                && !out.ends_with("\n\n")
            {
                out.push('\n');
            }
            pending_blank = false;
            let width = if kind == Kind::Comment {
                // A comment is layout trivia, not part of the block it happens
                // to be stored in. Keep the column the author chose so a
                // comment after a block does not get absorbed into it.
                physical_indent
            } else if continuation && !matches!(text, ")" | "]" | "}") {
                (indent + 1) * 4
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
        let parent = token.parent().map(|node| node.kind());
        // A call's or declaration's parentheses belong to their name. Every
        // other parenthesis is a tuple/group and follows normal token spacing.
        let tight_paren = text == "("
            && (matches!(
                parent,
                Some(Kind::Arguments | Kind::Parameters | Kind::Variant)
            ) || previous == "fn"
                || (parent == Some(Kind::Pattern)
                    && matches!(last_kind, Some(Kind::Ident | Kind::TypeIdent))));
        // Brackets are tight only when they index or slice a receiver.
        let tight_bracket = text == "[" && parent == Some(Kind::Index);
        let range = matches!(text, ".." | "..=");
        let prefix_range = range && matches!(previous.as_str(), "" | "," | "[" | "{" | "(");
        let no_before = matches!(text, "," | ":" | ")" | "]" | "}" | "." | "?")
            || (generic && matches!(text, "<" | ">"))
            || (range && !prefix_range);
        let no_after = matches!(previous.as_str(), "(" | "[" | "{" | "." | ".." | "..=")
            || (generic && previous == "<");
        let set = text == "{" && previous == "set";
        let comment = kind == Kind::Comment;
        if !previous.is_empty()
            && !no_before
            && !no_after
            && !tight_paren
            && !tight_bracket
            && !set
            && !previous_tight
            && !line.ends_with(' ')
        {
            line.push_str(if comment { "  " } else { " " });
        }
        line.push_str(text);
        if comma_after.contains(&u32::from(token.text_range().end())) {
            line.push(',');
        }
        previous_tight = text == "-" && token.parent().is_some_and(|n| n.kind() == Kind::Unary);
        previous = text.into();
        last_kind = Some(kind);
        last_was_comment = line_comment;
    }
    flush(&mut out, &mut line);
    // A module doc comment needs its separating blank line to stay a module
    // doc comment when no declaration follows it.
    if pending_blank && last_kind == Some(Kind::DocComment) && out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// The token order to format. Consecutive top-level imports are sorted as whole
/// CST nodes, so comments and multiline literals can never be mistaken for
/// import lines.
fn ordered_tokens(root: &SyntaxNode) -> Vec<SyntaxToken> {
    fn flush(out: &mut Vec<SyntaxToken>, imports: &mut Vec<SyntaxNode>) {
        // The blank line that starts a group belongs before the sorted group,
        // not to whichever import happened to be first in source order.
        if let Some(blank) = imports.first().and_then(|node| {
            node.children_with_tokens()
                .next()
                .and_then(|element| element.into_token())
                .filter(|token| token.kind() == Kind::Blank)
        }) {
            out.push(blank);
        }
        imports.sort_by_cached_key(|node| format_syntax(node).trim().to_owned());
        for node in imports.drain(..) {
            let mut tokens = node
                .descendants_with_tokens()
                .filter_map(|element| element.into_token())
                .peekable();
            if tokens
                .peek()
                .is_some_and(|token| token.kind() == Kind::Blank)
            {
                tokens.next();
            }
            out.extend(tokens);
        }
    }
    let mut out = Vec::new();
    let mut imports = Vec::new();
    for element in root.children_with_tokens() {
        if let Some(node) = element.clone().into_node() {
            if node.kind() == Kind::Import {
                // A blank line before an import starts a new import group. Go
                // formatting sorts inside a group and keeps the author's
                // separation between groups.
                if !imports.is_empty()
                    && node
                        .children_with_tokens()
                        .next()
                        .is_some_and(|element| element.kind() == Kind::Blank)
                {
                    flush(&mut out, &mut imports);
                }
                imports.push(node);
            } else {
                flush(&mut out, &mut imports);
                out.extend(
                    node.descendants_with_tokens()
                        .filter_map(|element| element.into_token()),
                );
            }
        } else if let Some(token) = element.into_token() {
            flush(&mut out, &mut imports);
            out.push(token);
        }
    }
    flush(&mut out, &mut imports);
    out
}
