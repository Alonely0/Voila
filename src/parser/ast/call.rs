use super::parser::{Parse, ParseRes, Parser};
use super::HasSpan;
use super::Lookup;
use super::Token;
use std::ops::Range;

/// Represents a call in the script like
/// `shell` or `delete`. All functions receive the arguments in
/// interpolated strings, which will have their variables resolved
/// before execution.
///
/// # List of non-destructive functions
/// These functions are safe to use without any worries that
/// any data will be eliminated:
///     - `print`
///     - `mkdir`
/// `shell` won't be in the list, since you can do absolutely anything
/// when we give you a shell. The shell is an escape hatch to enable the
/// integration of Voila to the rest of the system.
#[derive(Debug)]
pub struct Call<'source> {
    function_name: &'source str,
    arguments: Vec<Arg<'source>>,
    span: Range<usize>,
}

impl HasSpan for Call<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

/// Represents an argument to the call, with either
/// a single string like `hello world` or an interpolated
/// string with variables in it.
///
/// The argument maintains an invariant: Its sequence is
/// never empty
///
/// # Examples
///
/// - `@name`
/// - `@name is too big`
/// - `hello world`
#[derive(Debug)]
pub struct Arg<'source> {
    sequence: Vec<InterpolateComponent<'source>>,
    span: Range<usize>,
}

#[derive(Debug)]
enum InterpolateComponent<'source> {
    Literal(&'source str),
    Lookup(Lookup),
}

impl HasSpan for Arg<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

// TODO: refactor this into just the interpolated sequence
impl<'source> Arg<'source> {
    /// Construct a literal string argument
    fn str(string: &'source str, span: Range<usize>) -> Self {
        Self {
            sequence: vec![InterpolateComponent::Literal(string)],
            span,
        }
    }
    /// Construct a lookup argument
    fn lookup(lookup: Lookup, span: Range<usize>) -> Self {
        Self {
            sequence: vec![InterpolateComponent::Lookup(lookup)],
            span,
        }
    }
    /// Extend the argument with a string literal, returning the next span
    /// (might be modified)
    fn extend_str(
        &mut self,
        last_span: Range<usize>,
        mut span: Range<usize>,
        source: &'source str,
    ) -> Range<usize> {
        if matches!(
            self.sequence.last().unwrap(),
            InterpolateComponent::Literal(_)
        ) {
            // if the last component was a literal,
            // we can just extend the source
            span.start = last_span.start;

            // UNSAFE: safe. we already unwrapped
            let last_ref = self.sequence.last_mut().unwrap();
            *last_ref = InterpolateComponent::Literal(&source[span.clone()]);
        } else {
            // if the last component was a variable,
            // we will extend the span to accomodate the space in between
            span.start = last_span.end;
            self.sequence
                .push(InterpolateComponent::Literal(&source[span.clone()]));
        }
        span
    }

    /// Extend the argument with a lookup, returning the next span
    fn extend_lookup(
        &mut self,
        lookup: Lookup,
        last_span: Range<usize>,
        span: Range<usize>,
        source: &'source str,
    ) -> Range<usize> {
        if matches!(
            self.sequence.last().unwrap(),
            InterpolateComponent::Literal(_)
        ) {
            // if the last component was a literal, we can
            // extend its source to accomodate the space in between
            let last_span = last_span.start..span.start;
            // UNSAFE: safe, we already unwrapped
            let last_ref = self.sequence.last_mut().unwrap();
            *last_ref = InterpolateComponent::Literal(&source[last_span]);
        } else {
            // otherwise, we will put the spaces as a literal into the sequence
            self.sequence.push(InterpolateComponent::Literal(
                &source[last_span.end..span.start],
            ));
        }

        // now we can safely push the lookup, as we already handled the space in between
        self.sequence.push(InterpolateComponent::Lookup(lookup));

        span
    }
}

impl<'source> Parse<'source> for Call<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context("parsing function call", |parser| {
            let function_name =
                parser.expect_token(Token::Identifier, Some("name for the function to call"))?;
            let start = parser.current_token_span().start;
            parser.accept_current();
            parser.expect_token(
                Token::OpenParen,
                Some("to begin the function call arguments"),
            )?;
            parser.accept_current();
            let mut arguments = Vec::new();

            // parsing arguments
            'outer: loop {
                let (mut arg, mut arg_span) = match parser.expect_one_of_tokens(
                    &[Token::CloseParen, Token::Identifier, Token::Variable],
                    Some("argument to the function call or end the call"),
                )? {
                    Token::CloseParen => {
                        break;
                    },
                    Token::Identifier => {
                        let src = parser.current_token_source();
                        let span = parser.current_token_span().clone();
                        (Arg::str(src, span.clone()), span)
                    },
                    // TODO: check lookups? as they are known at first time
                    Token::Variable => {
                        let src = parser.parse()?;
                        let span = parser.current_token_span().clone();
                        (Arg::lookup(src, span.clone()), span)
                    },
                    _ => unreachable!(),
                };
                parser.accept_current();
                loop {
                    match parser.expect_one_of_tokens(
                        &[
                            Token::CloseParen,
                            Token::Identifier,
                            Token::Variable,
                            Token::Comma,
                        ],
                        None,
                    )? {
                        Token::CloseParen => {
                            arguments.push(arg);
                            break 'outer;
                        },
                        Token::Comma => {
                            parser.accept_current();
                            break;
                        },
                        Token::Variable => {
                            let src = parser.parse()?;
                            let span = parser.current_token_span().clone();
                            arg_span = arg.extend_lookup(src, arg_span, span, parser.source());
                            parser.accept_current();
                        },
                        Token::Identifier => {
                            let span = parser.current_token_span().clone();
                            arg_span = arg.extend_str(arg_span, span, parser.source());
                            parser.accept_current();
                        },
                        _ => unreachable!(),
                    }
                }
                arguments.push(arg);
            }
            //     arguments.push(arg);
            // }
            parser.expect_token(Token::CloseParen, Some("to end the argument list"))?;
            let end = parser.current_token_span().end;
            parser.accept_current();

            Ok(Self {
                function_name,
                arguments,
                span: start..end,
            })
        })
    }
}
