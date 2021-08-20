pub mod ast;
mod conditionals;
mod cycles;
mod exceptions;

use super::println_on_debug;
use ast::*;
use conditionals::Conditionals;
use cycles::Cycles;
use exceptions::Exceptions;

type Token = super::lexer::Token;
type Tokens = Vec<Token>;

pub fn parse(tokens: Vec<super::lexer::Token>) -> AST {
    // create parser object
    let parser: Parser = Parser::new(tokens);
    println_on_debug!("Parser started");

    let abstract_syntax_tree = parser.exec();

    println_on_debug!("Parser ended\n");
    abstract_syntax_tree
}

struct Parser {
    // basic stuff
    tokens: Tokens,
    position: usize,

    // conditionals' stuff
    val1: Option<Literal>,
    oper: Option<CondOperator>,
    val2: Option<Literal>,
    rela: Option<CondRelationship>,

    // cycles stuff
    cycles: Vec<Cycle>,
    funcs: Vec<Function>,
}

impl Parser {
    fn new(tokens: Tokens) -> Self {
        Self {
            tokens,
            position: 0usize,

            val1: None,
            oper: None,
            val2: None,
            rela: None,

            cycles: vec![],
            funcs: vec![],
        }
    }

    fn exec(mut self) -> AST {
        // parse conditionals
        let conditionals: Vec<Conditional> = self.get_conditionals();

        // parse cycles
        let cycles: Vec<Cycle> = self.get_cycles();

        // get AST
        let abstract_syntax_tree = AST {
            conditionals,
            cycles,
        };

        println_on_debug!("  {:#?}", &abstract_syntax_tree);
        abstract_syntax_tree
    }
}
