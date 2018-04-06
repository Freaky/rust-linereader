/*
A fast line-oriented buffered reader.

lines(): 501636842 lines in 175.270838902
read_until: 501636842 lines in 120.907068634
LineReader: 501636842 lines in 102.173938545
*/

use std::io;
// use std::slice::Split;

const NEWLINE: u8 = b'\n';

pub struct LineReader<T> {
    io: T,
    eof: bool,
    buf: Vec<u8>,
    pos: Option<usize>,
    last_newline: usize,
    // split: Option<Split<&'a [u8], &'a [u8]>>,
}

impl <T: io::Read>LineReader<T> {
    pub fn new(io: T) -> LineReader<T> {
        LineReader {
             io,
             eof: false,
             buf: Vec::with_capacity(1024 * 128),
             // split: None,
             pos: None,
             last_newline: 0,
        }
    }

    fn refill(&mut self) -> io::Result<bool> {
        // self.split = None;
        self.pos = Some(0);

        // println!("Drain to {}", self.last_newline);
        // Move the start of the next line, if any, to the start of buf
        self.buf.drain(0..self.last_newline).count();

        // Fill the rest of buf from the underlying IO
        let len = self.buf.len();
        let cap = self.buf.capacity();

        // println!("fill buf {}..{}", len, cap);
        let r = unsafe {
            self.buf.set_len(cap);
            let r = self.io.read(&mut self.buf[len..cap])?;
            self.buf.set_len(len + r);
            r
        };

        self.eof = r == 0;
        // println!("fill buf -> {}", r);

        // Find the new last end of line, unless we're at EOF
        // XXX: What's sensible behavior for missing newlines?
        self.last_newline = self.buf.iter().rposition(|&c| c == NEWLINE).unwrap_or(self.buf.len());
        // println!("last newline: {}", self.last_newline);

        // Make a new split iterator on the updated buf
        // self.split = &self.buf[0..self.last_newline].split(|b| b == NEWLINE);
        Ok(r > 0)
    }

    pub fn next_line(&mut self) -> io::Result<&[u8]> {
        if self.eof {
            return Err(Error::new(ErrorKind::Other, "EOF"));
        }
        use std::io::{Error, ErrorKind};
        if let Some(pos) = self.pos {
            let nextpos = 1 + pos + self.buf[pos..].iter().position(|&c| c == NEWLINE).unwrap_or(self.buf.len() - pos);
            // println!("current={} next={}", pos, nextpos);
            if nextpos < self.last_newline {
                // println!("advance to {}", nextpos);
                self.pos = Some(nextpos);
                return Ok(&self.buf[pos..nextpos]);
            }
        }

        // self.refill()?;

        match self.refill() {
            Ok(true) => { self.next_line() },
            Ok(false) => { Ok(&self.buf) },
            Err(e) => { Err(e) }
        }
    }

    pub fn finish(mut self) -> T {
        self.io
    }
}

