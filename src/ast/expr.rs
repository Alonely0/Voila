use super::HasSpan;
use super::Str;
use super::Token;
use std::ops::Range;

// TODO: update `Expr` docs when static analyzer is brought into life

/// The conditional to filter when a block will be executed.
///
/// They are composed by [values](Value) and [operators](Operator). The expression ends up in a `bool`,
/// which determines whether the block will be eecuted or not.
///
///
/// # Panics: Coherence
/// Voila doesn't have yet any way to check that the comparisons make sense before going and
/// executing them, so it will panic whenever it finds a one that is ill-constructed.
///
/// Because of this, the type rules are quite relaxed. But be careful with pattern matches and
/// relational comparisons, because there is no other reasonable way to relax those rules without
/// breaking the consistency of the operator.
///
/// # Examples
/// ```voila
/// @size=mb > 1.23 && @txt { ... }
/// ```
/// ```voila
/// @sha256sum ~= #.*e0.*# { ... }
/// ```
#[derive(Debug)]
pub enum Expr<'source> {
    Value(Str<'source>),
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

/// The operators than help build [Expr]s
/// # Supported operators
/// The currently supported operators are:
/// - Equality operators: `==` and `!=`
/// Currently the comparison is string-based, although that might change it the future.
/// - Relative operators: `>=`, `<=`, `>` and `<`
/// These comparisons are number-based on both sides. The numbers can be integers
/// or decimal numbers, which will be cut to a precision of 2 digits.
/// - Pattern match operators: `~=` and `!~`
/// The left hand side will always be converted to a string, and the right hand side
/// must be a valid regex.
/// - Logic operators: `&&` and `||`
/// Both sides must result in a bool. Anything that is not a bool (regex, string, variable) will
/// become true for the moment. There are plans to forbid this in the future with a static
/// analyzer.
///
#[derive(Debug)]
pub enum Operator {
    /// `!=`: True if the two sides are strictly not equal.
    NEquals,
    /// `==`: True if the two sides are strictly equal.
    Equals,
    /// `~=`: True if the string matches the regex
    Matches,
    /// `!~`: True if the string doesn't match the regex
    NMatches,
    /// `&&`: True if both sides are true.
    LogicAnd,
    /// `||`: True if either of the sides is true.
    LogicOr,
    /// `<` : True if the left hand side is strictly less than the right hand side
    LessThan,
    /// `<=`: True if the left hand side is less than, or equal to, the right hand side
    LessEqual,
    /// `>` : True if the left hand side is strictly greater than the right hand side
    GreaterThan,
    /// `>=`: True if the left hand side is greater than, or equal to, the right hand side
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

use super::parser::*;

impl<'source> Parse<'source> for Expr<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context(ContextLevel::Condition, |parser| {
            parser
                .parse()
                .map(Expr::Value)
                .and_then(|lhs| parse_expr(parser, lhs, 0))
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

use crate::interpreter::{Cache, ErrorKind, ExprResult, Resolve};
impl Resolve for Expr<'_> {
    fn resolve(&self, cache: &mut Cache) -> Result<ExprResult, ErrorKind> {
        match self {
            Self::Value(v) => cache.resolve(v),
            Self::Binary {
                operator, lhs, rhs, ..
            } => {
                let lhs = cache.resolve(lhs.as_ref())?;
                let rhs = cache.resolve(rhs.as_ref())?;
                Ok(ExprResult::from(match operator {
                    Operator::Equals => lhs.cast_to_string() == rhs.cast_to_string(),
                    Operator::GreaterEqual => lhs.cast_to_number()? >= rhs.cast_to_number()?,
                    Operator::GreaterThan => lhs.cast_to_number()? > rhs.cast_to_number()?,
                    Operator::LessEqual => lhs.cast_to_number()? <= rhs.cast_to_number()?,
                    Operator::LessThan => lhs.cast_to_number()? < rhs.cast_to_number()?,
                    Operator::NEquals => lhs.cast_to_string() != rhs.cast_to_string(),
                    // regex will be always on the right...?
                    // NOTE: now that I think about it, it should be on the left, right?
                    // like <pattern> matches <string>. I'll keep it consistent with how
                    // it's documented thaugh, and when we change the behavior we should
                    // update the docs. TODO: Think about the places in the matches and
                    // update the description on the README + operator enum before changing
                    // the behavior code.
                    Operator::Matches => rhs.cast_to_regex()?.is_match(&lhs.cast_to_string()),
                    Operator::NMatches => rhs.cast_to_regex()?.is_match(&lhs.cast_to_string()),
                    // note: using the single ones so a shortcut is not generated,
                    // and the casts are made first. This won't be relevant when types
                    // are validated prior to runtime.
                    Operator::LogicAnd => lhs.cast_to_bool()? & rhs.cast_to_bool()?,
                    Operator::LogicOr => lhs.cast_to_bool()? | rhs.cast_to_bool()?,
                }))
            },
        }
    }
}
