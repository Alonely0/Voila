use crate::ast::*;
use crate::error::SourceError;
use crate::i;
use rayon::prelude::*;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::ops::Range;

type Arg = Vec<StrComponent>;
type Args = Vec<Arg>;

/// Contains I/O operations
#[derive(Debug)]
struct IO {
    created: Option<Args>,
    accessed: Option<Args>,
    modified: Option<Args>,
    metadata: Metadata,
}
#[derive(Debug, Clone)]
enum Metadata {
    Single(Range<usize>),
    Multiple([HashMap<Range<usize>, Range<usize>>; 3]),
}

impl Metadata {
    fn get_multiple(&self) -> [HashMap<Range<usize>, Range<usize>>; 3] {
        match self {
            Self::Multiple(x) => x.to_owned(),
            _ => unreachable!("this should never happen"),
        }
    }
}

impl<'source> From<&'source Range<usize>> for Metadata {
    fn from(r: &Range<usize>) -> Self {
        Self::Single(r.to_owned())
    }
}

impl From<[HashMap<Range<usize>, Range<usize>>; 3]> for Metadata {
    fn from(r: [HashMap<Range<usize>, Range<usize>>; 3]) -> Self {
        Self::Multiple(r)
    }
}

impl From<Metadata> for Range<usize> {
    fn from(r: Metadata) -> Self {
        match r {
            Metadata::Single(x) => x,
            _ => unreachable!("this should never happen"),
        }
    }
}

impl From<Metadata> for [HashMap<Range<usize>, Range<usize>>; 3] {
    fn from(r: Metadata) -> Self {
        match r {
            Metadata::Multiple(x) => x,
            _ => unreachable!("this should never happen"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum SafetyErrorKind {
    Created,
    Accessed,
    Modified,
}

impl Error for SafetyErrorKind {}

impl fmt::Display for SafetyErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("This call ").unwrap();
        match self {
            SafetyErrorKind::Created => f.write_str("creates "),
            SafetyErrorKind::Accessed => f.write_str("accesses "),
            SafetyErrorKind::Modified => f.write_str("modifies "),
        }
        .unwrap();
        f.write_str("a file while another creates, accesses or modifies it at the same time (consider using multiple cycles or targets)")
    }
}

impl IO {
    /// New [IO]
    fn new(
        created: Option<Args>,
        accessed: Option<Args>,
        modified: Option<Args>,
        metadata: Metadata,
    ) -> Self {
        Self {
            created,
            accessed,
            modified,
            metadata,
        }
    }
    /// combines a [Vec<IO>] to a single [IO] maintaining metadata
    fn combine(vector: Vec<Self>) -> Self {
        let mut created = Vec::new().into_iter();
        let mut accessed = Vec::new().into_iter();
        let mut modified = Vec::new().into_iter();
        let mut metadata: [HashMap<Range<usize>, Range<usize>>; 3] =
            [HashMap::new(), HashMap::new(), HashMap::new()];
        for io in vector {
            // Push metadata
            metadata[0].insert(
                created.len()..io.created.clone().unwrap_or_else(Vec::new).len(),
                io.metadata.clone().into(),
            );
            metadata[1].insert(
                accessed.len()..io.accessed.clone().unwrap_or_else(Vec::new).len(),
                io.metadata.clone().into(),
            );
            metadata[2].insert(
                modified.len()..io.modified.clone().unwrap_or_else(Vec::new).len(),
                io.metadata.clone().into(),
            );

            // Chain vectors & push them
            created = created
                .chain(io.created.unwrap_or_else(Vec::new))
                .collect::<Args>()
                .into_iter();
            accessed = accessed
                .chain(io.accessed.unwrap_or_else(Vec::new))
                .collect::<Args>()
                .into_iter();
            modified = modified
                .chain(io.modified.unwrap_or_else(Vec::new))
                .collect::<Args>()
                .into_iter();
        }

        Self::new(
            if created.len() == 0 {
                None
            } else {
                Some(created.collect())
            },
            if accessed.len() == 0 {
                None
            } else {
                Some(accessed.collect())
            },
            if modified.len() == 0 {
                None
            } else {
                Some(modified.collect())
            },
            metadata.into(),
        )
    }
    // Variables that access file content or metadata
    const ACCESS_VARS: &'static [&'static str] = &[
        #[cfg(unix)]
        "ownerID",
        "empty",
        "readonly",
        "elf",
        "txt",
        "content",
        "line",
        "hidden",
        "size",
        "sum",
        "creation",
        "lastChange",
        "lastAccess",
    ];
    /// Search matches through all operations types
    /// of an [IO] and returns vectors representing matches
    fn cross_search_matches(&self) -> [Option<usize>; 3] {
        let s = |v: &Option<Args>, p: &Option<Args>| {
            i!(v)
                .map(|x| {
                    i!(p).position_first(|y| {
                        x == y
                            || y.iter().any(|x| {
                                Self::ACCESS_VARS.iter().any(|&v| {
                                    v == match x {
                                        StrComponent::Lookup(z) => z.as_str(),
                                        _ => "",
                                    }
                                })
                            })
                    }) != None
                })
                .position_first(|x| x)
        };
        [
            s(&self.created, &self.accessed),
            s(&self.accessed, &self.modified),
            s(&self.modified, &self.created),
        ]
    }
    /// Search matches through operation types
    /// of an [IO] and returns vectors representing matches
    fn plain_search_matches(&self) -> [Option<usize>; 3] {
        let s = |v: &Option<Args>| {
            i!(v)
                .map(|x| i!(v).position_first(|y| x == y) != i!(v).position_last(|y| x == y))
                .position_first(|x| x)
        };
        [s(&self.created), None, s(&self.modified)] // cache locks prevent accessing file descriptors at the same time
    }
    /// Get metadata of a specific value in a combined [IO]
    fn get_real_md(
        pos: usize,
        md: &HashMap<Range<usize>, Range<usize>>,
        offset: usize,
    ) -> Range<usize> {
        let mut real = 0..0;
        md.iter()
            .map(|x| {
                if x.1.contains(&(offset + pos)) {
                    real = x.1.to_owned();
                }
            })
            .count();
        real
    }
    /// Check for matches through operations
    fn cross_check_io_ops<T, F: Copy>(&self, err_cb: &F) -> Result<(), T>
    where
        F: FnOnce(usize, SafetyErrorKind) -> T,
    {
        let [created, accessed, modified] = self.cross_search_matches();
        let mut pos = None;
        let mut msg = None;
        if let Some(position) = created {
            pos = Some(position);
            msg = Some(SafetyErrorKind::Created);
        } else if let Some(position) = accessed {
            pos = Some(position);
            msg = Some(SafetyErrorKind::Accessed);
        } else if let Some(position) = modified {
            pos = Some(position);
            msg = Some(SafetyErrorKind::Modified);
        }
        if let (Some(p), Some(m)) = (pos, msg) {
            Err(err_cb(p, m))
        } else {
            Ok(())
        }
    }
    /// Check for matches through every operation
    fn plain_check_io_ops<T, F: Copy>(&self, err_cb: &F) -> Result<(), T>
    where
        F: FnOnce(usize, SafetyErrorKind) -> T,
    {
        let [created, accessed, modified] = self.plain_search_matches();
        if let Some(pos) = created {
            Err(err_cb(pos, SafetyErrorKind::Created))
        } else if let Some(pos) = accessed {
            Err(err_cb(pos, SafetyErrorKind::Accessed))
        } else if let Some(pos) = modified {
            Err(err_cb(pos, SafetyErrorKind::Modified))
        } else {
            Ok(())
        }
    }
    /// a wrapper over plain and cross checks
    fn check_ops<T, F: Copy>(&self, err_cb: F) -> Result<(), T>
    where
        F: FnOnce(usize, SafetyErrorKind) -> T,
    {
        self.cross_check_io_ops(&err_cb)?;
        self.plain_check_io_ops(&err_cb)?;
        Ok(())
    }
}

impl<'source> crate::ast::Script<'source> {
    /// this will be called after getting the AST
    /// and will static-analyze the code and spot
    /// possible undefined-behavior cases, if there
    /// are, it'll prevent voila from running unless
    /// you opt-out of it with `--bypass-all-checks`
    pub fn ub_checks(&self, source: &'source str) -> Result<(), Box<dyn Error>> {
        // for every target
        for target in &self.targets {
            // go through its cycles
            for cycle in &target.cycles {
                // and inspect its calls
                let mut calls = Vec::new();
                for call in &cycle.calls {
                    // ignore functions stated as unsafe
                    if !call.safe {
                        continue;
                    }
                    calls.push(
                        self.action(
                            call.function_kind,
                            call.arguments
                                .iter()
                                .map(|x| x.sequence.clone())
                                .collect::<Vec<Vec<StrComponent>>>(),
                            call.span().into(),
                        ),
                    );
                }
                // get combined IO
                let io = IO::combine(calls);

                // Search through different [IO] operation types
                io.check_ops(|pos, e| {
                    self.raise(
                        e,
                        source,
                        IO::get_real_md(
                            pos,
                            &io.metadata.get_multiple()[0],
                            cycle.calls[0].offset(),
                        ),
                    )
                })?
            }
        }
        Ok(())
    }
    fn action(&self, func: Function, args: Args, metadata: Metadata) -> IO {
        use Function::*;

        let mut created = None;
        let mut accessed = None;
        let mut modified = None;

        match func {
            Mkdir { safe: true } | Create { safe: true } => created = Some(args),
            Print { safe: true } => accessed = Some(args),
            Shell { safe: true } | Child { safe: true } => {
                modified = Some(args);
            },
            Delete { safe: true } => {
                modified = Some(vec![args.get(0).unwrap_or(&Vec::new()).to_vec()])
            },
            Move { safe: true } | GzipDecompress { safe: true } => {
                modified = Some(vec![args.get(0).unwrap_or(&Vec::new()).to_vec()]);
                created = Some(vec![args.get(1).unwrap_or(&Vec::new()).to_vec()]);
            },
            Copy { safe: true } | GzipCompress { safe: true } => {
                accessed = Some(vec![args.get(0).unwrap_or(&Vec::new()).to_vec()]);
                created = Some(vec![args.get(1).unwrap_or(&Vec::new()).to_vec()]);
            },
            _ => unreachable!(),
        }
        IO::new(created, accessed, modified, metadata)
    }
    fn raise(
        &self,
        err: SafetyErrorKind,
        code: &str,
        span: Range<usize>,
    ) -> SourceError<SafetyErrorKind, &'static str> {
        SourceError::new(err)
            .with_source(span, code)
            .with_context("checking possible undefined behavior cases")
            .with_context("checking data races")
    }
}
