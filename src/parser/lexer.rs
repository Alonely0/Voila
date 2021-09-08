use logos::Logos;
use std::fmt;

#[derive(Debug, Logos, Clone, Copy, PartialEq, Eq)]
pub enum Token {
    #[regex(r#"#[\-a-zA-Z^\$.*\[\](){}?@!%&*\-_=\+'";:,|\\]+#"#)]
    Regex,

    #[regex(r"@[A-Za-z0-9]+(?:=[A-Za-z0-9]+)?")]
    Variable,

    #[regex(r"[^@{}(),\s;]+")]
    Identifier,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    // operators
    #[token("==")]
    Equals,

    #[token("!=")]
    NEquals,

    #[token("~=")]
    Match,

    #[token("!~")]
    NMatch,

    #[token(">=")]
    GEq,

    #[token(">")]
    GThan,

    #[token("<")]
    LThan,

    #[token("<=")]
    LEq,

    #[token("&&")]
    LogicAnd,

    #[token("||")]
    LogicOr,

    #[regex(r"[ \t\n\f]", logos::skip)]
    #[error]
    Unidentified,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::CloseBrace => write!(f, "closing brace `}}`"),
            Self::CloseParen => write!(f, "closing paren `)`"),
            Self::Comma => write!(f, "comma `,`"),
            Self::Equals => write!(f, "operator `==`"),
            Self::GEq => write!(f, "operator `>=`"),
            Self::GThan => write!(f, "operator `>`"),
            Self::LEq => write!(f, "operator `<=`"),
            Self::LThan => write!(f, "operator `<`"),
            Self::Identifier => write!(f, "identifier"),
            Self::LogicAnd => write!(f, "operator `&&`"),
            Self::LogicOr => write!(f, "operator `||`"),
            Self::Match => write!(f, "match operator `~=`"),
            Self::NMatch => write!(f, "operator `~!`"),
            Self::NEquals => write!(f, "operator `==`"),
            Self::OpenBrace => write!(f, "opening brace `{{`"),
            Self::OpenParen => write!(f, "opening paren `)`"),
            Self::Variable => write!(f, "variable"),
            Self::Regex => write!(f, "regular expression"),
            Self::Semicolon => write!(f, "semicolon `;`"),
            Self::Unidentified => unreachable!(),
        }
    }
}
