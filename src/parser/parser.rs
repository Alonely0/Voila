pub struct Parser {
    // basic stuff
    pub tokens: super::Tokens,
    pub position: usize,

    // conditionals' stuff
    pub val1: Option<super::Literal>,
    pub oper: Option<super::CondOperator>,
    pub val2: Option<super::Literal>,
    pub rela: Option<super::CondRelationship>,

    // cycles stuff
    pub cycles: Vec<super::Cycle>,
    pub raw_cycles: Vec<Vec<super::Token>>,
    pub current_cycle_funcs: Vec<super::Function>,
    pub current_function: String,
    pub current_function_args: Vec<Vec<super::Literal>>,
    pub parsing_args: bool,
}
