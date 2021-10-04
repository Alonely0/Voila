use super::HasSpan;
use super::{Cycle, Expr};
use std::ops::Range;

/// A target is the combination of an (optional) [Expr] as its condition
/// and a block of [Cycle]s to execute.
///
/// # Examples
///
/// ```voila
/// @size=mb > 1 { delete(@name) }
/// ```
/// ```voila
/// { print(Found @name => @path) }
/// ```
#[derive(Debug)]
pub struct Target<'source> {
    pub condition: Option<Expr<'source>>,
    pub cycles: Vec<Cycle<'source>>,
    pub span: Range<usize>,
}

impl HasSpan for Target<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

use super::parser::*;
use super::Token;

impl<'source> Parse<'source> for Target<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        let res = match parser.expect_one_of_tokens(
            &[Token::OpenBrace, Token::Identifier, Token::Variable],
            Some("as the start of a target"),
        )? {
            Token::OpenBrace => {
                let start = parser.current_token_span().start;
                parser.accept_current();
                let cycles = parser.with_context(ContextLevel::TargetBlock, |parser| {
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
                let cycles = parser.with_context(ContextLevel::TargetBlock, |parser| {
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
    }
}

use crate::interpreter;
use std::sync::{mpsc, Arc, Mutex};
pub fn run_target(
    target: &Target,
    cache: Arc<Mutex<interpreter::Cache>>,
    pool: &rayon::ThreadPool,
    tx: mpsc::Sender<interpreter::ErrorKind>,
) -> Result<(), interpreter::ErrorKind> {
    let ok = target
        .condition
        .as_ref()
        .map_or(Ok(true.into()), |expr| cache.lock().unwrap().resolve(expr))?
        .cast_to_bool()?;
    if !ok {
        return Ok(());
    }

    for cycle in &target.cycles {
        super::run_cycle(cycle, cache.clone(), pool, tx.clone());
    }
    Ok(())
}
