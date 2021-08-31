use super::Call;
use super::HasSpan;
use std::ops::Range;

#[derive(Debug)]
pub struct Cycle<'source> {
    calls: Vec<Call<'source>>,
    span: Range<usize>,
}

impl HasSpan for Cycle<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

use super::parser::*;
use super::Token;

impl<'source> Parse<'source> for Cycle<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context("parsing cycle", |parser| {
            let start = parser.offset();
            let mut calls = Vec::new();
            loop {
                match parser.expect_one_of_tokens(
                    &[Token::CloseBrace, Token::Identifier, Token::Semicolon],
                    Some("function or end of cycle/target"),
                )? {
                    Token::CloseBrace => break,
                    Token::Semicolon => {
                        parser.accept_current();
                        break;
                    },
                    Token::Identifier => calls.push(parser.parse()?),
                    _ => unreachable!(),
                }
            }
            let end = parser.offset();
            Ok(Self {
                calls,
                span: start..end,
            })
        })
    }
}
