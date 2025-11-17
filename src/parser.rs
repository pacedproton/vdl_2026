//! Recursive descent parser for VDL++

use crate::ast::*;
use crate::error::{Result, VdlError};
use crate::lexer::{Lexer, Token};
use std::collections::BTreeMap;

/// Parser for VDL++ specifications
pub struct Parser {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    pos: usize,
}

impl Parser {
    pub fn new(source: &str) -> Result<Self> {
        let lexer = Lexer::new(source);
        let mut tokens = Vec::new();

        for (result, span) in lexer {
            match result {
                Ok(token) => tokens.push((token, span)),
                Err(_) => {
                    return Err(VdlError::LexerError {
                        position: span.start,
                        message: "Invalid token".to_string(),
                    })
                }
            }
        }

        Ok(Self { tokens, pos: 0 })
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos).map(|(t, _)| t)
    }

    fn peek_span(&self) -> Option<&std::ops::Range<usize>> {
        self.tokens.get(self.pos).map(|(_, s)| s)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos).map(|(t, _)| t);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        let pos = self.peek_span().map(|s| s.start).unwrap_or(0);
        match self.peek() {
            Some(t) if *t == expected => {
                self.advance();
                Ok(())
            }
            Some(t) => Err(VdlError::ParseError {
                position: pos,
                message: format!("Expected {:?}, found {:?}", expected, t),
            }),
            None => Err(VdlError::ParseError {
                position: pos,
                message: format!("Expected {:?}, found end of input", expected),
            }),
        }
    }

    fn expect_ident(&mut self) -> Result<String> {
        let pos = self.peek_span().map(|s| s.start).unwrap_or(0);
        match self.advance().cloned() {
            Some(Token::Ident(s)) => Ok(s),
            Some(t) => Err(VdlError::ParseError {
                position: pos,
                message: format!("Expected identifier, found {:?}", t),
            }),
            None => Err(VdlError::ParseError {
                position: pos,
                message: "Expected identifier, found end of input".to_string(),
            }),
        }
    }

    /// Parse a complete VDL++ specification
    pub fn parse_specification(&mut self) -> Result<Specification> {
        let mut spec = Specification::new();

        while self.peek().is_some() {
            match self.peek() {
                Some(Token::Type) => {
                    spec.type_defs.push(self.parse_type_def()?);
                }
                Some(Token::Fun) => {
                    spec.functions.push(self.parse_function_def()?);
                }
                Some(Token::Transition) => {
                    spec.transitions.push(self.parse_transition_def()?);
                }
                Some(Token::Inv) => {
                    spec.invariants.push(self.parse_invariant_def()?);
                }
                Some(t) => {
                    let pos = self.peek_span().map(|s| s.start).unwrap_or(0);
                    return Err(VdlError::ParseError {
                        position: pos,
                        message: format!(
                            "Expected 'type', 'fun', 'transition', or 'inv', found {:?}",
                            t
                        ),
                    });
                }
                None => break,
            }
        }

        Ok(spec)
    }

    /// Parse type definition: type Name = Type inv expr
    fn parse_type_def(&mut self) -> Result<TypeDef> {
        let start = self.peek_span().map(|s| s.start).unwrap_or(0);
        self.expect(Token::Type)?;
        let name = self.expect_ident()?;
        self.expect(Token::Eq)?;
        let ty = self.parse_type()?;

        let invariant = if self.peek() == Some(&Token::Inv) {
            self.advance();
            let _param = self.expect_ident()?; // invariant parameter
            self.expect(Token::FatArrow)?;
            Some(self.parse_expr()?)
        } else {
            None
        };

        let end = self.pos;
        Ok(TypeDef {
            name,
            ty,
            invariant,
            span: Some(Span { start, end }),
        })
    }

    /// Parse a type
    fn parse_type(&mut self) -> Result<Type> {
        match self.peek().cloned() {
            Some(Token::TyNat) => {
                self.advance();
                Ok(Type::Nat)
            }
            Some(Token::TyInt) => {
                self.advance();
                Ok(Type::Int)
            }
            Some(Token::TyBool) => {
                self.advance();
                Ok(Type::Bool)
            }
            Some(Token::TySet) => {
                self.advance();
                self.expect(Token::Lt)?;
                let inner = self.parse_type()?;
                self.expect(Token::Gt)?;
                Ok(Type::Set(Box::new(inner)))
            }
            Some(Token::TyList) => {
                self.advance();
                self.expect(Token::Lt)?;
                let inner = self.parse_type()?;
                self.expect(Token::Gt)?;
                Ok(Type::List(Box::new(inner)))
            }
            Some(Token::TyMap) => {
                self.advance();
                self.expect(Token::Lt)?;
                let key = self.parse_type()?;
                self.expect(Token::Comma)?;
                let val = self.parse_type()?;
                self.expect(Token::Gt)?;
                Ok(Type::Map(Box::new(key), Box::new(val)))
            }
            Some(Token::LBrace) => {
                self.advance();
                let mut fields = BTreeMap::new();
                while self.peek() != Some(&Token::RBrace) {
                    let field_name = self.expect_ident()?;
                    self.expect(Token::Colon)?;
                    let field_type = self.parse_type()?;
                    fields.insert(field_name, field_type);
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBrace)?;
                Ok(Type::Record(fields))
            }
            Some(Token::Ident(name)) => {
                self.advance();
                Ok(Type::Named(name))
            }
            Some(Token::LParen) => {
                self.advance();
                self.expect(Token::RParen)?;
                Ok(Type::Unit)
            }
            _ => {
                let pos = self.peek_span().map(|s| s.start).unwrap_or(0);
                Err(VdlError::ParseError {
                    position: pos,
                    message: "Expected type".to_string(),
                })
            }
        }
    }

    /// Parse function definition: fun name(params) -> Type = expr
    fn parse_function_def(&mut self) -> Result<FunctionDef> {
        let start = self.peek_span().map(|s| s.start).unwrap_or(0);
        self.expect(Token::Fun)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        while self.peek() != Some(&Token::RParen) {
            let param_name = self.expect_ident()?;
            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;
            params.push((param_name, param_type));
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RParen)?;

        self.expect(Token::Arrow)?;
        let return_type = self.parse_type()?;
        self.expect(Token::Eq)?;
        let body = self.parse_expr()?;

        let end = self.pos;
        Ok(FunctionDef {
            name,
            params,
            return_type,
            body,
            span: Some(Span { start, end }),
        })
    }

    /// Parse transition definition
    fn parse_transition_def(&mut self) -> Result<TransitionDef> {
        let start = self.peek_span().map(|s| s.start).unwrap_or(0);
        self.expect(Token::Transition)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;

        let mut params = Vec::new();
        let mut state_param = String::new();
        let mut state_type = Type::Unit;

        while self.peek() != Some(&Token::RParen) {
            let param_name = self.expect_ident()?;
            self.expect(Token::Colon)?;
            let param_type = self.parse_type()?;

            // Last parameter is the state parameter
            if self.peek() == Some(&Token::RParen) {
                state_param = param_name;
                state_type = param_type;
            } else {
                params.push((param_name, param_type));
                if self.peek() == Some(&Token::Comma) {
                    self.advance();
                }
            }
        }
        self.expect(Token::RParen)?;

        self.expect(Token::Arrow)?;
        let _return_type = self.parse_type()?; // Should match state_type

        let precondition = if self.peek() == Some(&Token::Pre) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect(Token::Post)?;
        let post_var = self.expect_ident()?;
        self.expect(Token::FatArrow)?;
        let postcondition = self.parse_expr()?;

        // Wrap postcondition to include post-state variable
        let postcondition = if post_var != "_" {
            Expr::Let(
                post_var,
                Box::new(Expr::PostState("__new_state__".to_string())),
                Box::new(postcondition),
            )
        } else {
            postcondition
        };

        let end = self.pos;
        Ok(TransitionDef {
            name,
            params,
            state_param,
            state_type,
            precondition,
            postcondition,
            span: Some(Span { start, end }),
        })
    }

    /// Parse invariant definition
    fn parse_invariant_def(&mut self) -> Result<InvariantDef> {
        let start = self.peek_span().map(|s| s.start).unwrap_or(0);
        self.expect(Token::Inv)?;
        let name = self.expect_ident()?;
        self.expect(Token::LParen)?;
        let state_param = self.expect_ident()?;
        self.expect(Token::Colon)?;
        let state_type = self.parse_type()?;
        self.expect(Token::RParen)?;
        self.expect(Token::FatArrow)?;
        let condition = self.parse_expr()?;

        let end = self.pos;
        Ok(InvariantDef {
            name,
            state_param,
            state_type,
            condition,
            span: Some(Span { start, end }),
        })
    }

    /// Parse expression (with operator precedence)
    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_implies()
    }

    fn parse_implies(&mut self) -> Result<Expr> {
        let mut left = self.parse_or()?;
        while self.peek() == Some(&Token::Arrow) {
            self.advance();
            let right = self.parse_or()?;
            left = Expr::Implies(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut left = self.parse_and()?;
        while self.peek() == Some(&Token::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp(Box::new(left), BinOp::Or, Box::new(right));
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut left = self.parse_comparison()?;
        while self.peek() == Some(&Token::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp(Box::new(left), BinOp::And, Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr> {
        let left = self.parse_additive()?;
        match self.peek().cloned() {
            Some(Token::EqEq) => {
                self.advance();
                let right = self.parse_additive()?;
                Ok(Expr::BinOp(Box::new(left), BinOp::Eq, Box::new(right)))
            }
            Some(Token::NotEq) => {
                self.advance();
                let right = self.parse_additive()?;
                Ok(Expr::BinOp(Box::new(left), BinOp::Ne, Box::new(right)))
            }
            Some(Token::Lt) => {
                self.advance();
                let right = self.parse_additive()?;
                Ok(Expr::BinOp(Box::new(left), BinOp::Lt, Box::new(right)))
            }
            Some(Token::LtEq) => {
                self.advance();
                let right = self.parse_additive()?;
                Ok(Expr::BinOp(Box::new(left), BinOp::Le, Box::new(right)))
            }
            Some(Token::Gt) => {
                self.advance();
                let right = self.parse_additive()?;
                Ok(Expr::BinOp(Box::new(left), BinOp::Gt, Box::new(right)))
            }
            Some(Token::GtEq) => {
                self.advance();
                let right = self.parse_additive()?;
                Ok(Expr::BinOp(Box::new(left), BinOp::Ge, Box::new(right)))
            }
            _ => Ok(left),
        }
    }

    fn parse_additive(&mut self) -> Result<Expr> {
        let mut left = self.parse_multiplicative()?;
        loop {
            match self.peek().cloned() {
                Some(Token::Plus) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Add, Box::new(right));
                }
                Some(Token::Minus) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Sub, Box::new(right));
                }
                Some(Token::PlusPlus) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::ListAppend(Box::new(left), Box::new(right));
                }
                Some(Token::Union) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::SetUnion(Box::new(left), Box::new(right));
                }
                Some(Token::Intersect) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::SetIntersect(Box::new(left), Box::new(right));
                }
                Some(Token::Backslash) => {
                    self.advance();
                    let right = self.parse_multiplicative()?;
                    left = Expr::SetDiff(Box::new(left), Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek().cloned() {
                Some(Token::Star) => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Mul, Box::new(right));
                }
                Some(Token::Slash) => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Div, Box::new(right));
                }
                Some(Token::Percent) => {
                    self.advance();
                    let right = self.parse_unary()?;
                    left = Expr::BinOp(Box::new(left), BinOp::Mod, Box::new(right));
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek().cloned() {
            Some(Token::Not) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Not, Box::new(expr)))
            }
            Some(Token::Minus) => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::UnaryOp(UnaryOp::Neg, Box::new(expr)))
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek().cloned() {
                Some(Token::Dot) => {
                    self.advance();
                    let field = self.expect_ident()?;
                    expr = Expr::FieldAccess(Box::new(expr), field);
                }
                Some(Token::LParen) => {
                    self.advance();
                    let mut args = Vec::new();
                    while self.peek() != Some(&Token::RParen) {
                        args.push(self.parse_expr()?);
                        if self.peek() == Some(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RParen)?;
                    expr = Expr::App(Box::new(expr), args);
                }
                Some(Token::With) => {
                    self.advance();
                    self.expect(Token::LBracket)?;
                    let mut updates = Vec::new();
                    while self.peek() != Some(&Token::RBracket) {
                        let field = self.expect_ident()?;
                        self.expect(Token::Eq)?;
                        let value = self.parse_expr()?;
                        updates.push((field, value));
                        if self.peek() == Some(&Token::Comma) {
                            self.advance();
                        }
                    }
                    self.expect(Token::RBracket)?;
                    expr = Expr::RecordUpdate(Box::new(expr), updates);
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.peek().cloned() {
            Some(Token::NatLit(n)) => {
                self.advance();
                Ok(Expr::NatLit(n))
            }
            Some(Token::IntLit(n)) => {
                self.advance();
                Ok(Expr::IntLit(n))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::BoolLit(true))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::BoolLit(false))
            }
            Some(Token::LBrace) => self.parse_record_or_set(),
            Some(Token::LBracket) => {
                self.advance();
                let mut elements = Vec::new();
                while self.peek() != Some(&Token::RBracket) {
                    elements.push(self.parse_expr()?);
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::ListLit(elements))
            }
            Some(Token::Let) => {
                self.advance();
                let name = self.expect_ident()?;
                self.expect(Token::Eq)?;
                let value = self.parse_expr()?;
                self.expect(Token::In)?;
                let body = self.parse_expr()?;
                Ok(Expr::Let(name, Box::new(value), Box::new(body)))
            }
            Some(Token::If) => {
                self.advance();
                let cond = self.parse_expr()?;
                self.expect(Token::Then)?;
                let then_branch = self.parse_expr()?;
                self.expect(Token::Else)?;
                let else_branch = self.parse_expr()?;
                Ok(Expr::If(
                    Box::new(cond),
                    Box::new(then_branch),
                    Box::new(else_branch),
                ))
            }
            Some(Token::Forall) => {
                self.advance();
                let var = self.expect_ident()?;
                self.expect(Token::In)?;
                let domain = self.parse_expr()?;
                self.expect(Token::Colon)?;
                let pred = self.parse_expr()?;
                Ok(Expr::Forall(var, Box::new(domain), Box::new(pred)))
            }
            Some(Token::Exists) => {
                self.advance();
                let var = self.expect_ident()?;
                self.expect(Token::In)?;
                let domain = self.parse_expr()?;
                self.expect(Token::Colon)?;
                let pred = self.parse_expr()?;
                Ok(Expr::Exists(var, Box::new(domain), Box::new(pred)))
            }
            Some(Token::Old) => {
                self.advance();
                self.expect(Token::LParen)?;
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr::Old(Box::new(expr)))
            }
            Some(Token::Member) => {
                self.advance();
                self.expect(Token::LParen)?;
                let elem = self.parse_expr()?;
                self.expect(Token::Comma)?;
                let set = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr::SetMember(Box::new(elem), Box::new(set)))
            }
            Some(Token::IsEmpty) => {
                self.advance();
                self.expect(Token::LParen)?;
                let coll = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr::SetEmpty(Box::new(coll)))
            }
            Some(Token::Length) => {
                self.advance();
                self.expect(Token::LParen)?;
                let list = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr::ListLength(Box::new(list)))
            }
            Some(Token::Head) => {
                self.advance();
                self.expect(Token::LParen)?;
                let list = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr::ListHead(Box::new(list)))
            }
            Some(Token::Tail) => {
                self.advance();
                self.expect(Token::LParen)?;
                let list = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(Expr::ListTail(Box::new(list)))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(Token::RParen)?;
                Ok(expr)
            }
            Some(Token::Ident(name)) => {
                self.advance();
                Ok(Expr::Var(name))
            }
            _ => {
                let pos = self.peek_span().map(|s| s.start).unwrap_or(0);
                Err(VdlError::ParseError {
                    position: pos,
                    message: "Expected expression".to_string(),
                })
            }
        }
    }

    fn parse_record_or_set(&mut self) -> Result<Expr> {
        self.expect(Token::LBrace)?;

        if self.peek() == Some(&Token::RBrace) {
            self.advance();
            return Ok(Expr::SetLit(Vec::new()));
        }

        // Peek ahead to determine if this is a record or set
        let saved_pos = self.pos;
        let first_ident = matches!(self.peek(), Some(Token::Ident(_)));

        if first_ident {
            let _name = self.expect_ident().ok();
            if self.peek() == Some(&Token::Eq) {
                // This is a record literal
                self.pos = saved_pos;
                let mut fields = BTreeMap::new();
                while self.peek() != Some(&Token::RBrace) {
                    let field_name = self.expect_ident()?;
                    self.expect(Token::Eq)?;
                    let field_value = self.parse_expr()?;
                    fields.insert(field_name, field_value);
                    if self.peek() == Some(&Token::Comma) {
                        self.advance();
                    }
                }
                self.expect(Token::RBrace)?;
                return Ok(Expr::RecordLit(fields));
            }
        }

        // This is a set literal
        self.pos = saved_pos;
        let mut elements = Vec::new();
        while self.peek() != Some(&Token::RBrace) {
            elements.push(self.parse_expr()?);
            if self.peek() == Some(&Token::Comma) {
                self.advance();
            }
        }
        self.expect(Token::RBrace)?;
        Ok(Expr::SetLit(elements))
    }
}

/// Parse a VDL++ source file
pub fn parse(source: &str) -> Result<Specification> {
    let mut parser = Parser::new(source)?;
    parser.parse_specification()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_def() {
        let src = r#"
            type ReaderId = Nat
        "#;
        let spec = parse(src).unwrap();
        assert_eq!(spec.type_defs.len(), 1);
        assert_eq!(spec.type_defs[0].name, "ReaderId");
    }

    #[test]
    fn test_parse_record_type() {
        let src = r#"
            type RCU = {
                readers: Set<Nat>,
                epoch: Nat,
                gpActive: Bool
            }
        "#;
        let spec = parse(src).unwrap();
        assert_eq!(spec.type_defs.len(), 1);
    }
}
