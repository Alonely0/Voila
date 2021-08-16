extern crate logos;

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
        // prepare token values
        let mut t_value = tokens.slice().to_string();

        // remove unnecessary symbols on some Literals
        match t_type {
            Tokens::Var => {
                let mut t_value_chars = t_value.chars();
                t_value_chars.next();
                t_value = t_value_chars.as_str().to_string();
            },
            Tokens::Rgx => {
                let mut t_value_chars = t_value.chars();
                t_value_chars.next();
                t_value_chars.next_back();
                t_value = t_value_chars.as_str().to_string();
            },
            _ => {},
        }

        // create a struct with the token
        let parsable_token = Token::new(format!("{:?}", t_type), t_value);

        println_on_debug!("  {}", &parsable_token);
        parsable_tokens.push(parsable_token);
    }
    println_on_debug!("Lexer ended\n");
    parsable_tokens
}

// Token list
#[derive(Logos, Debug, PartialEq)]
pub enum Tokens {
    #[regex(r#"( )*(delete)( )*"#)]
    #[regex(r#"( )*(create)( )*"#)]
    #[regex(r#"( )*(mkdir)( )*"#)]
    #[regex(r#"( )*(print)( )*"#)]
    #[regex(r#"( )*(move)( )*"#)]
    #[regex(r#"( )*(copy)( )*"#)]
    #[regex(r#"( )*(gz[cd])( )*"#)]
    #[regex(r#"( )*(shell)( )*"#)]
    Func,
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
    #[regex("@([a-z=A-Z1-9]+)")]
    Var,
    #[regex(r#"#[\-a-zA-Z^\$.*\[\](){}?@!%&*\-_=\+'";:,|\\]+#"#)]
    Rgx,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token("(")]
    Lparen,
    #[token(")")]
    Rparen,
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub tok_type: String,
    pub content: String,
}

// it is easier to create tokens like this, isn't it?
impl Token {
    fn new(token: String, value: String) -> Self {
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
