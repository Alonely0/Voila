use super::Target;

#[derive(Debug)]
pub struct Script(Vec<Target>);

use super::parser::*;

impl Parse for Script {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        parser
            .with_context("parsing script", Parser::many_eof)
            .map(Self)
    }
}
