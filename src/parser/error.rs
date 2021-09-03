use std::error::Error;
use std::fmt;
use std::ops::Range;

// TODO: display the span correctly with the markers

// TODO: remove the optional span, and just have a `with_source` on `SourceError`,
// with the start and end `Position`s in the source error struct.

/// Error that is ready to represent to the user in a nice way,
/// including a line snippet with the error position, and a bunch
/// of contexts to help the user know what the program was up to
/// when the error happened.
#[derive(Debug)]
pub struct SourceError<T> {
    pub kind: T,
    snippet: Option<Snippet>,
    span: Option<Range<usize>>,
    contexts: Vec<&'static str>,
}

impl<T> SourceError<T> {
    pub const fn new(kind: T) -> Self {
        Self {
            kind,
            snippet: None,
            span: None,
            contexts: Vec::new(),
        }
    }
    pub fn set_span(&mut self, span: Range<usize>) {
        self.span = Some(span)
    }
    pub fn set_source(&mut self, source: &str) {
        self.snippet = self
            .span
            .as_ref()
            .and_then(|span| Snippet::from_source(span, source))
    }
    pub fn with_span(mut self, span: Range<usize>) -> Self {
        self.set_span(span);
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.set_source(source);
        self
    }

    pub fn with_context(mut self, ctx: &'static str) -> Self {
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
impl<T> Error for SourceError<T>
where
    T: Error + 'static,
{
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.kind)
    }
}

impl<T: fmt::Display> fmt::Display for SourceError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref snippet) = self.snippet {
            write!(
                f,
                "
error: {kind}
    --> {line}:{col}
     |
 {line:3} | {snip}
     | {marker:>col$}",
                kind = self.kind,
                col = snippet.start.col + 1,
                line = snippet.start.line + 1,
                snip = snippet.line,
                marker = '^',
            )
        } else if let Some(ref span) = self.span {
            write!(
                f,
                "error at {span:?}: {kind}",
                span = span,
                kind = self.kind
            )
        } else {
            write!(f, "error: <no position info>: {kind}", kind = self.kind)
        }?;
        for ctx in &self.contexts {
            write!(f, "\n => while {}", ctx)?;
        }
        Ok(())
    }
}
