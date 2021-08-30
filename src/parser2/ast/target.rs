use super::{Cycle, Expr};
use std::ops::Range;

#[derive(Debug)]
pub struct Target {
    condition: Option<Expr>,
    cycles: Vec<Cycle>,
    span: Range<usize>,
}

use super::parser::*;
use super::Token;

impl Parse for Target {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        parser.with_context("parsing target", |parser| {
            let res = match parser.expect_one_of_tokens(
                &[Token::OpenBrace, Token::Identifier, Token::Variable],
                Some("as the start of a target"),
            )? {
                Token::OpenBrace => {
                    let start = parser.current_token_span().start;
                    parser.accept_current();
                    let cycles = parser.with_context("parsing target cycles", |parser| {
                        parser.repeat_until_token(Token::CloseBrace, Parser::parse)
                    })?;
                    let end = parser.offset();
                    Ok(Self {
                        condition: None,
                        cycles,
                        span: start..end,
                    })
                },
                Token::Identifier | Token::Variable => {
                    let start = parser.current_token_span().start;
                    let expr = parser.parse()?;
                    parser.expect_token(
                        Token::OpenBrace,
                        Some("to start the block executed by the target"),
                    )?;
                    parser.accept_current();
                    let cycles = parser.with_context("parsing target cycles", |parser| {
                        parser.repeat_until_token(Token::CloseBrace, Parser::parse)
                    })?;
                    let end = parser.offset();
                    Ok(Self {
                        condition: Some(expr),
                        cycles,
                        span: start..end,
                    })
                },
                _ => unreachable!(),
            }?;
            parser.expect_token(
                Token::CloseBrace,
                Some("to end the block executed by the target"),
            )?;
            parser.accept_current();
            Ok(res)
        })
    }
}
