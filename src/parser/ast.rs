use std::fmt;

#[derive(Debug, Clone)]
pub struct AST {
    pub conditionals: Vec<Conditional>,
    pub cycles: Vec<Cycle>,
}

#[derive(Debug, Clone)]
pub struct Literal {
    pub kind: LiteralKind,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LiteralKind {
    Str, //* String
    Var, //* Variable
    Rgx, //* Regular Expression
    Err, //* ERROR
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub val1: Literal,                                   //* Example: @name
    pub op: CondOperator,                                //* Example: ~=
    pub val2: Literal,                                   //* Example: /project-.*/
    pub next_conditional_relationship: CondRelationship, //* Example: &&
    pub position: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CondOperator {
    Eq, //* == |=> True if the first value matches the second
    Ne, //* != |=> True if the first value doesn't match the second
    Gt, //* >  |=> True if the first value is greater than the second
    Ge, //* >= |=> True if the first value is equal or greater than the second
    Lt, //* <  |=> True if the first value is less than the second
    Le, //* <= |=> True if the first value is equal or less than the second
    Re, //* ~= |=> True if the a value matches the regex provided in the other value
    Rn, //* ~! |=> True if the a value doesn't match the regex provided in the other value
    Er, //* ERROR
}

#[derive(Debug, PartialEq, Clone)]
pub enum CondRelationship {
    And,  //* &&
    Any,  //* ||
    Null, //* NOTHING
    Err,  //* ERROR
}

#[derive(Debug, Clone)]
pub struct Cycle {
    pub operations: Vec<Function>, //* Example: delete(...), print(...)
}

#[derive(Debug, Clone)]
pub struct Function {
    pub function: Func,          //* Example: delete
    pub args: Vec<Vec<Literal>>, //* Example: @path, @parent/../copy/@name
}

#[derive(Debug, PartialEq, Clone)]
pub enum Func {
    DELETE,
    CREATE,
    MKDIR,
    PRINT,
    MOVE,
    COPY,
    SHELL,
    NULL,
}

impl Literal {
    pub fn from_token(token: &super::super::lexer::Token) -> Literal {
        // Create a Literal & return it
        let content = token.content.clone();
        let kind = match token.tok_type.as_str() {
            "Txt" => LiteralKind::Str,
            "Var" => LiteralKind::Var,
            "Rgx" => LiteralKind::Rgx,
            _ => LiteralKind::Err,
        };

        Literal { kind, content }
    }
}

impl CondOperator {
    pub fn from_name(name: &String) -> Self {
        match name.as_str() {
            "Equal" => CondOperator::Eq,
            "NEqual" => CondOperator::Ne,
            "GreaterT" => CondOperator::Gt,
            "GreaterTorE" => CondOperator::Ge,
            "LessT" => CondOperator::Lt,
            "LessTorE" => CondOperator::Le,
            "RgxMatch" => CondOperator::Re,
            "RgxNMatch" => CondOperator::Rn,
            _ => CondOperator::Er,
        }
    }
}

impl CondRelationship {
    pub fn from_name(name: &String) -> Self {
        match name.as_str() {
            "And" => CondRelationship::And,
            "Any" => CondRelationship::Any,
            "Lbrace" => CondRelationship::Null,
            _ => CondRelationship::Err,
        }
    }
}

impl Func {
    pub fn from_name(func_name: String) -> Self {
        match func_name.trim() {
            "delete" => Func::DELETE,
            "create" => Func::CREATE,
            "mkdir" => Func::MKDIR,
            "print" => Func::PRINT,
            "move" => Func::MOVE,
            "copy" => Func::COPY,
            "shell" => Func::SHELL,
            _ => Func::NULL,
        }
    }
}

// custom display formatting: TOKEN_TYPE [ TOKEN_CONTENT ]
impl fmt::Display for Conditional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Conditional [ {} {:?} {} ]", self.val1.content, self.op, self.val2.content)
    }
}