use super::ErrorKind;
use super::ExprResult;
use super::Lookup;
use super::LookupError;
use super::PartialReader;
use std::collections::HashMap;
use std::lazy::OnceCell;

#[derive(Debug)]
pub struct Cache {
    variables: HashMap<Lookup, ExprResult>,
    file: OnceCell<PartialReader<std::fs::File>>,
    metadata: OnceCell<std::fs::Metadata>,
    path: std::path::PathBuf,
}

impl Cache {
    pub fn new(path: std::path::PathBuf) -> Self {
        Self {
            variables: HashMap::new(),
            metadata: OnceCell::new(),
            file: OnceCell::new(),
            path,
        }
    }
    pub fn get_path(&self) -> &std::path::PathBuf {
        &self.path
    }
    pub fn get_file_mut(&mut self) -> Result<&mut PartialReader<std::fs::File>, ErrorKind> {
        self.file.get_or_try_init(|| {
            std::fs::File::open(&self.path)
                .map_err(ErrorKind::from)
                .map(PartialReader::new)
        })?;
        Ok(self.file.get_mut().unwrap())
    }
    pub fn get_file_metadata(&mut self) -> Result<&std::fs::Metadata, ErrorKind> {
        self.metadata
            .get_or_try_init(|| std::fs::metadata(&self.path).map_err(ErrorKind::from))
    }
    pub fn get_lookup(&self, lookup: Lookup) -> Result<&ExprResult, ErrorKind> {
        self.variables
            .get(&lookup)
            .ok_or_else(|| ErrorKind::from(LookupError::new(lookup)))
    }
    pub fn resolve_var(&mut self, lookup: Lookup) -> Result<ExprResult, ErrorKind> {
        if self.variables.get(&lookup).is_none() {
            let res = lookup.cached_resolve(self)?;
            self.variables.insert(lookup, res);
        }
        Ok(self.variables[&lookup].clone())
    }
    /// Alternative to [`Resolve::resolve`]
    pub fn resolve<C: Resolve>(&mut self, resolved: &C) -> Result<ExprResult, ErrorKind> {
        resolved.resolve(self)
    }
}

pub trait Resolve {
    fn resolve(&self, cache: &mut Cache) -> Result<ExprResult, ErrorKind>;
}

// trait that is only implemented for the lookup
pub trait CachedResolve {
    fn cached_resolve(&self, cache: &mut Cache) -> Result<ExprResult, ErrorKind>;
}
