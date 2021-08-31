use super::Target;

// The script doesn't have a span, since it represents the **entire** script.
#[derive(Debug)]
pub struct Script<'source>(Vec<Target<'source>>);

use super::parser::*;

impl<'source> Parse<'source> for Script<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser
            .with_context("parsing script", Parser::many_eof)
            .map(Self)
    }
}
