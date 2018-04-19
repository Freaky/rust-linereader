/*
A fast line-oriented buffered reader.

128k blocks:        0 lines 31603121046 bytes in  36.85s (817.92 MB/s)
LineReader: 501636842 lines 31603121046 bytes in  73.96s (407.52 MB/s)
read_until: 501636842 lines 31603121046 bytes in 119.30s (252.62 MB/s)
read_line:  501636842 lines 31603121046 bytes in 139.14s (216.61 MB/s)
lines():    501636842 lines 30599847362 bytes in 167.17s (174.57 MB/s)
*/

use std::cmp;
use std::io;
use std::io::ErrorKind;

use memchr::{memchr, memrchr};

const NEWLINE: u8 = b'\n';
const DEFAULT_CAPACITY: usize = 1024 * 1024;

pub struct LineReader<R> {
    inner: R,
    buf: Vec<u8>,
    pos: usize,
    delimiter: u8,
    end_of_complete: usize,
    end_of_buffer: usize,
}

impl<R: io::Read> LineReader<R> {
    pub fn new(inner: R) -> Self {
        Self::with_delimiter_and_capacity(NEWLINE, DEFAULT_CAPACITY, inner)
    }

    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self::with_delimiter_and_capacity(NEWLINE, capacity, inner)
    }

    pub fn with_delimiter(delimiter: u8, inner: R) -> Self {
        Self::with_delimiter_and_capacity(delimiter, DEFAULT_CAPACITY, inner)
    }

    pub fn with_delimiter_and_capacity(delimiter: u8, capacity: usize, inner: R) -> Self {
        Self {
            inner,
            buf: vec![0; capacity],
            pos: 0,
            delimiter,
            end_of_complete: 0,
            end_of_buffer: 0,
        }
    }

    pub fn next_line(&mut self) -> Option<io::Result<&[u8]>> {
        let end = cmp::min(self.end_of_complete, self.end_of_buffer);

        if self.pos < end {
            let pos = self.pos;
            let nextpos = cmp::min(
                1 + pos + memchr(self.delimiter, &self.buf[pos..end]).unwrap_or(end),
                end,
            );

            self.pos = nextpos;
            return Some(Ok(&self.buf[pos..nextpos]));
        }

        match self.refill() {
            Ok(true) => self.next_line(),
            Ok(false) => {
                if self.end_of_buffer == self.pos {
                    None
                } else {
                    Some(Ok(&self.buf[..self.end_of_buffer]))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }

    fn refill(&mut self) -> io::Result<bool> {
        assert!(self.pos == self.end_of_complete);
        self.pos = 0;

        // Move the start of the next line, if any, to the start of buf
        let fragment_len = self.end_of_buffer - self.end_of_complete;
        if fragment_len > 0 {
            let (start, rest) = self.buf.split_at_mut(self.end_of_complete);
            start[0..fragment_len].copy_from_slice(&rest[0..fragment_len]);
            self.end_of_buffer = fragment_len;
        } else {
            self.end_of_buffer = 0;
        }

        let r;
        // Fill the rest of buf from the underlying IO
        loop {
            match self.inner.read(&mut self.buf[self.end_of_buffer..]) {
                Ok(n) => {
                    r = n;
                    break;
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(e) => return Err(e),
            }
        }

        self.end_of_buffer += r;

        // Find the new last end of line
        self.end_of_complete = cmp::min(
            1 + memrchr(self.delimiter, &self.buf[..self.end_of_buffer])
                .unwrap_or(self.end_of_buffer),
            self.end_of_buffer,
        );

        Ok(r > 0)
    }

    /// Get a reference to the reader.
    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    /// Get a mutable reference to the reader.
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    /// Unwrap this `LineReader`, returning the underlying reader and discarding any
    /// unread buffered lines.
    pub fn into_inner(self) -> R {
        self.inner
    }
}
