use std::io::{self, Read};
/// reads partially but caches everything in a growable buffer.
/// Eventually it will read the whole file into the buffer.
#[derive(Debug)]
pub struct PartialReader<R: Read> {
    source: R,
    buf: Vec<u8>,
    cursor: usize,
}

impl<R: Read> PartialReader<R> {
    pub fn new(source: R) -> Self {
        Self {
            source,
            buf: Vec::new(),
            cursor: 0,
        }
    }
    // read the file till the end
    pub fn fill(&mut self) -> io::Result<&[u8]> {
        self.cursor = self.buf.len();
        let mut rest = Vec::new();
        self.source.read_to_end(&mut rest)?;
        self.cursor += rest.len() - 1;
        self.buf.reserve(rest.len());
        self.buf.extend_from_slice(&rest);
        Ok(&self.buf)
    }
}

impl<R: Read> Read for PartialReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        // total requested
        let requested_len = buf.len();
        // what can I fulfill currently? (without calling I/O)
        let can_fulfill = (self.buf.len() - self.cursor).min(requested_len);
        if can_fulfill > 0 {
            buf[..can_fulfill].copy_from_slice(&self.buf[self.cursor..self.cursor + can_fulfill]);
            self.cursor += can_fulfill;
        }
        let not_fulfilled = requested_len - can_fulfill;
        let could_fulfill = can_fulfill
            + if not_fulfilled > 0 {
                // grow our buffer if needed to get more of the file. This updates the capacity
                self.buf.reserve(not_fulfilled);
                // UNSAFE: safe. we have already reserved the memory for that.
                // Though we won't update the length of the vector before we know
                // how many bytes are actually not garbage
                let actual_fulfilled = self.source.read(unsafe {
                    std::slice::from_raw_parts_mut(
                        self.buf.as_mut_ptr().add(self.cursor),
                        not_fulfilled,
                    )
                })?;
                unsafe { self.buf.set_len(self.buf.len() + actual_fulfilled) };

                buf[can_fulfill..can_fulfill + actual_fulfilled]
                    .copy_from_slice(&self.buf[self.cursor..self.cursor + actual_fulfilled]);
                self.cursor += actual_fulfilled;
                actual_fulfilled
            } else {
                0
            };
        Ok(could_fulfill)
    }
}

impl<R: Read> io::Seek for PartialReader<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        match pos {
            io::SeekFrom::Current(amt) => self.cursor += amt as usize,
            io::SeekFrom::Start(cursor) => self.cursor = cursor as usize,
            io::SeekFrom::End(cursor) => {
                self.fill()?;
                self.cursor = self.buf.len() - cursor as usize;
            },
        }
        Ok(self.cursor as u64)
    }
}
