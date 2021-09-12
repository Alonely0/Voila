use enum_dispatch::enum_dispatch;
use std::io::{self, Read};

#[enum_dispatch]
pub trait Hash: Sized {
    fn update_block(&mut self, block: &[u8]);
    fn end_hash(self) -> String;
    fn hash_reader<R: Read>(mut self, reader: &mut R) -> io::Result<String> {
        with_blocks(reader, |block| self.update_block(block))?;
        Ok(self.end_hash())
    }
}

use md5::Context as Md5;
use ring::digest::Context as RingContext;
use ring::digest::{
    SHA1_FOR_LEGACY_USE_ONLY as Sha1, SHA256 as Sha256, SHA384 as Sha384, SHA512 as Sha512,
};
use sha2::Digest;
use sha2::Sha224;

#[enum_dispatch(Hash)]
pub enum Hasher {
    RingContext,
    Md5,
    Sha224,
}

impl Hash for Sha224 {
    fn update_block(&mut self, block: &[u8]) {
        self.update(block);
    }
    fn end_hash(self) -> String {
        format!("{:x}", self.finalize())
    }
}

impl Hash for Md5 {
    fn update_block(&mut self, block: &[u8]) {
        self.consume(block);
    }
    fn end_hash(self) -> String {
        format!("{:x}", self.compute())
    }
}

impl Hash for RingContext {
    fn update_block(&mut self, block: &[u8]) {
        self.update(block);
    }
    fn end_hash(self) -> String {
        let digest = self.finish();
        let mut str = String::new();
        for x in digest.as_ref() {
            str += &format!("{:02x}", x);
        }
        str
    }
}

use crate::ast::SumKind;
impl Hasher {
    pub fn select_from_sum(target_sum: SumKind) -> Self {
        match target_sum {
            SumKind::Md5 => Self::from(Md5::new()),
            SumKind::Sha1 => Self::from(RingContext::new(&Sha1)),
            SumKind::Sha224 => Self::from(Sha224::new()),
            SumKind::Sha256 => Self::from(RingContext::new(&Sha256)),
            SumKind::Sha384 => Self::from(RingContext::new(&Sha384)),
            SumKind::Sha512 => Self::from(RingContext::new(&Sha512)),
        }
    }
}

fn with_blocks<F, R: Read>(reader: &mut R, mut cont: F) -> Result<(), io::Error>
where
    F: FnMut(&[u8]),
    R: Read,
{
    let mut buf = [0u8; 8192];
    loop {
        let len = reader.read(&mut buf)?;
        if len == 0 {
            break;
        }
        cont(&buf[..len])
    }
    Ok(())
}
