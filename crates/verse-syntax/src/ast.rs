//! Trivia-free, per-module arenas with typed IDs (architecture 02 §2.3).
use crate::{FileId, Kind, Span, SyntaxNode};
use std::{collections::HashMap, ops::Range};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Symbol(pub u32);

#[derive(Debug, Default)]
pub struct Interner {
    strings: Vec<String>,
    lookup: HashMap<String, Symbol>,
}
impl Interner {
    pub fn intern(&mut self, text: &str) -> Symbol {
        if let Some(&symbol) = self.lookup.get(text) {
            return symbol;
        }
        let symbol = Symbol(self.strings.len().try_into().expect("too many symbols"));
        self.strings.push(text.into());
        self.lookup.insert(text.into(), symbol);
        symbol
    }
    pub fn resolve(&self, symbol: Symbol) -> &str {
        &self.strings[symbol.0 as usize]
    }
}

#[derive(Debug)]
pub struct Node {
    pub kind: Kind,
    pub span: Span,
    pub children: Range<usize>,
    pub tokens: Range<usize>,
}
#[derive(Debug, Default)]
pub struct Module {
    pub nodes: Vec<Node>,
    pub edges: Vec<NodeId>,
    pub tokens: Vec<(Kind, Symbol)>,
    pub symbols: Interner,
    pub root: Option<NodeId>,
}
impl Module {
    pub fn lower(syntax: &SyntaxNode, file: FileId) -> Self {
        let mut module = Self::default();
        module.root = Some(module.append(syntax, file, 0));
        module
    }

    /// Lower embedded expression trees without reparsing them in a later stage.
    pub fn lower_parse(parsed: &crate::Parse, file: FileId) -> Self {
        let mut module = Self::lower(&parsed.syntax(), file);
        let mut literals: Vec<_> = module
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| n.kind == Kind::Literal)
            .map(|(i, n)| (n.span, NodeId(i as u32)))
            .collect();
        literals.sort_by_key(|(span, _)| span.start);
        let mut children = Vec::new();
        for interpolation in &parsed.interpolations {
            let candidate =
                literals.partition_point(|(span, _)| span.start <= interpolation.expression.start);
            if let Some((span, literal)) = candidate.checked_sub(1).and_then(|i| literals.get(i))
                && span.end >= interpolation.expression.end
            {
                let child = module.append(
                    &SyntaxNode::new_root(interpolation.green.clone()),
                    file,
                    interpolation.expression.start,
                );
                children.push((*literal, child));
            }
        }
        children.sort_by_key(|(parent, _)| parent.0);
        let mut index = 0;
        while index < children.len() {
            let parent = children[index].0;
            let start = module.edges.len();
            while index < children.len() && children[index].0 == parent {
                module.edges.push(children[index].1);
                index += 1;
            }
            module.nodes[parent.0 as usize].children = start..module.edges.len();
        }
        module
    }

    fn append(&mut self, syntax: &SyntaxNode, file: FileId, offset: u32) -> NodeId {
        fn allocate(module: &mut Module, syntax: &SyntaxNode, file: FileId) -> NodeId {
            let id = NodeId(module.nodes.len().try_into().expect("too many nodes"));
            module.nodes.push(Node {
                kind: syntax.kind(),
                span: Span {
                    file_id: file,
                    start: syntax.text_range().start().into(),
                    end: syntax.text_range().end().into(),
                },
                children: 0..0,
                tokens: 0..0,
            });
            id
        }
        let module = self;
        let first_node = module.nodes.len();
        let root = allocate(module, syntax, file);
        let mut pending = vec![(root, syntax.clone())];
        while let Some((id, syntax)) = pending.pop() {
            let children_start = module.edges.len();
            let tokens_start = module.tokens.len();
            for element in syntax.children_with_tokens() {
                match element {
                    rowan::NodeOrToken::Node(n) => {
                        let child = allocate(module, &n, file);
                        module.edges.push(child);
                        pending.push((child, n));
                    }
                    rowan::NodeOrToken::Token(t)
                        if !t.kind().is_trivia()
                            && !t.kind().is_layout()
                            && t.kind() != Kind::DocComment =>
                    {
                        let symbol = module.symbols.intern(t.text());
                        module.tokens.push((t.kind(), symbol));
                    }
                    _ => {}
                }
            }
            module.nodes[id.0 as usize].children = children_start..module.edges.len();
            module.nodes[id.0 as usize].tokens = tokens_start..module.tokens.len();
        }
        for node in &mut module.nodes[first_node..] {
            node.span.start += offset;
            node.span.end += offset;
        }
        root
    }
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0 as usize]
    }
}
