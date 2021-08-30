use super::Token;

#[derive(Debug)]
pub enum Expr {
    Value(Value),
    Binary {
        operator: Operator,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
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
pub enum Value {
    Literal(String),
    Regex(regex::Regex),
    Lookup(String),
}

use super::parser::*;

impl Parse for Expr {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        parser.with_context("parsing condition", |parser| {
            parser.parse().map(Expr::Value).and_then(|lhs| {
                parse_expr(parser, lhs, 0).map_err(|e| e.with_context("parsing binary expression"))
            })
        })
    }
}

impl Parse for Value {
    fn parse(parser: &mut Parser) -> ParseRes<Value> {
        parser.with_context("parsing value", |parser| {
            Ok(
                match parser.expect_one_of_tokens(
                    &[Token::Identifier, Token::Variable, Token::Regex],
                    Some("as a value"),
                )? {
                    Token::Identifier => {
                        let src = parser.current_token_source();
                        parser.accept_current();
                        Value::Literal(src.to_string())
                    }
                    Token::Variable => {
                        let src = parser.current_token_source();
                        parser.accept_current();
                        Value::Lookup(src.to_string())
                    }
                    Token::Regex => {
                        let src = parser.current_token_source();

                        let regex = regex::Regex::new(src)
                            .map_err(|err| parser.error(ParseErrorKind::RegexError(err)))?;

                        parser.accept_current();
                        Value::Regex(regex)
                    }
                    _ => unreachable!(),
                },
            )
        })
    }
}

fn parse_expr(parser: &mut Parser, mut lhs: Expr, min_precedence: u8) -> ParseRes<Expr> {
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
        lhs = Expr::Binary {
            operator: op,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        };
    }
    Ok(lhs)
}
