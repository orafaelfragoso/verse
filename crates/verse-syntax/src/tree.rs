/// Lexical and concrete syntax kinds. Virtual layout tokens have empty text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum Kind {
    Whitespace,
    Comment,
    DocComment,
    Newline,
    Blank,
    Indent,
    Dedent,
    ImplicitClose,
    Ident,
    TypeIdent,
    Keyword,
    Int,
    Float,
    Text,
    RawText,
    Char,
    Bytes,
    Symbol,
    ErrorToken,
    Eof,
    File,
    Import,
    Function,
    Parameters,
    Parameter,
    Generics,
    GenericParameter,
    TypeDeclaration,
    Field,
    Variant,
    Alias,
    Trait,
    Impl,
    Extern,
    Test,
    Type,
    Block,
    Binding,
    Assignment,
    If,
    For,
    While,
    Match,
    Arm,
    Pattern,
    Return,
    Break,
    Continue,
    Pass,
    Assert,
    ExpressionStatement,
    Name,
    Literal,
    Unary,
    Binary,
    Comparison,
    Call,
    Arguments,
    Argument,
    Index,
    Slice,
    FieldAccess,
    Group,
    Tuple,
    List,
    Map,
    Set,
    Entry,
    Spread,
    Lambda,
    KeyPath,
    Conditional,
    Error,
}

impl Kind {
    pub fn is_trivia(self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment | Self::Blank)
    }
    pub fn is_layout(self) -> bool {
        matches!(
            self,
            Self::Newline | Self::Indent | Self::Dedent | Self::ImplicitClose | Self::Eof
        )
    }
    pub fn is_literal(self) -> bool {
        matches!(
            self,
            Self::Int | Self::Float | Self::Text | Self::RawText | Self::Char | Self::Bytes
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VerseLanguage {}
impl rowan::Language for VerseLanguage {
    type Kind = Kind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Kind {
        // No unsafe conversion: the complete, dense enum is bounded here.
        const KINDS: &[Kind] = &[
            Kind::Whitespace,
            Kind::Comment,
            Kind::DocComment,
            Kind::Newline,
            Kind::Blank,
            Kind::Indent,
            Kind::Dedent,
            Kind::ImplicitClose,
            Kind::Ident,
            Kind::TypeIdent,
            Kind::Keyword,
            Kind::Int,
            Kind::Float,
            Kind::Text,
            Kind::RawText,
            Kind::Char,
            Kind::Bytes,
            Kind::Symbol,
            Kind::ErrorToken,
            Kind::Eof,
            Kind::File,
            Kind::Import,
            Kind::Function,
            Kind::Parameters,
            Kind::Parameter,
            Kind::Generics,
            Kind::GenericParameter,
            Kind::TypeDeclaration,
            Kind::Field,
            Kind::Variant,
            Kind::Alias,
            Kind::Trait,
            Kind::Impl,
            Kind::Extern,
            Kind::Test,
            Kind::Type,
            Kind::Block,
            Kind::Binding,
            Kind::Assignment,
            Kind::If,
            Kind::For,
            Kind::While,
            Kind::Match,
            Kind::Arm,
            Kind::Pattern,
            Kind::Return,
            Kind::Break,
            Kind::Continue,
            Kind::Pass,
            Kind::Assert,
            Kind::ExpressionStatement,
            Kind::Name,
            Kind::Literal,
            Kind::Unary,
            Kind::Binary,
            Kind::Comparison,
            Kind::Call,
            Kind::Arguments,
            Kind::Argument,
            Kind::Index,
            Kind::Slice,
            Kind::FieldAccess,
            Kind::Group,
            Kind::Tuple,
            Kind::List,
            Kind::Map,
            Kind::Set,
            Kind::Entry,
            Kind::Spread,
            Kind::Lambda,
            Kind::KeyPath,
            Kind::Conditional,
            Kind::Error,
        ];
        KINDS[raw.0 as usize]
    }
    fn kind_to_raw(kind: Kind) -> rowan::SyntaxKind {
        rowan::SyntaxKind(kind as u16)
    }
}
pub type SyntaxNode = rowan::SyntaxNode<VerseLanguage>;
pub type SyntaxToken = rowan::SyntaxToken<VerseLanguage>;

pub(crate) fn raw(kind: Kind) -> rowan::SyntaxKind {
    rowan::SyntaxKind(kind as u16)
}

/// Structural fingerprint for formatter tests: ignore trivia and layout text,
/// but retain grouping, token spelling, declarations, and expression structure.
pub fn fingerprint(node: &SyntaxNode) -> String {
    fn visit(node: &SyntaxNode, out: &mut String) {
        use std::fmt::Write;
        let _ = write!(out, "({:?}", node.kind());
        for child in node.children_with_tokens() {
            match child {
                rowan::NodeOrToken::Node(n) => visit(&n, out),
                rowan::NodeOrToken::Token(t)
                    if !t.kind().is_trivia()
                        && !t.kind().is_layout()
                        && t.kind() != Kind::DocComment
                        && t.text() != "," =>
                {
                    let _ = write!(out, " {:?}", t.text());
                }
                _ => {}
            }
        }
        out.push(')');
    }
    let mut out = String::new();
    visit(node, &mut out);
    out
}
