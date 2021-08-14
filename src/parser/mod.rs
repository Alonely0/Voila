pub mod ast;
mod conditionals;
mod cycles;
mod exceptions;
mod operations;

use super::println_on_debug;
use ast::*;
use conditionals::Conditionals;
use cycles::Cycles;
use exceptions::Exceptions;
use operations::Operations;
use std::mem;

type Token = super::lexer::Token;
type Tokens = Vec<Token>;

pub fn parse(tokens: Vec<super::lexer::Token>) -> AST {
    // create parser object
    let mut parser: Parser = Parser::new(tokens);
    println_on_debug!("Parser started");

    // parse conditionals
    let get_conditionals = |parser: &mut Parser| -> Vec<Conditional> {
        let mut conditionals: Vec<Conditional> = vec![];
        loop {
            // get conditional and send it to the vector
            let conditional: Conditional = parser.parse_next_conditional();
            conditionals.push(conditional.clone());
            // conditionals will stop when the condr of the next is null,
            // so then we stop
            match conditional.next_conditional_relationship {
                None => break,
                _ => continue,
            }
        }

        println_on_debug!("  Conditionals {:#?}", &conditionals);
        conditionals
    };

    let conditionals: Vec<Conditional> = get_conditionals(&mut parser);

    // parse operations
    let get_cycles = |parser: &mut Parser| -> Vec<Cycle> {
        let cycles: &Vec<Cycle> = parser.parse_operations();
        println_on_debug!("  Cycles {:#?}", &cycles);
        cycles.to_owned()
    };

    let cycles: Vec<Cycle> = get_cycles(&mut parser);
    let abstract_syntax_tree = AST {
        conditionals,
        cycles,
    };

    println_on_debug!("  {:#?}", &abstract_syntax_tree);
    println_on_debug!("Parser ended\n");
    abstract_syntax_tree
}

pub struct Parser {
    // basic stuff
    pub tokens: Tokens,
    pub position: usize,

    // conditionals' stuff
    pub val1: Option<Literal>,
    pub oper: Option<CondOperator>,
    pub val2: Option<Literal>,
    pub rela: Option<CondRelationship>,

    // cycles stuff
    pub cycles: Vec<Cycle>,
    pub raw_cycles: Vec<Vec<Token>>,
    pub current_cycle_funcs: Vec<Function>,
    pub current_function: Option<String>,
    pub current_function_args: Vec<Vec<Literal>>,
    pub parsing_args: bool,
}


impl Parser {
    pub fn new(tokens: Tokens) -> Self {
        Self {
            tokens,
            position: 0usize,

            val1: None,
            oper: None,
            val2: None,
            rela: None,

            cycles: vec![],
            raw_cycles: vec![],
            current_cycle_funcs: vec![],
            current_function: None,
            current_function_args: vec![],
            parsing_args: false,
        }
    }
}
