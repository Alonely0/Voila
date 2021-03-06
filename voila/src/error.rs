use ansi_term::{Colour::*, Style};
use std::error::Error;
use std::fmt;
use std::ops::Range;

/// Error that is ready to represent to the user in a nice way,
/// including a line snippet with the error position, and a bunch
/// of contexts to help the user know what the program was up to
/// when the error happened.
#[derive(Debug)]
pub struct SourceError<T, C> {
    pub kind: T,
    snippet: Option<(Range<usize>, Snippet)>,
    contexts: Vec<C>,
}

impl<T, C> SourceError<T, C> {
    pub const fn new(kind: T) -> Self {
        Self {
            kind,
            snippet: None,
            contexts: Vec::new(),
        }
    }
    pub fn set_source(&mut self, span: Range<usize>, source: &str) {
        self.snippet = Snippet::from_source(&span, source).map(|snip| (span, snip));
    }

    pub fn with_source(mut self, span: Range<usize>, source: &str) -> Self {
        self.set_source(span, source);
        self
    }

    pub fn with_context(mut self, ctx: C) -> Self {
        self.contexts.push(ctx);
        self
    }
}

#[derive(Debug)]
struct Snippet {
    start: Position,
    line: String,
}

impl Snippet {
    fn from_source(span: &Range<usize>, source: &str) -> Option<Self> {
        let start_pos = Position::from_offset(span.start, source);
        let end_pos = Position::from_offset(span.end, source);

        if start_pos.line == end_pos.line {
            Some(Self {
                start: start_pos,
                line: source.lines().nth(start_pos.line).unwrap().to_string(),
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub col: usize,
    pub line: usize,
}

impl Position {
    fn from_offset(target_offset: usize, source: &str) -> Self {
        let mut offset = 0;
        let mut line_count = 0;
        for (i, line) in source.split_terminator('\n').enumerate() {
            let line_len = line.len() + 1; // +1 to count for the '\n'
            let next_offset = offset + line_len;
            if next_offset >= target_offset {
                return Self {
                    line: i,
                    col: target_offset - offset,
                };
            }
            offset = next_offset;
            line_count = i;
        }
        Self {
            line: line_count,
            col: 0,
        }
    }
}
impl<T, C> Error for SourceError<T, C>
where
    T: Error + 'static,
    C: fmt::Debug + fmt::Display,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

impl<T: fmt::Display, C: fmt::Display> fmt::Display for SourceError<T, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((span, snippet)) = &self.snippet {
            write!(
                f,
                r#"
{error}: {kind}
 {arrow} {location}
  {separator}
{line:3} {separator}   {snip}
  {separator}   {red}{markers}{end}"#,
                error = Red.bold().paint("error"),
                kind = Style::new().bold().paint(self.kind.to_string()),
                arrow = Blue.bold().paint("-->"),
                location = Yellow.bold().paint(format!(
                    "{line}:{col}",
                    line = snippet.start.line + 1,
                    col = snippet.start.col + 1,
                )),
                line = Blue.bold().paint((snippet.start.line + 1).to_string()),
                separator = Blue.bold().paint("|"),
                snip = &snippet.line,
                red = Red.bold().prefix(),
                end = Red.bold().suffix(),
                markers = " ".repeat(snippet.start.col) + &"^".repeat(span.len())
            )
        } else {
            write!(f, "error:  {kind}", kind = self.kind)
        }?;
        for ctx in &self.contexts {
            write!(
                f,
                "{str}",
                str = Purple.italic().paint(format!("\n => while {ctx}"))
            )?;
        }
        Ok(())
    }
}
