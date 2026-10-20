//! Target-independent source syntax. Stage contracts: architecture 02 §2.2.
pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod source;
pub mod tree;

pub use diagnostic::{Diagnostic, explain};
pub use lexer::{Lexed, Token, lex};
pub use parser::{Parse, parse};
pub use source::{FileId, SourceFile, Span};
pub use tree::{Kind, SyntaxNode, SyntaxToken};
