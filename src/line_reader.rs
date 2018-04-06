/*
A fast line-oriented buffered reader.

lines(): 501636842 lines in 175.270838902
read_until: 501636842 lines in 120.907068634
LineReader: 501636842 lines in 102.173938545
*/

use std::io;

const NEWLINE: u8 = b'\n';

pub struct LineReader<T> {
    io: T,
    eof: bool,
    buf: Vec<u8>,
    pos: Option<usize>,
    last_newline: usize,
    end_of_buffer: usize,
}

impl <T: io::Read>LineReader<T> {
    pub fn new(io: T) -> LineReader<T> {
        LineReader {
             io,
             eof: false,
             buf: vec![0; 1024 * 128],
             pos: None,
             last_newline: 0,
             end_of_buffer: 0,
        }
    }

    fn refill(&mut self) -> io::Result<bool> {
        self.pos = Some(0);

        // Move the start of the next line, if any, to the start of buf
        if self.last_newline > 0 {
            let next_line_len = self.end_of_buffer - self.last_newline;
            let (start, rest) = self.buf.split_at_mut(self.last_newline);
            start[0..next_line_len].copy_from_slice(&rest[0..next_line_len]);
            self.end_of_buffer = next_line_len;
        } else {
            self.end_of_buffer = 0;
        }

        // Fill the rest of buf from the underlying IO
        let r = self.io.read(&mut self.buf[self.end_of_buffer..])?;
        self.end_of_buffer += r;

        self.eof = r == 0;
        // Find the new last end of line, unless we're at EOF
        // XXX: What's sensible behavior for missing newlines?
        self.last_newline = self.buf[..self.end_of_buffer].iter().rposition(|&c| c == NEWLINE).unwrap_or(self.end_of_buffer);

        Ok(r > 0)
    }

    pub fn next_line(&mut self) -> io::Result<&[u8]> {
        if self.eof {
            return Err(Error::new(ErrorKind::Other, "EOF"));
        }
        use std::io::{Error, ErrorKind};
        if let Some(pos) = self.pos {
            let nextpos = 1 + pos + self.buf[pos..self.end_of_buffer].iter().position(|&c| c == NEWLINE).unwrap_or(self.buf.len() - pos);
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
            Ok(false) => { Ok(&self.buf[..self.end_of_buffer]) },
            Err(e) => { Err(e) }
        }
    }

    pub fn finish(mut self) -> T {
        self.io
    }
}

