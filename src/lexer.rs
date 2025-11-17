//! Lexer for VDL++ using logos

use logos::Logos;

/// Token types for VDL++
#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"//[^\n]*")]
pub enum Token {
    // Keywords
    #[token("type")]
    Type,
    #[token("fun")]
    Fun,
    #[token("transition")]
    Transition,
    #[token("inv")]
    Inv,
    #[token("pre")]
    Pre,
    #[token("post")]
    Post,
    #[token("let")]
    Let,
    #[token("in")]
    In,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("forall")]
    Forall,
    #[token("exists")]
    Exists,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("old")]
    Old,
    #[token("with")]
    With,

    // Type keywords
    #[token("Nat")]
    TyNat,
    #[token("Int")]
    TyInt,
    #[token("Bool")]
    TyBool,
    #[token("Set")]
    TySet,
    #[token("List")]
    TyList,
    #[token("Map")]
    TyMap,

    // Set/List operations
    #[token("union")]
    Union,
    #[token("intersect")]
    Intersect,
    #[token("isEmpty")]
    IsEmpty,
    #[token("member")]
    Member,
    #[token("length")]
    Length,
    #[token("head")]
    Head,
    #[token("tail")]
    Tail,
    #[token("filter")]
    Filter,

    // Operators
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<=")]
    LtEq,
    #[token(">=")]
    GtEq,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Eq,
    #[token("\\")]
    Backslash,
    #[token("++")]
    PlusPlus,
    #[token("::")]
    ColonColon,

    // Delimiters
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token(";")]
    Semicolon,
    #[token(".")]
    Dot,
    #[token("|")]
    Pipe,

    // Literals
    #[regex(r"[0-9]+", |lex| lex.slice().parse().ok())]
    NatLit(u64),

    #[regex(r"-[0-9]+", |lex| lex.slice().parse().ok())]
    IntLit(i64),

    // Identifiers
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),
}

/// Lexer wrapper
pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = (Result<Token, ()>, std::ops::Range<usize>);

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.inner.next()?;
        let span = self.inner.span();
        Some((token, span))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "type RCU = { readers: Set<Nat> }";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|(t, _)| t).collect();
        assert!(tokens.len() > 0);
    }

    #[test]
    fn test_transition() {
        let input = "transition rcu_read_lock(r: Nat, s: RCU) -> RCU";
        let lexer = Lexer::new(input);
        let tokens: Vec<_> = lexer.map(|(t, _)| t.unwrap()).collect();
        assert_eq!(tokens[0], Token::Transition);
    }
}
