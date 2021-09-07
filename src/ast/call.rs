use super::parser::{Parse, ParseErrorKind, ParseRes, Parser};
use super::HasSpan;
use super::Lookup;
use super::Token;
use std::ops::Range;

/// Represents a call in the script like
/// `shell` or `delete`. All functions receive the arguments in
/// interpolated strings, which will have their variables resolved
/// before execution.
///
/// # List of non-destructive functions
/// These functions are safe to use without any worries that
/// any data will be eliminated:
///     - `print`
///     - `mkdir`
/// `shell` won't be in the list, since you can do absolutely anything
/// when we give you a shell. The shell is an escape hatch to enable the
/// integration of Voila to the rest of the system.
#[derive(Debug)]
pub struct Call<'source> {
    function_kind: Function,
    arguments: Vec<Arg<'source>>,
    span: Range<usize>,
}

/// The function that is [`Call`]ed.
///
/// # Panic
/// The interpreter will panic when the function has not
/// enough arguments to execute.
#[derive(Debug)]
pub enum Function {
    /// Create a directory with its parents recursively.
    /// This function is not destructive, it will error
    /// if a file with the name of the directory exists already.
    ///
    /// # Call format
    /// `mkdir` receives one argument: the path to create
    Mkdir,
    /// Print something to standard output
    ///
    /// # Call format
    /// `print` receives a variadic number of arguments which it prints
    /// separated by spaces (similar to python's print function without parameters), and a newline
    /// after.
    Print,
    /// Execute a command in `sh`
    ///
    /// # Call format
    /// `shell` needs at least one argument. When called, it gets all the arguments, joins all by spaces
    /// and feeds that to `sh -c` in the case of linux and `powershell` on windows. No input is given to it, so things like `sudo`
    /// won't work unless you start voila with privileges
    ///
    /// # Safety
    /// This function may modify the outer system!
    Shell,
    /// Delete the given file
    ///
    /// # Call format
    /// `delete` receives one argument: the file/directory to delete.
    /// Directories are deleted recursively!
    ///
    /// # Safety
    /// `delete` will modify the outer system! Make sure that you're not doing
    /// accesses to the file in the argument on the same cycle, otherwise you will
    /// get undefined behavior.
    Delete,
    /// Moves or renames a file, with a similar behavior to the `mv` command.
    ///
    /// # Call format
    /// `move` receives two arguments: the source file/directory and the target destination
    ///
    /// # Safety
    /// `move` is a destructive call, so please make sure that you're not using it with the same file in the same cycle. Refer to [`Function::Delete`] for details
    Move,
    /// Copy a file or a directory. Directories are copied recursively.
    ///
    /// # Call format
    /// `copy` receives two arguments: the source file/directory and the target destination
    ///
    /// # Safety
    /// `copy` might overwrite files in the system, so use it carefully! Avoid using it in the same
    /// cycle unless you can prove it's safe to do so.
    Copy,
    /// Gzip a file or a directory. Directories are gzipped recursively.
    ///
    // NOTE: please rename this to `gzip` and `gunzip` like the binutils
    /// # Call format
    /// `gzc` receives two arguments: the source file/directory to compress and the file to save it
    /// into. Note that the destination name is not manipulated in any way (nothing is added or
    /// removed to it)
    ///
    /// # Safety
    /// Since `gzc` has an output file. it may overwrite another that's in the system.
    GzipCompress,
    /// Gunzip a file into a file/directory.
    ///
    /// # Call format
    /// `gzd` receives two arguments: the gzipped file, and the destination to decompress into.
    /// The destination, if not specified, is the directory in which the gzipped file is, **not the
    /// one that voila is executing in**
    ///
    /// # Safety
    /// since `gzd` has an output directory, it may overwrite a lot af files! Use with care.
    GzipDecompress,
}

impl Function {
    pub const fn minimum_arg_count(&self) -> u8 {
        match self {
            Self::Copy | Self::Move | Self::GzipCompress | Self::GzipDecompress => 2,
            Self::Delete | Self::Shell | Self::Mkdir => 1,
            Self::Print => 0,
        }
    }
    fn from_name(source: &str) -> Option<Self> {
        Some(match source {
            "copy" => Self::Copy,
            "move" => Self::Move,
            "gzc" => Self::GzipCompress,
            "gzd" => Self::GzipDecompress,
            "delete" => Self::Delete,
            "shell" => Self::Shell,
            "mkdir" => Self::Mkdir,
            "print" => Self::Print,
            _ => return None,
        })
    }
}

impl Parse<'_> for Function {
    fn parse(parser: &mut Parser) -> ParseRes<Self> {
        // this doesn't accept the token because it is accepted by the calling parser (`Call`), so
        // it can use the identifier start as a more accurrate start of the function span.
        let src = parser.expect_token(Token::Identifier, Some("as the name of the function"))?;
        Self::from_name(src).ok_or_else(|| parser.error(ParseErrorKind::UnknownFunction))
    }
}

impl HasSpan for Call<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

/// Represents an argument to the call, with either
/// a single string like `hello world` or an interpolated
/// string with variables in it.
///
/// The argument maintains an invariant: Its sequence is
/// never empty
///
/// # Examples
///
/// - `@name`
/// - `@name is too big`
/// - `hello world`
#[derive(Debug)]
pub struct Arg<'source> {
    sequence: Vec<InterpolateComponent<'source>>,
    span: Range<usize>,
}

#[derive(Debug)]
enum InterpolateComponent<'source> {
    Literal(&'source str),
    Lookup(Lookup),
}

impl HasSpan for Arg<'_> {
    fn span(&self) -> &Range<usize> {
        &self.span
    }
}

// TODO: refactor this into just the interpolated sequence
impl<'source> Arg<'source> {
    /// Construct a literal string argument
    fn str(string: &'source str, span: Range<usize>) -> Self {
        Self {
            sequence: vec![InterpolateComponent::Literal(string)],
            span,
        }
    }
    /// Construct a lookup argument
    fn lookup(lookup: Lookup, span: Range<usize>) -> Self {
        Self {
            sequence: vec![InterpolateComponent::Lookup(lookup)],
            span,
        }
    }
    /// Extend the argument with a string literal, returning the next span
    /// (might be modified)
    fn extend_str(
        &mut self,
        last_span: Range<usize>,
        mut span: Range<usize>,
        source: &'source str,
    ) -> Range<usize> {
        if matches!(
            self.sequence.last().unwrap(),
            InterpolateComponent::Literal(_)
        ) {
            // if the last component was a literal,
            // we can just extend the source
            span.start = last_span.start;

            // UNSAFE: safe. we already unwrapped
            let last_ref = self.sequence.last_mut().unwrap();
            *last_ref = InterpolateComponent::Literal(&source[span.clone()]);
        } else {
            // if the last component was a variable,
            // we will extend the span to accomodate the space in between
            span.start = last_span.end;
            self.sequence
                .push(InterpolateComponent::Literal(&source[span.clone()]));
        }
        span
    }

    /// Extend the argument with a lookup, returning the next span
    fn extend_lookup(
        &mut self,
        lookup: Lookup,
        last_span: Range<usize>,
        span: Range<usize>,
        source: &'source str,
    ) -> Range<usize> {
        if matches!(
            self.sequence.last().unwrap(),
            InterpolateComponent::Literal(_)
        ) {
            // if the last component was a literal, we can
            // extend its source to accomodate the space in between
            let last_span = last_span.start..span.start;
            // UNSAFE: safe, we already unwrapped
            let last_ref = self.sequence.last_mut().unwrap();
            *last_ref = InterpolateComponent::Literal(&source[last_span]);
        } else {
            // otherwise, we will put the spaces as a literal into the sequence
            self.sequence.push(InterpolateComponent::Literal(
                &source[last_span.end..span.start],
            ));
        }

        // now we can safely push the lookup, as we already handled the space in between
        self.sequence.push(InterpolateComponent::Lookup(lookup));

        span
    }
}

impl<'source> Parse<'source> for Call<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.with_context("parsing function call", |parser| {
            let function_kind = parser.parse()?;
            let start = parser.current_token_span().start;
            parser.accept_current();
            parser.expect_token(
                Token::OpenParen,
                Some("to begin the function call arguments"),
            )?;
            parser.accept_current();
            let mut arguments = Vec::new();

            // parsing arguments
            'outer: loop {
                let (mut arg, mut arg_span) = match parser.expect_one_of_tokens(
                    &[Token::CloseParen, Token::Identifier, Token::Variable],
                    Some("argument to the function call or end the call"),
                )? {
                    Token::CloseParen => {
                        break;
                    },
                    Token::Identifier => {
                        let src = parser.current_token_source();
                        let span = parser.current_token_span().clone();
                        (Arg::str(src, span.clone()), span)
                    },
                    // TODO: check lookups? as they are known at first time
                    Token::Variable => {
                        let src = parser.parse()?;
                        let span = parser.current_token_span().clone();
                        (Arg::lookup(src, span.clone()), span)
                    },
                    _ => unreachable!(),
                };
                parser.accept_current();
                loop {
                    match parser.expect_one_of_tokens(
                        &[
                            Token::CloseParen,
                            Token::Identifier,
                            Token::Variable,
                            Token::Comma,
                        ],
                        None,
                    )? {
                        Token::CloseParen => {
                            arguments.push(arg);
                            break 'outer;
                        },
                        Token::Comma => {
                            parser.accept_current();
                            break;
                        },
                        Token::Variable => {
                            let src = parser.parse()?;
                            let span = parser.current_token_span().clone();
                            arg_span = arg.extend_lookup(src, arg_span, span, parser.source());
                            parser.accept_current();
                        },
                        Token::Identifier => {
                            let span = parser.current_token_span().clone();
                            arg_span = arg.extend_str(arg_span, span, parser.source());
                            parser.accept_current();
                        },
                        _ => unreachable!(),
                    }
                }
                arguments.push(arg);
            }
            //     arguments.push(arg);
            // }
            parser.expect_token(Token::CloseParen, Some("to end the argument list"))?;
            let end = parser.current_token_span().end;
            parser.accept_current();

            Ok(Self {
                function_kind,
                arguments,
                span: start..end,
            })
        })
    }
}
