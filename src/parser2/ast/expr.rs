use super::HasSpan;
use super::Token;
use std::ops::Range;

#[derive(Debug)]
pub enum Expr<'source> {
    Value(Value<'source>),
    Binary {
        operator: Operator,
        lhs: Box<Expr<'source>>,
        rhs: Box<Expr<'source>>,
        span: Range<usize>,
    },
}

impl HasSpan for Expr<'_> {
    fn span(&self) -> &Range<usize> {
        match self {
            Self::Value(val) => val.span(),
            Self::Binary { span, .. } => span,
        }
    }
}

#[derive(Debug)]
pub enum Operator {
    NEquals,
    Equals,
    Matches,
    NMatches,
    LogicAnd,
    LogicOr,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
}

impl Operator {
    const fn precedence(&self) -> u8 {
        match self {
            Self::LogicOr => 0,
            Self::LogicAnd => 1,
            Self::NEquals
            | Self::NMatches
            | Self::Matches
            | Self::Equals
            | Self::LessEqual
            | Self::LessThan
            | Self::GreaterEqual
            | Self::GreaterThan => 2,
        }
    }
    fn from_token(tok: Token) -> Option<Self> {
        Some(match tok {
            Token::NEquals => Self::NEquals,
            Token::NMatch => Self::NMatches,
            Token::Equals => Self::Equals,
            Token::Match => Self::Matches,
            Token::LogicAnd => Self::LogicAnd,
            Token::LogicOr => Self::LogicOr,
            Token::LThan => Self::LessThan,
            Token::LEq => Self::LessEqual,
            Token::GThan => Self::GreaterThan,
            Token::GEq => Self::GreaterEqual,
            _ => return None,
        })
    }
}
#[derive(Debug)]
pub enum Value<'source> {
    Literal(&'source str, Range<usize>),
    Regex(regex::Regex, Range<usize>),
    Lookup(&'source str, Range<usize>),
}

impl HasSpan for Value<'_> {
    fn span(&self) -> &Range<usize> {
        match self {
            Self::Literal(_, span) | Self::Regex(_, span) | Self::Lookup(_, span) => span,
        }
    }
}

use super::parser::*;

impl<'source> Parse<'source> for Expr<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context("parsing condition", |parser| {
            parser.parse().map(Expr::Value).and_then(|lhs| {
                parse_expr(parser, lhs, 0).map_err(|e| e.with_context("parsing binary expression"))
            })
        })
    }
}

impl<'source> Parse<'source> for Value<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context("parsing value", |parser| {
            Ok(
                match parser.expect_one_of_tokens(
                    &[Token::Identifier, Token::Variable, Token::Regex],
                    Some("as a value"),
                )? {
                    Token::Identifier => parser.accept_after(|parser| {
                        Value::Literal(
                            parser.current_token_source(),
                            parser.current_token_span().clone(),
                        )
                    }),
                    Token::Variable => parser.accept_after(|parser| {
                        Value::Lookup(
                            parser.current_token_source(),
                            parser.current_token_span().clone(),
                        )
                    }),
                    Token::Regex => {
                        let src = parser.current_token_source();

                        let regex = regex::Regex::new(src)
                            .map_err(|err| parser.error(ParseErrorKind::RegexError(err)))?;

                        let value = Value::Regex(regex, parser.current_token_span().clone());
                        parser.accept_current();
                        value
                    },
                    _ => unreachable!(),
                },
            )
        })
    }
}

fn parse_expr<'source>(
    parser: &mut Parser<'source>,
    mut lhs: Expr<'source>,
    min_precedence: u8,
) -> ParseRes<Expr<'source>> {
    while let Some(op) = parser
        .current_token()?
        .and_then(Operator::from_token)
        .filter(|x| x.precedence() >= min_precedence)
    {
        parser.accept_current();
        let mut rhs = parser.parse().map(Expr::Value)?;
        while parser
            .current_token()?
            .and_then(Operator::from_token)
            .filter(|op| op.precedence() > op.precedence())
            .is_some()
        {
            rhs = parse_expr(parser, rhs, min_precedence + 1)?;
        }
        let rhs_end = rhs.span().end;
        lhs = Expr::Binary {
            span: lhs.span().start..rhs_end,
            operator: op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }
    Ok(lhs)
}
