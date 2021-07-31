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
        return match Some(&*token.tok_type) {
            Some("Txt") => Literal {
                kind: LiteralKind::Str,
                content: token.content.clone(),
            },
            Some("Var") => Literal {
                kind: LiteralKind::Var,
                content: token.content.clone(),
            },
            Some("Rgx") => Literal {
                kind: LiteralKind::Rgx,
                content: token.content.clone(),
            },
            _ => Literal {
                kind: LiteralKind::Err,
                content: token.content.clone(),
            },
        };
    }
}

impl CondOperator {
    pub fn from_name(name: String) -> Self {
        return match Some(&*name) {
            Some("Equal") => CondOperator::Eq,
            Some("NEqual") => CondOperator::Ne,
            Some("GreaterT") => CondOperator::Gt,
            Some("GreaterTorE") => CondOperator::Ge,
            Some("LessT") => CondOperator::Lt,
            Some("LessTorE") => CondOperator::Le,
            Some("RgxMatch") => CondOperator::Re,
            Some("RgxNMatch") => CondOperator::Rn,
            _ => CondOperator::Er,
        };
    }
}

impl CondRelationship {
    pub fn from_name(name: String) -> Self {
        return match Some(&*name) {
            Some("And") => CondRelationship::And,
            Some("Any") => CondRelationship::Any,
            Some("Lbrace") => CondRelationship::Null,
            _ => CondRelationship::Err,
        };
    }
}

impl Func {
    pub fn get_from(func_name: String) -> Self {
        match Some(&*func_name) {
            Some("delete") | Some(" delete") | Some("delete ") => Func::DELETE,
            Some("create") | Some(" create") | Some("create ") => Func::CREATE,
            Some("mkdir") | Some(" mkdir") | Some("mkdir ") => Func::MKDIR,
            Some("print") | Some(" print") | Some("print ") => Func::PRINT,
            Some("move") | Some(" move") | Some("move ") => Func::MOVE,
            Some("copy") | Some(" copy") | Some("copy ") => Func::COPY,
            Some("shell") | Some(" shell") | Some("shell ") => Func::SHELL,
            None | _ => Func::NULL,
        }
    }
}
