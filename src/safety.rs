use crate::ast::*;
use crate::i;
use rayon::prelude::*;
use std::collections::HashMap;
use std::ops::Range;

type Arg<'source> = Vec<StrComponent<'source>>;
type Args<'source> = Vec<Arg<'source>>;

/// Contains I/O operations
#[derive(Debug)]
struct IO<'source> {
    created: Option<Args<'source>>,
    accessed: Option<Args<'source>>,
    modified: Option<Args<'source>>,
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

//trait Metadata<T: Sized, O: Sized>: Any + Sized + IntoIterator<Item = T, IntoIter = O> {}
impl<'source> IO<'source> {
    /// New [IO]
    fn new(
        created: Option<Args<'source>>,
        accessed: Option<Args<'source>>,
        modified: Option<Args<'source>>,
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
        let s = |v: &Option<Args<'source>>, p: &Option<Args<'source>>| {
            i!(v)
                .map(|x| {
                    i!(p).position_first(|y| {
                        x == y
                            || y.iter()
                                .find(|&x| {
                                    Self::ACCESS_VARS
                                        .iter()
                                        .find(|&&y| {
                                            y.starts_with(match x {
                                                StrComponent::Lookup(z) => z.as_str(),
                                                _ => "",
                                            })
                                        })
                                        .is_some()
                                })
                                .is_some()
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
        [
            i!(self.created)
                .map(|x| {
                    i!(self.created).position_first(|y| x == y)
                        != i!(self.created).position_last(|y| x == y)
                })
                .position_first(|x| x),
            i!(self.modified)
                .map(|x| {
                    i!(self.modified).position_first(|y| x == y)
                        != i!(self.modified).position_last(|y| x == y)
                })
                .position_first(|x| x),
            i!(self.accessed)
                .map(|x| {
                    i!(self.accessed).position_first(|y| x == y)
                        != i!(self.accessed).position_last(|y| x == y)
                })
                .position_first(|x| x),
        ]
    }
    /// Get metadata of a specific value in a combined [IO]
    fn get_real_md(
        pos: usize,
        md: &HashMap<Range<usize>, Range<usize>>,
        offset: usize,
    ) -> Range<usize> {
        let mut real = 0..0;
        for x in md
            .iter()
            .map(|x| {
                if x.1.contains(&(offset + pos)) {
                    Some(x.1)
                } else {
                    None
                }
            })
            .collect::<Vec<Option<&Range<usize>>>>()
        {
            if let Some(s) = x {
                real = s.to_owned()
            }
        }
        return real;
    }
    /// Check for matches through operations
    fn cross_check_io_ops<F>(&self, err_cb: F)
    where
        F: FnOnce(usize, &'source str) -> !,
    {
        let [created, modified, accessed] = self.cross_search_matches();
        if let Some(pos) = created {
            err_cb(pos, "created")
        }
        if let Some(pos) = modified {
            err_cb(pos, "modified")
        }
        if let Some(pos) = accessed {
            err_cb(pos, "accessed")
        }
    }
    /// Check for matches through every operation
    fn plain_check_io_ops<F>(&self, err_cb: F)
    where
        F: FnOnce(usize, &'source str) -> !,
    {
        let [created, modified, accessed] = self.plain_search_matches();
        if let Some(pos) = created {
            err_cb(pos, "created")
        }
        if let Some(pos) = modified {
            err_cb(pos, "modified")
        }
        if let Some(pos) = accessed {
            err_cb(pos, "accessed")
        }
    }
    /// a wrapper over plain and cross checks
    fn check_ops<F: Copy>(&self, err_cb: F)
    where
        F: FnOnce(usize, &'source str) -> !,
    {
        self.cross_check_io_ops(err_cb);
        self.plain_check_io_ops(err_cb);
    }
}

impl<'source> crate::ast::Script<'source> {
    /// this will be called after getting the AST
    /// and will static-analyze the code and spot
    /// possible undefined-behavior cases, if there
    /// are, it'll prevent voila from running unless
    /// you opt-out of it with `--bypass-all-checks`
    pub fn ub_checks(&self, source: &'source str) {
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
                        &format!("this call {e} a file while another one creates, accesses or modifies it"),
                        source,
                        IO::get_real_md(
                            pos,
                            &io.metadata.get_multiple()[0],
                            cycle.calls[0].offset(),
                        ),
                    )
                });
            }
        }
    }
    fn action(&self, func: Function, args: Args<'source>, metadata: Metadata) -> IO<'source> {
        use Function::*;

        let mut created = None;
        let mut accessed = None;
        let mut modified = None;

        match func {
            Mkdir { safe: true } | Create { safe: true } => created = Some(args),
            Print { safe: true } => accessed = Some(args),
            Shell { safe: true } => {
                todo!("idk what to do with this")
            },
            Delete { safe: true } => modified = Some(vec![args[0].clone()]),
            Move { safe: true } | GzipDecompress { safe: true } => {
                modified = Some(vec![args[0].clone()]);
                created = Some(vec![args[1].clone()]);
            },
            Copy { safe: true } | GzipCompress { safe: true } => {
                accessed = Some(vec![args[0].clone()]);
                created = Some(vec![args[1].clone()]);
            },
            _ => unreachable!(),
        }
        IO::new(created, accessed, modified, metadata)
    }
    fn raise(&self, err: &'source str, code: &'source str, span: Range<usize>) -> ! {
        use crate::error::SourceError;
        use std::process;

        let mut error = SourceError::new(err);
        error = error.with_source(span, code);
        error = error.with_context("checking possible undefined behavior cases");
        error = error.with_context("checking data races");
        eprintln!("{error}");
        drop(error);
        process::exit(1)
    }
}
