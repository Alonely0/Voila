use super::error::SourceError;
use super::lexer::Token;
use std::error::Error;
use std::fmt;
use std::ops::Range;

pub type ParseError = SourceError<ParseErrorKind>;
pub type ParseRes<T> = Result<T, ParseError>;

#[derive(Debug)]
pub enum ParseErrorKind {
    UnexpectedChar(char),
    Expected {
        wanted: WantedSpec<Token>,
        found: Token,
    },
    UnexpectedEOF {
        wanted: Option<WantedSpec<Token>>,
    },
    RegexError(regex::Error),
}

impl ParseErrorKind {
    const fn is_critical(&self) -> bool {
        matches!(self, Self::UnexpectedChar(_))
    }
}

#[derive(Debug, Clone)]
pub struct WantedSpec<T> {
    explicit: Vec<T>,
    explanation: Option<&'static str>,
}

impl<T> WantedSpec<T> {
    #[allow(dead_code)]
    pub const fn new() -> Self {
        Self {
            explicit: Vec::new(),
            explanation: None,
        }
    }
    pub fn set_explanation(&mut self, explanation: &'static str) {
        self.explanation = Some(explanation)
    }
    pub fn with_explanation(mut self, explanation: &'static str) -> Self {
        self.set_explanation(explanation);
        self
    }
    pub fn explicit_single(single: T) -> Self {
        Self {
            explicit: vec![single],
            explanation: None,
        }
    }
    pub fn explicit_multiple(multiple: Vec<T>) -> Self {
        Self {
            explicit: multiple,
            explanation: None,
        }
    }
    #[allow(dead_code)]
    pub fn explanation(explanation: &'static str) -> Self {
        Self::new().with_explanation(explanation)
    }
}

pub trait Parse<'source>: Sized {
    fn from_source(src: &'source str) -> ParseRes<Self> {
        let mut parser = Parser::new(src);
        parser.parse()
    }
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self>;
}

pub struct Parser<'source> {
    input: &'source str,
    lexer: logos::Lexer<'source, Token>,
    current: Option<(Token, Range<usize>)>,
}

impl<'source> Parser<'source> {
    pub fn new(input: &'source str) -> Self {
        use logos::Logos;
        Self {
            lexer: Token::lexer(input),
            input,
            current: None,
        }
    }
    pub fn error(&self, kind: ParseErrorKind) -> ParseError {
        ParseError::new(kind)
            .with_span(self.lexer.span())
            .with_source(self.input)
    }
    pub fn current_token(&mut self) -> ParseRes<Option<Token>> {
        if let Some((tok, _)) = self.current {
            Ok(Some(tok))
        } else {
            Ok(
                if let Some(next) = match self.lexer.next() {
                    Some(Token::Unidentified) => Err(self.error(ParseErrorKind::UnexpectedChar(
                        self.input[self.lexer.span()].chars().next().unwrap(),
                    ))),
                    opt => Ok(opt),
                }? {
                    let span = self.lexer.span();
                    self.current = Some((next, span));
                    Some(next)
                } else {
                    None
                },
            )
        }
    }
    pub fn offset(&self) -> usize {
        self.lexer.span().end
    }
    pub fn current_token_source(&self) -> &'source str {
        let range = self
            .current
            .as_ref()
            .expect("called current_token with no token")
            .1
            .clone();
        &self.input[range]
    }
    pub fn current_token_span(&self) -> &Range<usize> {
        &self
            .current
            .as_ref()
            .expect("called current_token_span with no token")
            .1
    }
    pub fn expect_any_token(&mut self, wanted: Option<WantedSpec<Token>>) -> ParseRes<Token> {
        self.current_token()?
            .ok_or_else(|| self.error(ParseErrorKind::UnexpectedEOF { wanted }))
    }
    /// Expect a token from a list
    pub fn expect_one_of_tokens(
        &mut self,
        options: &[Token],
        explanation: Option<&'static str>,
    ) -> ParseRes<Token> {
        debug_assert_ne!(options.len(), 0, "no options to choose from");
        let wanted = {
            let w = WantedSpec::explicit_multiple(options.iter().copied().collect());
            if let Some(expl) = explanation {
                w.with_explanation(expl)
            } else {
                w
            }
        };
        let tok = self.expect_any_token(Some(wanted.clone()))?;
        for x in options {
            if x == &tok {
                return Ok(tok);
            }
        }
        Err(self.error(ParseErrorKind::Expected { wanted, found: tok }))
    }

    pub fn expect_token(
        &mut self,
        tok: Token,
        explanation: Option<&'static str>,
    ) -> ParseRes<&'source str> {
        let wanted = {
            let w = WantedSpec::explicit_single(tok);
            if let Some(expl) = explanation {
                w.with_explanation(expl)
            } else {
                w
            }
        };
        let found = self.expect_any_token(Some(wanted.clone()))?;
        if tok != found {
            Err(self.error(ParseErrorKind::Expected { wanted, found }))
        } else {
            Ok(self.current_token_source())
        }
    }

    pub fn accept_current(&mut self) {
        self.current = None
    }

    /// Alternative to [Parse::parse]
    pub fn parse<P: Parse<'source>>(&mut self) -> ParseRes<P> {
        P::parse(self)
    }

    /// Iterate the parser as many times as it can,
    /// only resulting in error if the error is critical
    pub fn many<P: Parse<'source>>(&mut self) -> ParseRes<Vec<P>> {
        self.repeat(Self::parse)
    }

    /// Run the closure and then accept the current token.
    /// Useful to get stuff like the span and source
    pub fn accept_after<F, T>(&mut self, mut closure: F) -> T
    where
        F: FnMut(&Self) -> T,
    {
        let result = closure(self);
        self.accept_current();
        result
    }

    pub fn sep_by_token<F, P>(&mut self, tok: Token, mut parser: F) -> ParseRes<Vec<P>>
    where
        F: FnMut(&mut Self) -> ParseRes<P>,
    {
        let mut vec = match parser(self) {
            Err(e) => return Err(e),
            Ok(first) => vec![first],
        };
        while self.current_token()?.filter(|t| t == &tok).is_some() {
            self.accept_current();
            vec.push(parser(self)?);
        }
        Ok(vec)
    }

    pub fn repeat<F, P>(&mut self, mut parser: F) -> ParseRes<Vec<P>>
    where
        F: FnMut(&mut Self) -> ParseRes<P>,
    {
        let mut vec = Vec::new();
        while self.current_token()?.is_some() {
            vec.push(parser(self)?);
        }
        Ok(vec)
    }

    /// Repeats the same parser until a specific token is matched,
    /// without accepting it.
    pub fn repeat_until_token<F, P>(&mut self, tok: Token, mut parser: F) -> ParseRes<Vec<P>>
    where
        F: FnMut(&mut Self) -> ParseRes<P>,
    {
        let mut vec = Vec::new();
        while self.current_token()?.filter(|t| t != &tok).is_some() {
            vec.push(parser(self)?);
        }
        Ok(vec)
    }

    pub fn repeat_sep<F, P>(&mut self, sep: Token, end: Token, mut parser: F) -> ParseRes<Vec<P>>
    where
        F: FnMut(&mut Self) -> ParseRes<P>,
    {
        let mut vec = if self.current_token()?.filter(|t| t == &end).is_some() {
            return Ok(Vec::new());
        } else {
            vec![parser(self)?]
        };
        while self
            .current_token()?
            .filter(|t| t != &end && t == &sep)
            .is_some()
        {
            self.accept_current();
            vec.push(parser(self)?);
        }
        Ok(vec)
    }

    pub fn many_eof<P: Parse<'source>>(&mut self) -> ParseRes<Vec<P>> {
        let mut vec = Vec::new();
        #[allow(unused_assignments)]
        let mut last_err = None;
        loop {
            match self.parse::<P>() {
                Err(e) if e.kind.is_critical() => return Err(e),
                Err(e) => {
                    last_err = Some(e);
                    break;
                },
                Ok(p) => vec.push(p),
            }
        }
        if let Some(err) = self.lexer.next().and(last_err) {
            Err(err)
        } else {
            Ok(vec)
        }
    }

    pub fn with_context<F, T>(&mut self, ctx: &'static str, mut cont: F) -> ParseRes<T>
    where
        F: FnMut(&mut Self) -> ParseRes<T>,
    {
        cont(self).map_err(|e| e.with_context(ctx))
        // eprintln!("entering context: {}", ctx);
        // let res = cont(self).map_err(|e| e.with_context(ctx))?;
        // eprintln!("exiting context: {}", ctx);
        // Ok(res)
    }

    pub fn source(&self) -> &'source str {
        self.input
    }
}

impl Error for ParseErrorKind {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let Self::RegexError(re) = self {
            Some(re)
        } else {
            None
        }
    }
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::RegexError(re) => write!(f, "could not parse regex: {}", re),
            Self::Expected { wanted, found } => {
                write!(f, "expected {}, but got instead {}", wanted, found)
            },
            Self::UnexpectedChar(ch) => write!(f, "unexpected char: {:?}", ch),
            Self::UnexpectedEOF {
                wanted: Some(ref wanted),
            } => write!(f, "unexpected EOF, wanted {}", wanted),
            Self::UnexpectedEOF { wanted: None } => write!(f, "unexpected EOF"),
        }
    }
}
impl<T: fmt::Display> fmt::Display for WantedSpec<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.explicit.len() {
            0 => Ok(()),
            1 => self.explicit[0].fmt(f),
            _ => {
                let first = &self.explicit[0];
                let last = &self.explicit[self.explicit.len() - 1];
                first.fmt(f)?;
                for x in &self.explicit[1..self.explicit.len() - 1] {
                    f.write_str(", ")?;
                    x.fmt(f)?;
                }
                f.write_str(" or ")?;
                last.fmt(f)
            },
        }?;
        if let Some(expl) = self.explanation {
            write!(f, " ({})", expl)
        } else {
            Ok(())
        }
    }
}
