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
/// string with variables in it
///
/// # Examples
///
/// - `@name`
/// - `@name is too big`
/// - `hello world`
#[derive(Debug)]
pub enum Arg<'source> {
    Str(&'source str, Range<usize>),
    Lookup(Lookup, Range<usize>),
    Interpolate {
        lookups: Vec<Lookup>,
        sequence: Vec<(Option<&'source str>, Range<usize>)>,
        span: Range<usize>,
    },
}

impl HasSpan for Arg<'_> {
    fn span(&self) -> &Range<usize> {
        match self {
            Self::Str(_, span) | Self::Lookup(_, span) | Self::Interpolate { span, .. } => span,
        }
    }
}

impl<'source> Arg<'source> {
    fn extend_var(self, var_lookup: Lookup, var_span: &Range<usize>) -> Self {
        match self {
            Self::Str(str, str_span) => Self::Interpolate {
                lookups: vec![var_lookup],
                span: str_span.start..var_span.end,
                sequence: vec![(Some(str), str_span), (None, var_span.clone())],
            },
            Self::Lookup(lookup, lookup_span) => Self::Interpolate {
                lookups: vec![lookup, var_lookup],
                span: lookup_span.start..var_span.end,
                sequence: vec![(None, lookup_span), (None, var_span.clone())],
            },
            Self::Interpolate {
                mut lookups,
                mut sequence,
                span: interp_span,
            } => {
                lookups.push(var_lookup);
                sequence.push((None, var_span.clone()));
                Self::Interpolate {
                    lookups,
                    sequence,
                    span: interp_span.start..var_span.end,
                }
            },
        }
    }
    fn extend_str(self, str_src: &'source str, span: &Range<usize>, source: &'source str) -> Self {
        match self {
            Self::Str(_, mut first_span) => {
                // extend the span
                first_span.end = span.end;
                Self::Str(&source[first_span.start..first_span.end], first_span)
            },
            Self::Lookup(var_name, var_span) => Self::Interpolate {
                lookups: vec![var_name],
                span: var_span.start..span.end,
                sequence: vec![(None, var_span), (Some(str_src), span.clone())],
            },
            Self::Interpolate {
                lookups,
                mut sequence,
                span: interp_span,
            } => {
                sequence.push((Some(str_src), span.clone()));
                Self::Interpolate {
                    lookups,
                    sequence,
                    span: interp_span.start..span.end,
                }
            },
        }
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
                let mut arg = match parser.expect_one_of_tokens(
                    &[Token::CloseParen, Token::Identifier, Token::Variable],
                    Some("argument to the function call or end the call"),
                )? {
                    Token::CloseParen => {
                        break;
                    },
                    Token::Identifier => {
                        let src = parser.current_token_source();
                        let span = parser.current_token_span().clone();
                        Arg::Str(src, span)
                    },
                    // TODO: check lookups? as they are known at first time
                    Token::Variable => {
                        let src = parser.parse()?;
                        let span = parser.current_token_span().clone();
                        Arg::Lookup(src, span)
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
                            let span = parser.current_token_span();
                            arg = arg.extend_var(src, &span);
                            parser.accept_current();
                        },
                        Token::Identifier => {
                            let src = parser.current_token_source();
                            let span = parser.current_token_span();
                            arg = arg.extend_str(src, &span, parser.source());
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
