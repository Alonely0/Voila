use super::parser::{Parse, ParseRes, Parser};
use super::Token;
use std::ops::Range;

#[derive(Debug)]
pub struct Call {
    function_name: String,
    arguments: Vec<Arg>,
    span: Range<usize>,
}

#[derive(Debug)]
enum Arg {
    Str(String, Range<usize>),
    Lookup(String, Range<usize>),
    Interpolate {
        lookups: Vec<String>,
        sequence: Vec<(Option<String>, Range<usize>)>,
        span: Range<usize>,
    },
}

impl Arg {
    fn extend_var(self, var_name: String, var_span: &Range<usize>) -> Self {
        match self {
            Self::Str(str, str_span) => Self::Interpolate {
                lookups: vec![var_name],
                span: str_span.start..var_span.end,
                sequence: vec![(Some(str), str_span), (None, var_span.clone())],
            },
            Self::Lookup(lookup, lookup_span) => Self::Interpolate {
                lookups: vec![lookup, var_name],
                span: lookup_span.start..var_span.end,
                sequence: vec![(None, lookup_span), (None, var_span.clone())],
            },
            Self::Interpolate {
                mut lookups,
                mut sequence,
                span: interp_span,
            } => {
                lookups.push(var_name);
                sequence.push((None, var_span.clone()));
                Self::Interpolate {
                    lookups,
                    sequence,
                    span: interp_span.start..var_span.end,
                }
            },
        }
    }
    fn extend_str(self, src: String, span: &Range<usize>, source: &str) -> Self {
        match self {
            Self::Str(mut first_src, mut first_span) => {
                let spaces = &source[first_span.end..span.start];
                // extend the original spaces
                first_src.push_str(spaces);
                // extend the literal
                first_src.push_str(&src);
                // extend the span
                first_span.end = span.end;
                Self::Str(first_src, first_span)
            },
            Self::Lookup(var_name, var_span) => Self::Interpolate {
                lookups: vec![var_name],
                span: var_span.start..span.end,
                sequence: vec![(None, var_span), (Some(src), span.clone())],
            },
            Self::Interpolate {
                lookups,
                mut sequence,
                span: interp_span,
            } => {
                sequence.push((Some(src), span.clone()));
                Self::Interpolate {
                    lookups,
                    sequence,
                    span: interp_span.start..span.end,
                }
            },
        }
    }
}
impl Parse for Call {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        parser.with_context("parsing function call", |parser| {
            let function_name = parser
                .expect_token(Token::Identifier, Some("name for the function to call"))?
                .to_string();
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
                        Arg::Str(src.to_string(), span)
                    },
                    // TODO: check lookups? as they are known at first time
                    Token::Variable => {
                        let src = parser.current_token_source();
                        let span = parser.current_token_span().clone();
                        Arg::Lookup(src.to_string(), span)
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
                            let src = parser.current_token_source().to_string();
                            let span = parser.current_token_span().clone();
                            parser.accept_current();
                            arg = arg.extend_var(src, &span);
                        },
                        Token::Identifier => {
                            let src = parser.current_token_source().to_string();
                            let span = parser.current_token_span().clone();
                            parser.accept_current();
                            arg = arg.extend_str(src, &span, parser.source());
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
