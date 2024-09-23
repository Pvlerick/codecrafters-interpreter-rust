use std::{io::Read, slice::Iter};

pub struct StrReader<'a> {
    iter: Iter<'a, u8>,
}

impl<'a> StrReader<'a> {
    pub fn new(content: &'a str) -> StrReader {
        StrReader {
            iter: content.as_bytes().iter(),
        }
    }
}

impl<'a> Read for StrReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        for i in 0..buf.len() {
            match self.iter.next() {
                Some(b) => buf[i] = *b,
                None => return Ok(i),
            }
        }
        Ok(buf.len())
    }
}
