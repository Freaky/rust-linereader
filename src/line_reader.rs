/*
A fast line-oriented buffered reader.

128k blocks:                31603121046 bytes in 36.93 (816.03 MB/s)
LineReader: 501636842 lines 31603121046 bytes in 85.75 (351.48 MB/s)
read_until: 501636842 lines 31603121046 bytes in 119.34 (252.54 MB/s)
lines():    501636842 lines 30599847362 bytes in 160.22 (182.14 MB/s)
*/

use std::io;
use std::cmp;

const NEWLINE: u8 = b'\n';

pub struct LineReader<T> {
    io: T,
    buf: Vec<u8>,
    pos: usize,
    end_of_complete: usize,
    end_of_buffer: usize,
}

impl<T: io::Read> LineReader<T> {
    pub fn new(io: T) -> LineReader<T> {
        LineReader {
            io,
            buf: vec![0; 1024 * 128],
            pos: 0,
            end_of_complete: 0,
            end_of_buffer: 0,
        }
    }

    fn refill(&mut self) -> io::Result<bool> {
        assert!(self.pos == self.end_of_complete);
        self.pos = 0;

        // Move the start of the next line, if any, to the start of buf
        let fragment_len = self.end_of_buffer - self.end_of_complete;
        if self.end_of_complete < self.end_of_buffer {
            let (start, rest) = self.buf.split_at_mut(self.end_of_complete);
            start[0..fragment_len].copy_from_slice(&rest[0..fragment_len]);
            self.end_of_buffer = fragment_len;
        } else {
            self.end_of_buffer = 0;
        }

        // Fill the rest of buf from the underlying IO
        let r = self.io.read(&mut self.buf[self.end_of_buffer..])?;
        self.end_of_buffer += r;

        // Find the new last end of line, unless we're at EOF
        self.end_of_complete = cmp::min(
            self.buf[..self.end_of_buffer]
                .iter()
                .rposition(|&c| c == NEWLINE)
                .unwrap_or(self.end_of_buffer) + 1,
            self.end_of_buffer,
        );

        Ok(r > 0)
    }

    pub fn next_line(&mut self) -> Option<io::Result<&[u8]>> {
        let end = cmp::min(self.end_of_complete, self.end_of_buffer);

        if self.pos < end {
            let pos = self.pos;
            let nextpos = cmp::min(
                1 + pos
                    + self.buf[pos..end]
                        .iter()
                        .position(|&c| c == NEWLINE)
                        .unwrap_or(end),
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

    pub fn finish(self) -> T {
        self.io
    }
}
