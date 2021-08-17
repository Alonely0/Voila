extern crate logos;

use super::parser::ast::{Func, Function, Literal};
use super::println_on_debug;
use logos::Logos;
use std::fmt;

pub fn lex(source: &str) -> Vec<Token> {
    println_on_debug!("Lexer started");

    // tokenize source
    let mut tokens = Tokens::lexer(source);

    // empty vector which will contain the struct with the tokens
    let mut parsable_tokens: Vec<Token> = vec![];

    // convert logos' tokens to our own struct, that let us parse
    // the tokens easier and implement our own display formatting
    while let Some(t_type) = tokens.next() {
        let mut t_type_str = format!("{:?}", t_type);
        // prepare token values
        let mut t_value: TokContent = TokContent::new_string(tokens.slice().to_string());
        println!("{}: {}", t_type_str, t_value);
        // Get specific val on some Literals
        match &t_type {
            Tokens::Var(ref val) => {
                t_type_str = String::from("Var");
                t_value = TokContent::new_string(val.to_owned());
            },
            Tokens::Rgx(ref val) => {
                t_type_str = String::from("Rgx");
                t_value = TokContent::new_string(val.to_owned());
            },
            Tokens::Func(ref val) => {
                t_type_str = String::from("Func");
                t_value = TokContent::new_function(val.to_owned());
            },
            Tokens::Error => {
                t_type_str = String::from("Txt");
                t_value = t_value;
            },
            _ => {},
        }

        // create a struct with the token
        let parsable_token = Token::new(t_type_str, t_value);

        println_on_debug!("  {}", &parsable_token);
        parsable_tokens.push(parsable_token);
    }
    println_on_debug!("Lexer ended\n");
    parsable_tokens
}

// Token list
#[derive(Logos, Debug, PartialEq)]
pub enum Tokens {
    // get full function at lex time
    #[regex(r#"( *)(delete|mkdir|print|move|copy|gz[cd]|shell)( *)\(([ @#a-zA-Z0-9\$%^*\-_\+\[\]\\./:&|=!<>~'",]+)\)( *)"#,
    |lex| {
        // split content into function and args
        let mut content: Vec<&str> = lex.slice().split('(').collect();

        // remove last ')'
        let length = content.len(); // needed to make the borrow checker happy
        content[length - 1] = content[length - 1].split(')').collect::<Vec<&str>>()[0];

        let mut arguments = content;
        let func_str = arguments.remove(0);

        // tokenize & parse args
        let mut args_parsed: Vec<Vec<Literal>> = vec![];
        for arg in arguments.join("").as_str().split(',').collect::<Vec<&str>>() {
            // initialize a lexer instance
            let mut lexer = Tokens::lexer(arg);

            // create empty vector for args
            let mut args: Vec<Literal> = vec![];

            // the same as in the full lexer,
            // logos returns None when no tokens left
            while let Some(t_type) = lexer.next() {
                // initialize some variables
                let tok_type: Tokens;
                let mut tok_value: String = lexer.slice().to_string();

                // Error is the coma or other stuff
                // we can safely ignore it because
                // it matched the first regex
                if t_type != Tokens::Txt {
                    if let Tokens::Var(val) = t_type {
                        tok_type = Tokens::Var(val.clone());

                        // set value
                        tok_value = val.to_string();
                    } else {
                        tok_type = Tokens::Txt;
                    }
                } else {
                    tok_type = t_type;
                };

                // push to args
                args.push(
                    Literal::from_token(
                        &Token::new(
                            format!("{:?}", tok_type),
                            TokContent::new_string(tok_value)
                        )
                    ).unwrap()
                );
            }
            // push to supervec of args
            args_parsed.push(args);
        };

        Function {
            function: Func::from_name(func_str.to_string()).unwrap(),
            args: args_parsed
        }
    })]
    Func(Function),
    #[token("==")]
    Equal,
    #[token("!=")]
    NEqual,
    #[token(">")]
    GreaterT,
    #[token(">=")]
    GreaterTorE,
    #[token("<")]
    LessT,
    #[token("<=")]
    LessTorE,
    #[token("~=")]
    RgxMatch,
    #[token("~!")]
    RgxNMatch,
    #[token("||")]
    Any,
    #[token("&&")]
    And,
    #[regex("@([a-z=A-Z1-9]+)",
    |lex| {
        let mut content = lex.slice().chars();
        content.next();
        content.as_str().to_string()
    })]
    Var(String),
    #[regex(r#"#[\-a-zA-Z^\$.*\[\](){}?@!%&*\-_=\+'";:,|\\]+#"#,
    |lex| {
        let mut content = lex.slice().chars();
        content.next();
        content.next_back();
        content.as_str().to_string()
    })]
    Rgx(String),
    #[regex(r#"( *),( *)"#)]
    Comma,
    #[token(";")]
    Semicolon,
    #[token("{")]
    Lbrace,
    #[token("}")]
    Rbrace,
    #[regex(r#"(( )?[a-zA-Z0-9\$%^*\-_\+\[\]\\./:'"]+)([ a-zA-Z0-9\$%^*\-_\+\[\]\\./:';"]+)*"#)]
    Txt,
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

// token struct
#[derive(Clone, Eq, PartialEq)]
pub struct Token {
    pub tok_type: String,
    pub content: TokContent,
}

// it is easier to create tokens like this, isn't it?
impl Token {
    fn new(token: String, value: TokContent) -> Self {
        Self {
            tok_type: token,
            content: value,
        }
    }
}

// custom display formatting: TOKEN_TYPE [ TOKEN_CONTENT ]
impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [ {} ]", self.tok_type, self.content)
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct TokContent {
    pub string: Option<String>,
    pub function: Option<Function>,
}

impl TokContent {
    fn new_string(str: String) -> Self {
        Self {
            string: Some(str),
            function: None,
        }
    }
    fn new_function(func: Function) -> Self {
        Self {
            string: None,
            function: Some(func),
        }
    }
}

impl fmt::Display for TokContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref string) = self.string {
            write!(f, "{}", string)
        } else if let Some(ref function) = self.function {
            write!(f, "{:?}", function)
        } else {
            panic!()
        }
    }
}
