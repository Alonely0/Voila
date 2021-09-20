use super::Call;
use super::HasSpan;
use std::ops::Range;

/// A cycle is a group of calls that are run asynchronously.
/// Every cycle must finish completely before the next cycle
/// starts executing.
///
/// # Safety
/// Note that inside the same cycle, all the functions will be executed
/// in their own thread. This means that destructive functions might cause
/// data races in the system, so use them carefully!
///
/// # Example
/// ```voila
/// @size=gb > 1 {
///     print(@name is too big. Deleting it)
///     delete(@path)
///     ;
///     print(@name has been deleted)
/// }
/// ```
///
/// In this example, first `print` will be executid while the file is being deleted,
/// and the second `pritn` will be executed when the file has been deleted successfully.
#[derive(Debug)]
pub struct Cycle<'source> {
    pub calls: Vec<Call<'source>>,
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
        parser.with_context(ContextLevel::Cycle, |parser| {
            let start = parser.offset();
            let mut calls = Vec::new();
            loop {
                match parser.expect_one_of_tokens(
                    &[Token::CloseBrace, Token::Identifier, Token::Semicolon],
                    Some("safe/unsafe function or end of cycle/target"),
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

use crate::interpreter;
use std::sync::{mpsc, Arc, Mutex};

pub fn run_cycle(
    cycle: &Cycle,
    cache: Arc<Mutex<interpreter::Cache>>,
    pool: &rayon::ThreadPool,
    tx: mpsc::Sender<interpreter::ErrorKind>,
) {
    pool.scope(move |s| {
        for call in &cycle.calls {
            let cache = cache.clone();
            let tx = tx.clone();
            s.spawn(move |_| {
                if let Err(e) = super::run_call(call, cache) {
                    tx.send(e).unwrap();
                }
            })
        }
    });
}
