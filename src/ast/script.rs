use super::Target;

// The script doesn't have a span, since it represents the **entire** script.
/// The whole voila script to execute, with a bunch of [Target]s
#[derive(Debug)]
pub struct Script<'source>(pub Vec<Target<'source>>);

use super::parser::*;

impl<'source> Parse<'source> for Script<'source> {
    fn parse(parser: &mut Parser<'source>) -> ParseRes<Self> {
        parser.many_eof().map(Self)
    }
}

use crate::interpreter::{Cache, ErrorKind};
use std::sync::{mpsc, Arc, Mutex};
pub fn run_script(
    script: &Script,
    path: std::path::PathBuf,
    pool: &rayon::ThreadPool,
    tx: mpsc::Sender<ErrorKind>,
) {
    let cache = Arc::new(Mutex::new(Cache::new(path)));
    pool.scope(move |s| {
        for target in &script.0 {
            let tx = tx.clone();
            let cache = cache.clone();
            s.spawn(move |_| {
                if let Err(e) = super::run_target(target, cache, pool, tx.clone()) {
                    tx.send(e).unwrap();
                }
            })
        }
    });
    // for target in &script.0 {
    //     let tx = tx.clone();
    //     let cache = cache.clone();
    //     pool.install(move || {
    //         let res = super::target::run_target(target, cache, pool, tx.clone());
    //         tx.send(res).unwrap();
    //     });
    // }
}
