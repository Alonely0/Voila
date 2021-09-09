use super::HasSpan;
use super::Lookup;
use crate::parser::{ContextLevel, Parse, ParseErrorKind, ParseRes, Parser, Token};
use std::ops::Range;

/// An interpolated string, which supports spaces in between and variables:
/// `@name.file is broken` will resolve to "<name of the file> is broken" on each
/// instance.
///
/// The interpolated string maintains an invariant: its sequence is never empty
#[derive(Debug)]
pub struct Str<'source> {
    sequence: Vec<StrComponent<'source>>,
    span: Range<usize>,
}

impl HasSpan for Str<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

#[derive(Debug)]
enum StrComponent<'source> {
    Literal(&'source str),
    Lookup(Lookup),
}

impl<'source> Str<'source> {
    fn new(first_component: StrComponent<'source>, component_span: Range<usize>) -> Self {
        Self {
            sequence: vec![first_component],
            span: component_span,
        }
    }
    /// Extend the interpolation with a component, returning the new component span (might be
    /// modified)
    fn extend(
        &mut self,
        component: StrComponent<'source>,
        last_component_span: Range<usize>,
        mut component_span: Range<usize>,
        full_input: &'source str,
    ) -> Range<usize> {
        match component {
            StrComponent::Literal(_) => {
                if matches!(self.sequence.last().unwrap(), StrComponent::Literal(_)) {
                    // if the last component was a literal,
                    // we can just extend the source.
                    component_span.start = last_component_span.start;
                    let last_ref = self.sequence.last_mut().unwrap();
                    *last_ref = StrComponent::Literal(&full_input[component_span.clone()]);
                } else {
                    // if the last component wasa variable, we will extend the span to accomodate
                    // the space in between
                    component_span.start = last_component_span.end;
                    self.sequence
                        .push(StrComponent::Literal(&full_input[component_span.clone()]));
                }
            },
            StrComponent::Lookup(_) => {
                if matches!(self.sequence.last().unwrap(), StrComponent::Literal(_)) {
                    // if the last component was a literal, we can extend its source to accomodate
                    // the spece in between
                    let last_component_span = last_component_span.start..component_span.start;
                    let last_ref = self.sequence.last_mut().unwrap();
                    *last_ref = StrComponent::Literal(&full_input[last_component_span]);
                } else {
                    // otherwise, we will put the spaces as a literal into the sequence
                    self.sequence.push(StrComponent::Literal(
                        &full_input[last_component_span.end..component_span.start],
                    ));
                }
                // now we can safely push the lookup, since we already handled the space before it
                self.sequence.push(component);
            },
        }
        component_span
    }
}

// this parser is more of a helper than anything, to avoid repeating the matching code
// for unknown variables and so.
impl<'source> Parse<'source> for (StrComponent<'source>, Range<usize>) {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        let res = match parser.current_token()?.unwrap() {
            Token::Variable => {
                let span = parser.current_token_span().clone();
                (
                    match parser.parse() {
                        Ok(lookup) => StrComponent::Lookup(lookup),
                        // unknown variables become literals with the at they came with
                        Err(e) if matches!(e.kind, ParseErrorKind::UnknownVariable) => {
                            StrComponent::Literal(parser.current_token_source())
                        },
                        // other errors are not caught though
                        Err(e) => return Err(e),
                    },
                    span,
                )
            },
            Token::Identifier => (
                StrComponent::Literal(parser.current_token_source()),
                parser.current_token_span().clone(),
            ),
            _ => unreachable!("The main str parser should have stopped on these already."),
        };
        parser.accept_current();
        Ok(res)
    }
}

impl<'source> Parse<'source> for Str<'source> {
    // this parser gets all the variables and identifiers that it can and
    // mashes them up into a Str.
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context(ContextLevel::InterpSeq, |parser| {
            let (mut str, mut last_span) =
                {
                    parser.expect_one_of_tokens(
                &[Token::Variable, Token::Identifier],
                Some("interpolated strings need at least one variable or string without spaces"),
            )?;
                    let (component, span) = parser.parse()?;
                    (Self::new(component, span.clone()), span)
                };
            while parser
                .current_token()?
                .filter(|tok| matches!(tok, Token::Variable | Token::Identifier))
                .is_some()
            {
                let (next_component, next_span) = parser.parse()?;
                last_span = str.extend(next_component, last_span, next_span, parser.source());
            }
            Ok(str)
        })
    }
}

use crate::interpreter;

impl interpreter::Resolve for Str<'_> {
    fn resolve(
        &self,
        cache: &mut interpreter::Cache,
    ) -> Result<interpreter::ExprResult, interpreter::ErrorKind> {
        let mut str = String::new();
        for x in &self.sequence {
            match x {
                StrComponent::Literal(lit) => str.push_str(lit),
                StrComponent::Lookup(lookup) => {
                    str.push_str(&cache.resolve(lookup)?.cast_to_string())
                },
            }
        }
        Ok(str.into())
    }
}
