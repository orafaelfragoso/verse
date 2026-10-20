use crate::{Diagnostic, FileId, Kind, SyntaxNode, Token, lex, tree::raw};
use rowan::{GreenNode, GreenToken, NodeOrToken};
use std::collections::HashSet;
use std::ops::Deref;

/// A checkpoint for wrapping already-emitted siblings.
#[derive(Clone, Copy)]
struct Checkpoint(usize);

/// A green-tree builder that never hash-conses.
///
/// Rowan's `GreenNodeBuilder` hashes each subtree to share identical nodes.
/// That hash walks the subtree recursively, so a deeply nested but valid
/// expression overflows the stack inside `finish_node`. Building `GreenNode`
/// values directly keeps parsing linear and independent of nesting depth.
struct GreenBuilder {
    parents: Vec<(Kind, usize)>,
    children: Vec<NodeOrToken<GreenNode, GreenToken>>,
}

impl GreenBuilder {
    fn new() -> Self {
        Self {
            parents: Vec::new(),
            children: Vec::new(),
        }
    }
    fn token(&mut self, kind: Kind, text: &str) {
        self.children
            .push(NodeOrToken::Token(GreenToken::new(raw(kind), text)));
    }
    fn start_node(&mut self, kind: Kind) {
        let first_child = self.children.len();
        self.parents.push((kind, first_child));
    }
    fn finish_node(&mut self) {
        let (kind, first_child) = self.parents.pop().expect("unbalanced green tree");
        let children: Vec<_> = self.children.drain(first_child..).collect();
        self.children
            .push(NodeOrToken::Node(GreenNode::new(raw(kind), children)));
    }
    fn checkpoint(&self) -> Checkpoint {
        Checkpoint(self.children.len())
    }
    fn start_node_at(&mut self, checkpoint: Checkpoint, kind: Kind) {
        assert!(
            checkpoint.0 <= self.children.len(),
            "checkpoint no longer valid"
        );
        if let Some(&(_, first_child)) = self.parents.last() {
            assert!(
                checkpoint.0 >= first_child,
                "checkpoint outside the current node"
            );
        }
        self.parents.push((kind, checkpoint.0));
    }
    fn finish(mut self) -> GreenNode {
        assert!(
            self.parents.is_empty(),
            "unclosed green tree nodes: {}",
            self.parents.len()
        );
        assert_eq!(
            self.children.len(),
            1,
            "the green tree must have exactly one root"
        );
        match self.children.pop().unwrap() {
            NodeOrToken::Node(node) => node,
            NodeOrToken::Token(_) => panic!("the green tree root must be a node"),
        }
    }
}

/// A green tree whose drop is iterative for pathological nesting. Rowan drops
/// nested `GreenNode`s recursively; walking the tree first keeps shallow parses
/// allocation-free while deep valid input cannot overflow the caller's stack.
#[derive(Debug, Clone)]
pub struct GreenTree(GreenNode);

impl GreenTree {
    pub fn clone_green(&self) -> GreenNode {
        self.0.clone()
    }
}

impl Drop for GreenTree {
    fn drop(&mut self) {
        fn depth(root: &GreenNode) -> usize {
            let mut maximum = 1;
            let mut stack = vec![(&**root, 1usize)];
            while let Some((node, current)) = stack.pop() {
                maximum = maximum.max(current);
                for child in node.children() {
                    if let NodeOrToken::Node(child) = child {
                        stack.push((child, current + 1));
                    }
                }
            }
            maximum
        }
        if depth(&self.0) <= 1_024 {
            return;
        }
        let green = std::mem::replace(
            &mut self.0,
            GreenNode::new(
                raw(Kind::File),
                std::iter::empty::<NodeOrToken<GreenNode, GreenToken>>(),
            ),
        );
        stacker::grow(128 * 1024 * 1024, move || drop(green));
    }
}

#[derive(Debug, Clone)]
pub struct Parse {
    pub green: GreenTree,
    pub diagnostics: Vec<Diagnostic>,
    pub interpolations: Vec<ParsedInterpolation>,
}

/// Embedded expressions keep their own green trees and absolute source ranges,
/// while the enclosing text token retains its exact original spelling.
#[derive(Debug, Clone)]
pub struct ParsedInterpolation {
    pub expression: crate::Span,
    pub green: GreenTree,
}

/// A guarded view of a parse's concrete syntax tree. Holding this keeps the
/// iterative drop of [`GreenTree`] alive for as long as any `SyntaxNode` cloned
/// from it, so dropping a deep valid tree cannot overflow the caller's stack.
#[derive(Clone)]
pub struct Cst {
    node: SyntaxNode,
    /// Held only so the iterative drop of the green tree outlives `node`.
    #[allow(dead_code)]
    tree: GreenTree,
}

impl Cst {
    fn new(green: GreenTree) -> Self {
        let node = SyntaxNode::new_root(green.clone_green());
        Self { node, tree: green }
    }
}

impl Deref for Cst {
    type Target = SyntaxNode;
    fn deref(&self) -> &SyntaxNode {
        &self.node
    }
}

impl std::fmt::Debug for Cst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.node.fmt(f)
    }
}

impl Parse {
    pub fn syntax(&self) -> Cst {
        Cst::new(self.green.clone())
    }
    pub fn is_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub fn parse(source: &str, file: FileId) -> Parse {
    let lexed = lex(source, file);
    let pairs = delimiter_pairs(source, &lexed.tokens);
    let range_inside = top_level_ranges(source, &lexed.tokens);
    let mut p = Parser {
        source,
        tokens: &lexed.tokens,
        pos: 0,
        builder: GreenBuilder::new(),
        diagnostics: lexed.diagnostics,
        pairs: &pairs,
        range_inside: &range_inside,
        imports_done: false,
        implicit_closed: false,
    };
    p.start(Kind::File);
    p.module_doc();
    while !p.at_kind(Kind::Eof) {
        let before = p.pos;
        if p.at_kind(Kind::Newline) {
            p.bump();
        } else {
            p.statement(Context::Top);
        }
        p.ensure_progress(before);
    }
    p.bump();
    p.finish();
    let mut interpolations = Vec::new();
    // Text is a lossless token; interpolation expressions are parsed independently
    // and their diagnostics are mapped back into the enclosing source file.
    for interpolation in lexed.interpolations {
        let mut start = interpolation.expression.start as usize;
        let end = interpolation.expression.end as usize;
        start += source[start..end].len() - source[start..end].trim_start().len();
        let text = source[start..end].trim_end();
        let nested = lex(text, file);
        let pairs = delimiter_pairs(text, &nested.tokens);
        let range_inside = top_level_ranges(text, &nested.tokens);
        let mut q = Parser {
            source: text,
            tokens: &nested.tokens,
            pos: 0,
            builder: GreenBuilder::new(),
            diagnostics: nested.diagnostics,
            pairs: &pairs,
            range_inside: &range_inside,
            imports_done: false,
            implicit_closed: false,
        };
        q.start(Kind::File);
        q.expr(0, true);
        q.eat_kind(Kind::Newline);
        if !q.at_kind(Kind::Eof) {
            q.expected("the end of the interpolation");
            q.start(Kind::Error);
            while !q.at_kind(Kind::Eof) {
                q.bump();
            }
            q.finish();
        }
        q.bump();
        q.finish();
        for mut diagnostic in q.diagnostics {
            diagnostic.primary.span.start += start as u32;
            diagnostic.primary.span.end += start as u32;
            for label in &mut diagnostic.secondary {
                label.span.start += start as u32;
                label.span.end += start as u32;
            }
            for help in &mut diagnostic.help {
                for edit in &mut help.edits {
                    edit.span.start += start as u32;
                    edit.span.end += start as u32;
                }
            }
            p.diagnostics.push(diagnostic);
        }
        interpolations.push(ParsedInterpolation {
            expression: crate::Span::new(file, start, start + text.len()),
            green: GreenTree(q.builder.finish()),
        });
    }
    // Re-lexing a text interpolation can report the same malformed token as
    // the enclosing module. A diagnostic is identified by its code and span;
    // keep the first copy so one source mistake is one error.
    let mut seen = HashSet::new();
    p.diagnostics
        .retain(|diagnostic| seen.insert((diagnostic.code.clone(), diagnostic.primary.span)));
    Parse {
        green: GreenTree(p.builder.finish()),
        diagnostics: p.diagnostics,
        interpolations,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Context {
    Top,
    Block,
    Trait,
    Impl,
    Extern,
    Fields,
}

struct Parser<'a> {
    source: &'a str,
    tokens: &'a [Token],
    pos: usize,
    builder: GreenBuilder,
    diagnostics: Vec<Diagnostic>,
    pairs: &'a [Option<usize>],
    range_inside: &'a [bool],
    imports_done: bool,
    implicit_closed: bool,
}

fn delimiter_pairs(source: &str, tokens: &[Token]) -> Vec<Option<usize>> {
    let mut pairs = vec![None; tokens.len()];
    let mut stack: Vec<(usize, &str)> = vec![];
    for (index, token) in tokens.iter().enumerate() {
        let text = token.text(source);
        if matches!(text, "(" | "[" | "{" | "<") {
            stack.push((index, text));
        } else if matches!(text, ")" | "]" | "}" | ">") || token.kind == Kind::ImplicitClose {
            let expected = if token.kind == Kind::ImplicitClose {
                "("
            } else {
                match text {
                    ")" => "(",
                    "]" => "[",
                    "}" => "{",
                    _ => "<",
                }
            };
            if stack.last().is_some_and(|(_, open)| *open == expected) {
                let (open, _) = stack.pop().unwrap();
                pairs[open] = Some(index);
            }
        }
    }
    pairs
}

/// For every `[` token, whether a top-level `..` or `..=` appears directly
/// inside it (before its matching `]`). A slice's bounds are `additive`
/// expressions, while an index's expression is any expression, so the parser
/// must choose the precedence before it sees the range.
fn top_level_ranges(source: &str, tokens: &[Token]) -> Vec<bool> {
    let mut out = vec![false; tokens.len()];
    let mut brackets: Vec<usize> = vec![];
    for (index, token) in tokens.iter().enumerate() {
        match token.text(source) {
            "(" | "[" | "{" => brackets.push(index),
            ")" | "]" | "}" => {
                brackets.pop();
            }
            ".." | "..="
                if brackets
                    .last()
                    .is_some_and(|&open| tokens[open].text(source) == "[") =>
            {
                out[*brackets.last().unwrap()] = true;
            }
            _ => {}
        }
    }
    out
}

impl Parser<'_> {
    fn module_doc(&mut self) {
        let start = self.index();
        if self.tokens[start].kind != Kind::DocComment {
            return;
        }
        let mut i = start;
        while i < self.tokens.len() {
            match self.tokens[i].kind {
                Kind::DocComment | Kind::Newline | Kind::Whitespace => i += 1,
                Kind::Blank => {
                    // Consume the separator as raw trivia, without swallowing
                    // the following declaration through significant lookahead.
                    while self.pos <= i {
                        let token = self.tokens[self.pos];
                        self.builder.token(token.kind, token.text(self.source));
                        self.pos += 1;
                    }
                    return;
                }
                _ => return,
            }
        }
    }
    fn index(&self) -> usize {
        let mut i = self.pos;
        while i < self.tokens.len() && self.tokens[i].kind.is_trivia() {
            i += 1;
        }
        i.min(self.tokens.len() - 1)
    }
    fn token(&self) -> Token {
        self.tokens[self.index()]
    }
    fn kind(&self) -> Kind {
        self.token().kind
    }
    fn text(&self) -> &str {
        self.token().text(self.source)
    }
    fn at(&self, text: &str) -> bool {
        self.text() == text && self.kind() != Kind::Eof
    }
    fn at_kind(&self, kind: Kind) -> bool {
        self.kind() == kind
    }
    fn nth(&self, n: usize) -> Token {
        self.tokens[self.index()..]
            .iter()
            .filter(|t| !t.kind.is_trivia())
            .nth(n)
            .copied()
            .unwrap_or(*self.tokens.last().unwrap())
    }
    fn nth_text(&self, n: usize) -> &str {
        self.nth(n).text(self.source)
    }
    fn start(&mut self, kind: Kind) {
        self.builder.start_node(kind);
    }
    fn finish(&mut self) {
        self.builder.finish_node();
    }
    fn checkpoint(&self) -> Checkpoint {
        self.builder.checkpoint()
    }
    fn wrap(&mut self, checkpoint: Checkpoint, kind: Kind) {
        self.builder.start_node_at(checkpoint, kind);
    }
    fn bump(&mut self) {
        let index = self.index();
        for token in &self.tokens[self.pos..=index] {
            self.builder.token(token.kind, token.text(self.source));
        }
        self.pos = index + 1;
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn eat_kind(&mut self, kind: Kind) -> bool {
        if self.at_kind(kind) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, text: &str) {
        if !self.eat(text) {
            self.expected(text);
        }
    }
    fn expected(&mut self, what: &str) {
        let token = self.token();
        let found = if token.kind == Kind::Eof {
            "end of file"
        } else if token.kind == Kind::Newline {
            "end of line"
        } else {
            token.text(self.source)
        };
        self.diagnostics.push(Diagnostic::new(
            "V0201",
            token.span,
            [("what", what.into()), ("found", found.into())],
        ));
    }
    fn ensure_progress(&mut self, before: usize) {
        if self.pos == before && !self.at_kind(Kind::Eof) {
            self.start(Kind::Error);
            self.expected("a declaration or statement");
            self.bump();
            self.finish();
        }
    }
    fn recover_line(&mut self) {
        self.start(Kind::Error);
        while !matches!(self.kind(), Kind::Newline | Kind::Dedent | Kind::Eof) {
            self.bump();
        }
        self.eat_kind(Kind::Newline);
        self.finish();
    }
    fn line_end(&mut self) {
        if self.eat_kind(Kind::Newline) || self.at_kind(Kind::Eof) || self.at_kind(Kind::Dedent) {
            return;
        }
        let previous = self.tokens[..self.pos]
            .iter()
            .rev()
            .find(|t| !t.kind.is_trivia());
        if previous.is_some_and(|t| matches!(t.kind, Kind::Dedent | Kind::ImplicitClose)) {
            return;
        }
        self.expected("end of line");
        self.recover_line();
    }
    fn name(&mut self, type_name: bool) {
        if self.kind()
            == if type_name {
                Kind::TypeIdent
            } else {
                Kind::Ident
            }
            || (type_name && self.at("Self"))
        {
            self.bump();
        } else if self.kind() == Kind::Keyword {
            self.diagnostics.push(Diagnostic::new(
                "V0203",
                self.token().span,
                [("kw", self.text().into())],
            ));
            self.bump();
        } else {
            self.expected(if type_name { "a type name" } else { "a name" });
        }
    }
    fn any_name(&mut self) {
        if matches!(self.kind(), Kind::Ident | Kind::TypeIdent) {
            self.bump();
        } else {
            self.name(false);
        }
    }

    fn statement(&mut self, context: Context) {
        self.implicit_closed = false;
        let cp = self.checkpoint();
        while self.eat_kind(Kind::DocComment) {
            self.eat_kind(Kind::Newline);
        }
        let exported = self.eat("export");
        // `export` precedes only a declaration or a new binding, never an
        // import, an impl or extern block, a test, a statement, or a
        // reassignment (grammar §17 `declaration`).
        if exported
            && context == Context::Top
            && !matches!(self.text(), "fn" | "type" | "alias" | "trait")
            && !(self.binding_ahead() && !self.place_assignment_ahead())
        {
            self.wrap(cp, Kind::Error);
            self.expected("a declaration after `export`");
            self.recover_line();
            self.finish();
            return;
        }
        let restricted = match context {
            Context::Trait | Context::Impl | Context::Extern => !self.at("fn"),
            Context::Fields => self.kind() != Kind::Ident,
            Context::Block => {
                matches!(
                    self.text(),
                    "type" | "alias" | "trait" | "impl" | "extern" | "test"
                ) || exported
            }
            Context::Top => false,
        };
        if restricted {
            self.wrap(cp, Kind::Error);
            self.expected(match context {
                Context::Fields => "a field",
                Context::Trait | Context::Impl | Context::Extern => "a function",
                _ => "a statement",
            });
            self.recover_line();
            self.finish();
            return;
        }
        if context == Context::Top && !self.at("import") {
            self.imports_done = true;
        }
        match self.text() {
            "import" => self.import(cp, context),
            "from" => {
                self.wrap(cp, Kind::Error);
                let span = self.token().span;
                let module = self.nth_text(1).to_owned();
                let names = self.tokens[self.index()..]
                    .iter()
                    .position(|t| t.text(self.source) == "import")
                    .map(|i| {
                        self.tokens[self.index() + i + 1..]
                            .iter()
                            .take_while(|t| t.kind != Kind::Newline)
                            .map(|t| t.text(self.source))
                            .collect::<String>()
                            .trim()
                            .to_owned()
                    })
                    .unwrap_or_default();
                self.diagnostics.push(Diagnostic::new(
                    "V0206",
                    span,
                    [("names", names.into()), ("module", module.into())],
                ));
                self.recover_line();
                self.finish();
            }
            "fn" => self.function(cp, context),
            "type" => {
                self.wrap(cp, Kind::TypeDeclaration);
                self.bump();
                self.name(true);
                self.generics();
                if self.eat("=") {
                    loop {
                        self.start(Kind::Variant);
                        self.name(true);
                        if self.eat("(") {
                            self.fields_in_parens();
                        }
                        self.finish();
                        if !self.eat("|") {
                            break;
                        }
                    }
                    self.line_end();
                } else {
                    self.block(Context::Fields);
                }
                self.finish();
            }
            "alias" => {
                self.wrap(cp, Kind::Alias);
                self.bump();
                self.name(true);
                self.generics();
                self.expect("=");
                self.ty();
                self.line_end();
                self.finish();
            }
            "trait" => {
                self.wrap(cp, Kind::Trait);
                self.bump();
                self.name(true);
                self.generics();
                if self.eat(":") {
                    self.bounds();
                }
                self.block(Context::Trait);
                self.finish();
            }
            "impl" => {
                self.wrap(cp, Kind::Impl);
                self.bump();
                self.generics();
                self.ty();
                if self.eat("for") {
                    self.ty();
                }
                if self.at_kind(Kind::Newline) && self.nth(1).kind != Kind::Indent {
                    self.bump();
                } else {
                    self.block(Context::Impl);
                }
                self.finish();
            }
            "extern" => {
                self.wrap(cp, Kind::Extern);
                self.bump();
                self.name(false);
                self.block(Context::Extern);
                self.finish();
            }
            "test" => {
                self.wrap(cp, Kind::Test);
                self.bump();
                if !self.eat_kind(Kind::Text) {
                    self.expected("a test name in quotes");
                }
                self.block(Context::Block);
                self.finish();
            }
            _ if context == Context::Fields => {
                self.wrap(cp, Kind::Field);
                self.name(false);
                if self.eat(":") {
                    self.ty();
                    if self.eat("=") {
                        self.expr(0, true);
                    }
                } else {
                    self.expect("=");
                    self.expr(0, true);
                }
                self.line_end();
                self.finish();
            }
            "if" => {
                self.if_block(cp, false);
            }
            "for" | "while" => self.loop_statement(cp),
            _ if self.kind() == Kind::Ident
                && self.nth_text(1) == ":"
                && matches!(self.nth_text(2), "for" | "while") =>
            {
                self.bump();
                self.bump();
                self.loop_statement(cp);
            }
            "return" | "break" | "continue" => {
                self.exit(cp);
                if self.eat("if") {
                    self.expr(0, true);
                }
                self.line_end();
            }
            "pass" => {
                self.wrap(cp, Kind::Pass);
                self.bump();
                self.line_end();
                self.finish();
            }
            "assert" => {
                self.wrap(cp, Kind::Assert);
                self.bump();
                self.expr(0, true);
                if self.eat(",") {
                    self.expr(0, true);
                }
                self.line_end();
                self.finish();
            }
            "match" => {
                self.match_expr(cp);
            }
            _ => {
                if self.binding_ahead() {
                    let place = self.place_assignment_ahead();
                    self.wrap(
                        cp,
                        if place {
                            Kind::Assignment
                        } else {
                            Kind::Binding
                        },
                    );
                    self.eat("mut");
                    if place {
                        self.place();
                    } else {
                        self.pattern(false);
                        if self.eat(":") {
                            self.ty();
                        }
                    }
                    if matches!(self.text(), "=" | "+=" | "-=" | "*=" | "/=" | "%=") {
                        self.bump();
                    } else {
                        self.expected("=");
                    }
                    self.expr(0, true);
                    self.line_end();
                    self.finish();
                } else {
                    self.wrap(cp, Kind::ExpressionStatement);
                    self.expr(0, true);
                    self.line_end();
                    self.finish();
                }
            }
        }
    }

    fn binding_ahead(&self) -> bool {
        let mut nesting = 0usize;
        for token in &self.tokens[self.index()..] {
            if matches!(token.kind, Kind::Newline | Kind::Eof | Kind::Dedent) {
                break;
            }
            match token.text(self.source) {
                "(" | "[" | "{" => nesting += 1,
                ")" | "]" | "}" => nesting = nesting.saturating_sub(1),
                "=" | "+=" | "-=" | "*=" | "/=" | "%=" if nesting == 0 => return true,
                _ => {}
            }
        }
        false
    }
    fn place_assignment_ahead(&self) -> bool {
        self.at("self")
            || matches!(self.nth_text(1), "." | "[")
            || self.tokens[self.index()..]
                .iter()
                .take_while(|t| t.kind != Kind::Newline)
                .any(|t| matches!(t.text(self.source), "+=" | "-=" | "*=" | "/=" | "%="))
    }
    fn place(&mut self) {
        // Grammar §17 `place = ( IDENT | "self" ) { "." IDENT | "[" expr "]" }`.
        // Only a name, self, field access and indexing may be assigned; a call
        // or a numeric field is not a place.
        if self.eat("self") {
        } else {
            self.name(false);
        }
        loop {
            if self.eat(".") {
                if matches!(self.kind(), Kind::Ident) {
                    self.bump();
                } else {
                    self.expected("a field name");
                    break;
                }
            } else if self.eat("[") {
                self.expr(0, true);
                self.expect("]");
            } else {
                break;
            }
        }
    }
    fn block(&mut self, context: Context) {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.block_inner(context));
    }
    fn block_inner(&mut self, context: Context) {
        self.start(Kind::Block);
        if !self.eat_kind(Kind::Newline) {
            self.expected("an indented block");
            self.finish();
            return;
        }
        while self.eat_kind(Kind::Newline) {}
        if !self.eat_kind(Kind::Indent) {
            self.diagnostics
                .push(Diagnostic::simple("V0202", self.token().span));
            self.finish();
            return;
        }
        let mut count = 0;
        while !matches!(self.kind(), Kind::Dedent | Kind::Eof) {
            let before = self.pos;
            if !self.eat_kind(Kind::Newline) {
                self.statement(context);
                count += 1;
            }
            self.ensure_progress(before);
        }
        if count == 0 {
            self.diagnostics
                .push(Diagnostic::simple("V0202", self.token().span));
        }
        self.eat_kind(Kind::Dedent);
        self.finish();
    }

    fn import(&mut self, cp: Checkpoint, context: Context) {
        self.wrap(cp, Kind::Import);
        if self.imports_done || context != Context::Top {
            self.diagnostics
                .push(Diagnostic::simple("V0207", self.token().span));
        }
        self.bump();
        let parenthesized = self.eat("(");
        let first = self.token().span;
        self.any_name();
        while self.eat(".") {
            self.name(false);
        }
        if self.eat("as") {
            self.any_name();
        }
        let mut multiple = false;
        while self.eat(",") {
            multiple = true;
            if self.at(")") {
                break;
            }
            if self.at_kind(Kind::Newline) && !parenthesized {
                self.diagnostics.push(Diagnostic::simple("V0208", first));
                break;
            }
            self.any_name();
            if self.eat("as") {
                self.any_name();
            }
        }
        if parenthesized {
            self.expect(")");
        }
        if self.eat("from") {
            self.name(false);
            while self.eat(".") {
                self.name(false);
            }
        } else if parenthesized || multiple {
            self.expected("from");
        }
        self.line_end();
        self.finish();
    }

    fn function(&mut self, cp: Checkpoint, context: Context) {
        self.wrap(cp, Kind::Function);
        self.bump();
        if context == Context::Extern {
            self.any_name();
            if self.eat("as") {
                self.name(false);
            }
        } else {
            self.name(false);
        }
        self.generics();
        self.start(Kind::Parameters);
        self.expect("(");
        while !self.at(")") && !matches!(self.kind(), Kind::Newline | Kind::Eof | Kind::Dedent) {
            let before = self.pos;
            self.start(Kind::Parameter);
            if !self.eat("mut") {
                self.eat("take");
            }
            if !self.eat("self") {
                self.name(false);
            }
            if self.eat(":") {
                self.ty();
            }
            if self.eat("=") {
                self.expr(0, true);
            }
            self.finish();
            if !self.eat(",") {
                break;
            }
            self.ensure_progress(before);
        }
        self.expect(")");
        self.finish();
        if self.eat(":") {
            self.ty();
        }
        if context == Context::Extern
            || (context == Context::Trait
                && self.at_kind(Kind::Newline)
                && self.nth(1).kind != Kind::Indent)
        {
            self.line_end();
        } else {
            self.block(Context::Block);
        }
        self.finish();
    }
    fn generics(&mut self) {
        if !self.at("<") {
            return;
        }
        self.start(Kind::Generics);
        self.bump();
        loop {
            let before = self.pos;
            self.start(Kind::GenericParameter);
            self.name(true);
            if self.eat(":") {
                self.bounds();
            }
            self.finish();
            if !self.eat(",") || self.at(">") {
                break;
            }
            if before == self.pos {
                break;
            }
        }
        self.expect(">");
        self.finish();
    }
    fn bounds(&mut self) {
        self.type_atom();
        while self.eat("+") {
            self.type_atom();
        }
    }
    fn ty(&mut self) {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.ty_inner());
    }
    fn ty_inner(&mut self) {
        self.start(Kind::Type);
        self.type_atom();
        while self.eat("?") {}
        while self.eat("|") {
            self.type_atom();
            while self.eat("?") {}
        }
        if self.eat("raises") {
            self.type_atom();
            while self.eat("|") {
                self.type_atom();
            }
        }
        self.finish();
    }
    fn type_atom(&mut self) {
        if self.eat("fn") {
            self.expect("(");
            while !self.at(")") && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
                if !self.eat("mut") {
                    self.eat("take");
                }
                if self.kind() == Kind::Ident && self.nth_text(1) == ":" {
                    self.bump();
                    self.bump();
                }
                self.ty();
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")");
            self.expect(":");
            self.ty();
        } else if self.eat("(") {
            loop {
                if self.kind() == Kind::Ident && self.nth_text(1) == ":" {
                    self.bump();
                    self.bump();
                }
                self.ty();
                if !self.eat(",") || self.at(")") {
                    break;
                }
            }
            self.expect(")");
        } else {
            while self.kind() == Kind::Ident && self.nth_text(1) == "." {
                self.bump();
                self.bump();
            }
            self.name(true);
            if self.eat("<") {
                self.ty();
                while self.eat(",") {
                    if self.at(">") {
                        break;
                    }
                    self.ty();
                }
                self.expect(">");
            }
        }
    }
    fn fields_in_parens(&mut self) {
        while !self.at(")") && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
            self.start(Kind::Field);
            self.name(false);
            self.expect(":");
            self.ty();
            self.finish();
            if !self.eat(",") {
                break;
            }
        }
        self.expect(")");
    }

    fn if_block(&mut self, cp: Checkpoint, value: bool) {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.if_block_inner(cp, value));
    }
    fn if_block_inner(&mut self, cp: Checkpoint, value: bool) {
        self.expect("if");
        self.expr(0, true);
        if self.eat("then") {
            self.wrap(cp, Kind::Conditional);
            self.expr(0, true);
            self.expect("else");
            self.expr(0, true);
            self.line_end();
            self.finish();
            return;
        }
        self.wrap(cp, Kind::If);
        self.block(Context::Block);
        if self.eat("else") {
            if self.at("if") {
                let nested = self.checkpoint();
                self.if_block(nested, value);
            } else {
                self.block(Context::Block);
            }
        } else if value {
            self.expected("else");
        }
        self.finish();
    }
    fn loop_statement(&mut self, cp: Checkpoint) {
        let for_loop = self.at("for");
        self.wrap(cp, if for_loop { Kind::For } else { Kind::While });
        self.bump();
        if for_loop {
            self.pattern(false);
            self.expect("in");
        }
        self.expr(0, true);
        self.block(Context::Block);
        self.finish();
    }
    fn exit(&mut self, cp: Checkpoint) {
        let kind = match self.text() {
            "return" => Kind::Return,
            "break" => Kind::Break,
            _ => Kind::Continue,
        };
        self.wrap(cp, kind);
        self.bump();
        if kind == Kind::Return {
            if !matches!(self.kind(), Kind::Newline | Kind::Dedent | Kind::Eof)
                && !self.at("if")
                && !self.at(")")
            {
                self.expr(0, true);
            }
        } else if self.kind() == Kind::Ident {
            self.bump();
        }
        self.finish();
    }
    fn match_expr(&mut self, cp: Checkpoint) {
        self.wrap(cp, Kind::Match);
        self.bump();
        let subjectless = self.at_kind(Kind::Newline);
        if !subjectless {
            self.expr(0, true);
        }
        if !self.eat_kind(Kind::Newline) || !self.eat_kind(Kind::Indent) {
            self.expected("match arms");
            self.finish();
            return;
        }
        let mut count = 0;
        let mut final_wildcard = false;
        while !matches!(self.kind(), Kind::Dedent | Kind::Eof) {
            let before = self.pos;
            self.start(Kind::Arm);
            final_wildcard = self.at("_");
            if subjectless {
                self.expr(0, false);
            } else {
                self.pattern(true);
                if self.eat("if") {
                    self.expr(0, false);
                }
            }
            self.expect("->");
            if self.at_kind(Kind::Newline) {
                self.block(Context::Block);
            } else if self.at("pass") {
                self.start(Kind::Pass);
                self.bump();
                self.finish();
                self.line_end();
            } else if matches!(self.text(), "return" | "break" | "continue") {
                let exit = self.checkpoint();
                self.exit(exit);
                self.line_end();
            } else if self.binding_ahead() {
                self.start(Kind::Assignment);
                self.expr(12, false);
                if matches!(self.text(), "=" | "+=" | "-=" | "*=" | "/=" | "%=") {
                    self.bump();
                } else {
                    self.expected("an assignment");
                }
                self.expr(0, true);
                self.line_end();
                self.finish();
            } else {
                self.expr(0, false);
                self.line_end();
            }
            self.finish();
            count += 1;
            self.ensure_progress(before);
        }
        if count == 0 {
            self.diagnostics
                .push(Diagnostic::simple("V0202", self.token().span));
        }
        if subjectless && !final_wildcard {
            self.expected("a final `_ ->` arm");
        }
        self.eat_kind(Kind::Dedent);
        self.finish();
    }

    fn pattern(&mut self, typed: bool) {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || self.pattern_inner(typed));
    }
    fn pattern_inner(&mut self, typed: bool) {
        self.start(Kind::Pattern);
        loop {
            if self.kind().is_literal() || matches!(self.text(), "true" | "false" | "none") {
                self.bump();
                if self.eat("..") || self.eat("..=") {
                    if self.kind().is_literal() {
                        self.bump();
                    } else {
                        self.expected("a range endpoint");
                    }
                }
            } else if self.eat("(") {
                self.pattern(false);
                self.expect(",");
                self.pattern(false);
                while self.eat(",") && !self.at(")") {
                    self.pattern(false);
                }
                self.expect(")");
            } else if self.eat("[") {
                while !self.at("]") && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
                    if self.eat("..") {
                        self.name(false);
                        self.eat(",");
                        break;
                    } else {
                        self.pattern(false);
                    }
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect("]");
            } else if self.kind() == Kind::TypeIdent {
                self.bump();
                if self.eat("(") {
                    while !self.at(")") && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
                        let field = self.kind() == Kind::Ident && self.nth_text(1) == ":";
                        if field {
                            self.bump();
                            self.bump();
                        }
                        if !(field && matches!(self.text(), "," | ")")) {
                            self.pattern(field);
                        }
                        if !self.eat(",") {
                            break;
                        }
                    }
                    self.expect(")");
                }
            } else if self.kind() == Kind::Ident {
                self.bump();
                if typed && self.eat(":") {
                    self.ty();
                }
            } else {
                self.expected("a pattern");
                break;
            }
            while self.eat("?") {}
            if !self.eat("|") {
                break;
            }
        }
        self.finish();
    }

    fn lambda_ahead(&self) -> bool {
        if self.kind() == Kind::Ident {
            return self.nth_text(1) == "->";
        }
        if !self.at("(") {
            return false;
        }
        self.pairs[self.index()].is_some_and(|end| {
            self.tokens[end + 1..]
                .iter()
                .find(|t| !t.kind.is_trivia())
                .is_some_and(|t| t.text(self.source) == "->")
        })
    }
    fn expr(&mut self, min: u8, allow_lambda: bool) {
        stacker::maybe_grow(64 * 1024, 1024 * 1024, || {
            self.expr_inner(min, allow_lambda)
        });
    }
    fn expr_inner(&mut self, min: u8, allow_lambda: bool) {
        let cp = self.checkpoint();
        let expression_start = self.token().span.start as usize;
        let key_path = self.at(".");
        let mut associated_receiver = self.kind() == Kind::TypeIdent && self.nth_text(1) == ".";
        let mut range_seen = false;
        if min == 0 && allow_lambda && self.lambda_ahead() {
            self.wrap(cp, Kind::Lambda);
            if self.eat("(") {
                while !self.at(")") && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
                    self.start(Kind::Parameter);
                    self.name(false);
                    if self.eat(":") {
                        self.ty();
                    }
                    self.finish();
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(")");
            } else {
                self.name(false);
            }
            self.expect("->");
            if self.at_kind(Kind::Newline) {
                self.block(Context::Block);
            } else {
                self.expr(0, true);
            }
            self.finish();
        } else if min == 0 && self.at("if") {
            // The condition stops at then or at the block's logical newline.
            self.wrap(cp, Kind::Conditional);
            self.bump();
            self.expr(0, true);
            if self.eat("then") {
                self.expr(0, true);
                self.expect("else");
                self.expr(0, true);
            } else {
                self.block(Context::Block);
                self.expect("else");
                if self.at("if") {
                    self.expr(0, true);
                } else {
                    self.block(Context::Block);
                }
            }
            self.finish();
        } else if self.at("match") {
            self.match_expr(cp);
        } else if (self.at("-") && min <= 9)
            || (self.at("not") && min <= 4)
            || (self.at("try") && min <= 11)
            || (self.at("spawn") && min <= 9)
        {
            let op = self.text().to_owned();
            self.wrap(cp, Kind::Unary);
            self.bump();
            self.expr(
                match op.as_str() {
                    "not" => 4,
                    "-" => 9,
                    "try" => 11,
                    _ => 12,
                },
                allow_lambda,
            );
            self.finish();
        } else {
            self.primary();
        }

        loop {
            if self.implicit_closed {
                let continuation = matches!(
                    self.text(),
                    "(" | "["
                        | "."
                        | "otherwise"
                        | "or"
                        | "and"
                        | "=="
                        | "!="
                        | "<"
                        | "<="
                        | ">"
                        | ">="
                        | "in"
                        | "is"
                        | ".."
                        | "..="
                        | "+"
                        | "-"
                        | "*"
                        | "/"
                        | "%"
                        | "**"
                ) || (self.at("not") && self.nth_text(1) == "in");
                if continuation {
                    // A block lambda's dedent closes its call. Chaining or
                    // applying an operator to it on that dedent line would
                    // silently attach the text to the lambda body instead.
                    let span = self.token().span;
                    if !self
                        .diagnostics
                        .iter()
                        .any(|d| d.code == "V0201" && d.primary.span == span)
                    {
                        self.expected("end of line");
                    }
                    self.implicit_closed = false;
                    break;
                }
            }
            if min <= 12 && self.at("(") {
                associated_receiver = false;
                self.wrap(cp, Kind::Call);
                self.arguments();
                self.finish();
                continue;
            }
            if min <= 12 && self.eat_index(cp) {
                associated_receiver = false;
                continue;
            }
            if min <= 12 && self.at(".") {
                self.wrap(cp, Kind::FieldAccess);
                self.bump();
                if matches!(self.kind(), Kind::Ident | Kind::TypeIdent | Kind::Int) {
                    self.bump();
                } else {
                    self.expected("a field name");
                }
                self.finish();
                // Only an associated function with a capitalised receiver may
                // have explicit arguments. Lowercase `a.b < c > (d)` compares.
                if self.at("<") && associated_receiver && self.associated_type_arguments_ahead() {
                    self.start(Kind::Generics);
                    self.bump();
                    self.ty();
                    while self.eat(",") && !self.at(">") {
                        self.ty();
                    }
                    self.expect(">");
                    self.finish();
                    if !self.at("(") {
                        self.expected("a call after explicit type arguments");
                    }
                }
                associated_receiver = false;
                continue;
            }
            let op = self.text().to_owned();
            let bp = match op.as_str() {
                "otherwise" => 1,
                "or" => 2,
                "and" => 3,
                "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" | "is" => 5,
                "not" if self.nth_text(1) == "in" => 5,
                ".." | "..=" => 6,
                "+" | "-" => 7,
                "*" | "/" | "%" => 8,
                "**" => 10,
                _ => break,
            };
            if bp < min {
                break;
            }
            if bp == 6 {
                if range_seen {
                    self.expected("parentheses around a nested range");
                }
                range_seen = true;
            }
            if key_path {
                self.expected("a plain key-path access chain; write a lambda for an operator");
            }
            self.wrap(
                cp,
                if bp == 5 {
                    Kind::Comparison
                } else {
                    Kind::Binary
                },
            );
            let operator_start = self.token().span.start as usize;
            self.bump();
            if op == "not" {
                self.expect("in");
            }
            if op == "is" {
                self.pattern(true);
            } else if op == "otherwise" {
                let fallback_start = self.token().span;
                if self.at("(") && matches!(self.nth_text(1), "return" | "break" | "continue") {
                    let mut diagnostic = Diagnostic::new(
                        "V0209",
                        self.nth(1).span,
                        [
                            ("op", self.nth_text(1).into()),
                            ("left", "left".into()),
                            ("right", "right".into()),
                            ("rest", "rest".into()),
                        ],
                    );
                    diagnostic.primary.label = "an exit must be the whole right operand".into();
                    diagnostic.help.clear();
                    diagnostic.facts.retain(|key, _| key == "op");
                    self.diagnostics.push(diagnostic);
                    self.start(Kind::Error);
                    while !matches!(self.kind(), Kind::Newline | Kind::Dedent | Kind::Eof) {
                        self.bump();
                    }
                    self.finish();
                } else if matches!(self.text(), "return" | "break" | "continue") {
                    let exit = self.checkpoint();
                    self.exit(exit);
                } else if self.at("-") {
                    self.start(Kind::Unary);
                    self.bump();
                    if matches!(self.kind(), Kind::Int | Kind::Float) {
                        self.start(Kind::Literal);
                        self.bump();
                        self.finish();
                    } else {
                        self.expected("a negative number or parenthesized fallback");
                    }
                    self.finish();
                } else {
                    self.expr(12, true);
                }
                if matches!(
                    self.text(),
                    "+" | "-"
                        | "*"
                        | "/"
                        | "%"
                        | "**"
                        | "and"
                        | "or"
                        | "=="
                        | "!="
                        | "<"
                        | ">"
                        | "<="
                        | ">="
                        | "in"
                        | "is"
                        | "not"
                        | ".."
                        | "..="
                ) {
                    let span = self.token().span;
                    let rest_end = self.tokens[self.index() + 1..]
                        .iter()
                        .take_while(|t| {
                            !matches!(t.kind, Kind::Newline | Kind::Comment | Kind::Eof)
                        })
                        .last()
                        .map_or(span.end, |t| t.span.end);
                    self.diagnostics.push(Diagnostic::new(
                        "V0209",
                        span,
                        [
                            ("op", self.text().into()),
                            (
                                "left",
                                self.source[expression_start..operator_start].trim().into(),
                            ),
                            (
                                "right",
                                self.source[fallback_start.start as usize..span.start as usize]
                                    .trim()
                                    .into(),
                            ),
                            (
                                "rest",
                                self.source[span.end as usize..rest_end as usize]
                                    .trim()
                                    .into(),
                            ),
                        ],
                    ));
                }
            } else {
                self.expr(if op == "**" { 9 } else { bp + 1 }, allow_lambda);
            }
            self.finish();
        }
    }

    fn type_arguments_ahead(&self) -> bool {
        let Some(close) = self.pairs[self.index()] else {
            return false;
        };
        // Type arguments sit on one logical line. A `<` whose `>` is on a later
        // line is a comparison, never the type argument list of a call here.
        !self.tokens[self.index()..close]
            .iter()
            .any(|token| token.kind == Kind::Newline)
    }

    /// Explicit type arguments on an associated call (`TYPE_IDENT . IDENT
    /// type_args call`) must end in a call, so `<`/`>` that are not followed by
    /// `(` stay comparisons even on the same logical line.
    fn associated_type_arguments_ahead(&self) -> bool {
        let Some(close) = self.pairs[self.index()] else {
            return false;
        };
        if self.tokens[self.index()..close]
            .iter()
            .any(|token| token.kind == Kind::Newline)
        {
            return false;
        }
        self.tokens[close + 1..]
            .iter()
            .find(|token| !token.kind.is_trivia() && !token.kind.is_layout())
            .is_some_and(|token| token.text(self.source) == "(")
    }

    fn primary(&mut self) {
        let cp = self.checkpoint();
        if self.kind().is_literal() || matches!(self.text(), "true" | "false" | "none") {
            self.wrap(cp, Kind::Literal);
            self.bump();
            self.finish();
        } else if matches!(self.kind(), Kind::Ident | Kind::TypeIdent)
            || matches!(self.text(), "self" | "Self")
        {
            let type_name = self.kind() == Kind::TypeIdent;
            self.wrap(cp, Kind::Name);
            self.bump();
            if type_name && self.at("<") && self.type_arguments_ahead() {
                self.start(Kind::Generics);
                self.bump();
                self.ty();
                while self.eat(",") && !self.at(">") {
                    self.ty();
                }
                self.expect(">");
                self.finish();
            }
            self.finish();
        } else if self.eat("(") {
            // Keep grouping explicit: lifted chains and `(try a).b` rely on it.
            if self.at(")") {
                self.wrap(cp, Kind::Error);
                self.expected("a grouped expression or tuple element");
                self.bump();
                self.finish();
                return;
            }
            let named = self.kind() == Kind::Ident && self.nth_text(1) == ":";
            self.argument();
            if self.eat(",") {
                self.wrap(cp, Kind::Tuple);
                while !self.at(")") && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
                    self.argument();
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(")");
                self.finish();
            } else {
                self.wrap(cp, Kind::Group);
                if named {
                    self.expected("a comma after a labeled tuple element");
                }
                self.expect(")");
                self.finish();
            }
        } else if self.eat("[") {
            self.wrap(cp, Kind::List);
            self.elements("]");
            self.finish();
        } else if self.eat("set") {
            self.wrap(cp, Kind::Set);
            self.expect("{");
            self.elements("}");
            self.finish();
        } else if self.eat("{") {
            self.wrap(cp, Kind::Map);
            if self.at("}") {
                self.diagnostics
                    .push(Diagnostic::simple("V0210", self.token().span));
                self.bump();
            } else if self.eat(":") {
                self.expect("}");
            } else {
                while !self.at("}") && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
                    self.start(Kind::Entry);
                    if self.eat("..") {
                        self.expr(0, true);
                    } else {
                        self.expr(0, true);
                        self.expect(":");
                        self.expr(0, true);
                    }
                    self.finish();
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect("}");
            }
            self.finish();
        } else if self.eat(".") {
            self.wrap(cp, Kind::KeyPath);
            if matches!(self.kind(), Kind::Ident | Kind::Int) {
                self.bump();
            } else {
                self.expected("a key-path field");
            }
            while self.eat(".") {
                if matches!(self.kind(), Kind::Ident | Kind::Int) {
                    self.bump();
                } else {
                    self.expected("a key-path field");
                    break;
                }
            }
            self.finish();
        } else {
            self.start(Kind::Error);
            self.expected("an expression");
            if !matches!(
                self.kind(),
                Kind::Eof | Kind::Newline | Kind::Dedent | Kind::ImplicitClose
            ) && !matches!(self.text(), ")" | "]" | "}" | "," | "then" | "else" | "->")
            {
                self.bump();
            }
            self.finish();
        }
    }
    fn elements(&mut self, closing: &str) {
        while !self.at(closing) && !matches!(self.kind(), Kind::Newline | Kind::Eof) {
            if self.at("..") {
                self.start(Kind::Spread);
                self.bump();
                self.expr(0, true);
                self.finish();
            } else {
                self.expr(0, true);
            }
            if !self.eat(",") {
                break;
            }
        }
        self.expect(closing);
    }
    fn arguments(&mut self) {
        self.start(Kind::Arguments);
        self.expect("(");
        while !self.at(")")
            && !matches!(
                self.kind(),
                Kind::ImplicitClose | Kind::Newline | Kind::Eof | Kind::Dedent
            )
        {
            self.argument();
            if !self.eat(",") {
                break;
            }
        }
        if self.eat_kind(Kind::ImplicitClose) {
            self.implicit_closed = true;
        } else {
            self.expect(")");
        }
        self.finish();
    }
    fn argument(&mut self) {
        self.start(Kind::Argument);
        if self.kind() == Kind::Ident && self.nth_text(1) == ":" {
            self.bump();
            self.bump();
            if !matches!(self.text(), "," | ")") {
                self.expr(0, true);
            }
        } else {
            self.eat("mut");
            self.expr(0, true);
        }
        self.finish();
    }
    fn eat_index(&mut self, cp: Checkpoint) -> bool {
        if !self.at("[") {
            return false;
        }
        let open = self.index();
        self.wrap(cp, Kind::Index);
        self.bump();
        let slice = self.checkpoint();
        if self.eat("..") || self.eat("..=") {
            self.wrap(slice, Kind::Slice);
            self.expr(7, true);
            self.finish();
        } else {
            // A top-level range inside the brackets makes this a slice; its
            // bounds are `additive` expressions. Precomputed per `[`, so the
            // lookahead is constant and nested indexing stays linear.
            self.expr(if self.range_inside[open] { 7 } else { 0 }, true);
            if self.eat("..") || self.eat("..=") {
                self.wrap(slice, Kind::Slice);
                if !self.at("]") {
                    self.expr(7, true);
                }
                self.finish();
            }
        }
        self.expect("]");
        self.finish();
        true
    }
}
