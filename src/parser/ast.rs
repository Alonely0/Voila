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
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub val1: Literal,                                           //* Example: @name
    pub op: CondOperator,                                        //* Example: ~=
    pub val2: Literal,                                           //* Example: /project-.*/
    pub next_conditional_relationship: Option<CondRelationship>, //* Example: &&
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
}

#[derive(Debug, PartialEq, Clone)]
pub enum CondRelationship {
    And, //* &&
    Any, //* ||
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
    GZC,
    GZD,
    SHELL,
}

impl Literal {
    pub fn from_token(token: &super::super::lexer::Token) -> Result<Self, String> {
        // Create a Literal & return it
        let content = token.content.to_owned();
        match token.tok_type.as_str() {
            "Txt" => Ok(Self {
                content,
                kind: LiteralKind::Str,
            }),
            "Var" => Ok(Self {
                content,
                kind: LiteralKind::Var,
            }),
            "Rgx" => Ok(Self {
                content,
                kind: LiteralKind::Rgx,
            }),
            _ => Err(content),
        }
    }
}

impl CondOperator {
    pub fn from_name(name: &str) -> Result<Self, ()> {
        match name {
            "Equal" => Ok(CondOperator::Eq),
            "NEqual" => Ok(CondOperator::Ne),
            "GreaterT" => Ok(CondOperator::Gt),
            "GreaterTorE" => Ok(CondOperator::Ge),
            "LessT" => Ok(CondOperator::Lt),
            "LessTorE" => Ok(CondOperator::Le),
            "RgxMatch" => Ok(CondOperator::Re),
            "RgxNMatch" => Ok(CondOperator::Rn),
            _ => Err(()),
        }
    }
}

impl CondRelationship {
    pub fn from_name(name: &str) -> Result<Option<Self>, ()> {
        match name {
            "And" => Ok(Some(CondRelationship::And)),
            "Any" => Ok(Some(CondRelationship::Any)),
            "Lbrace" => Ok(None),
            _ => Err(()),
        }
    }
}

impl Func {
    pub fn from_name(func_name: String) -> Result<Self, ()> {
        match func_name.trim() {
            "delete" => Ok(Func::DELETE),
            "create" => Ok(Func::CREATE),
            "mkdir" => Ok(Func::MKDIR),
            "print" => Ok(Func::PRINT),
            "move" => Ok(Func::MOVE),
            "copy" => Ok(Func::COPY),
            "gzc" => Ok(Func::GZC),
            "gzd" => Ok(Func::GZD),
            "shell" => Ok(Func::SHELL),
            _ => Err(()),
        }
    }
}

// custom display formatting: TOKEN_TYPE [ TOKEN_CONTENT ]
impl fmt::Display for Conditional {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Conditional [ {} {:?} {} ]",
            self.val1.content, self.op, self.val2.content
        )
    }
}
