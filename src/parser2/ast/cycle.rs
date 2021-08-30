use super::Call;
use std::ops::Range;

#[derive(Debug)]
pub struct Cycle {
    calls: Vec<Call>,
    span: Range<usize>,
}

use super::parser::*;
use super::Token;

impl Parse for Cycle {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
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
