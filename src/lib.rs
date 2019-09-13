//! LineReader
//!
//! A fast byte-delimiter-oriented buffered reader, offering a faster alternative
//! to `read_until` that returns byte slices into its internal buffer rather than
//! copying them out to one you provide.
//!
//! Because the internal buffer is fixed, lines longer than the buffer will be
//! split.

/*
128k blocks:        0 lines 31603121046 bytes in  36.85s (817.92 MB/s)
LineReader: 501636842 lines 31603121046 bytes in  73.96s (407.52 MB/s)
read_until: 501636842 lines 31603121046 bytes in 119.30s (252.62 MB/s)
read_line:  501636842 lines 31603121046 bytes in 139.14s (216.61 MB/s)
lines():    501636842 lines 30599847362 bytes in 167.17s (174.57 MB/s)
*/

use std::cmp;
use std::io;
use std::io::ErrorKind;

extern crate memchr;
use memchr::{memchr, memrchr};

const NEWLINE: u8 = b'\n';
const DEFAULT_CAPACITY: usize = 1024 * 64;

/// The `LineReader` struct adds buffered, byte-delimited (default: `\n`)
/// reading to any io::Reader.
pub struct LineReader<R> {
    inner: R,
    delimiter: u8,
    buf: Vec<u8>,
    pos: usize,
    end_of_complete: usize,
    end_of_buffer: usize,
}

use std::fmt;

impl<R: io::Read> fmt::Debug for LineReader<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "LineReader {{ delimiter: {:?}, pos: {}, end_of_complete: {}, end_of_buffer: {} }}",
            self.delimiter, self.pos, self.end_of_complete, self.end_of_buffer
        )
    }
}

impl<R: io::Read> LineReader<R> {
    /// Create a new `LineReader` around the reader with a default capacity of
    /// 64 KiB and delimiter of `\n`.
    ///
    /// ```no_run
    /// # use linereader::LineReader;
    /// # use std::fs::File;
    /// # use std::io;
    /// # fn x() -> io::Result<()> {
    /// let reader = LineReader::new(File::open("myfile.txt")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(inner: R) -> Self {
        Self::with_delimiter_and_capacity(NEWLINE, DEFAULT_CAPACITY, inner)
    }

    /// Create a new `LineReader` around the reader with a given capacity and
    /// delimiter of `\n`.
    ///
    /// ```no_run
    /// # use linereader::LineReader;
    /// # use std::fs::File;
    /// # use std::io;
    /// # fn x() -> io::Result<()> {
    /// let mut reader = LineReader::with_capacity(1024*512, File::open("myfile.txt")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self::with_delimiter_and_capacity(NEWLINE, capacity, inner)
    }

    /// Create a new `LineReader` around the reader with a default capacity of
    /// 64 KiB and the given delimiter.
    ///
    /// ```no_run
    /// # use linereader::LineReader;
    /// # use std::fs::File;
    /// # use std::io;
    /// # fn x() -> io::Result<()> {
    /// let mut reader = LineReader::with_delimiter(b'\t', File::open("myfile.txt")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_delimiter(delimiter: u8, inner: R) -> Self {
        Self::with_delimiter_and_capacity(delimiter, DEFAULT_CAPACITY, inner)
    }

    /// Create a new `LineReader` around the reader with a given capacity and
    /// delimiter.
    ///
    /// ```no_run
    /// # use linereader::LineReader;
    /// # use std::fs::File;
    /// # use std::io;
    /// # fn x() -> io::Result<()> {
    /// let mut reader = LineReader::with_delimiter_and_capacity(b'\t', 1024*512, File::open("myfile.txt")?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_delimiter_and_capacity(delimiter: u8, capacity: usize, inner: R) -> Self {
        Self {
            inner,
            delimiter,
            buf: vec![0; capacity],
            pos: 0,
            end_of_complete: 0,
            end_of_buffer: 0,
        }
    }

    /// Run the given closure for each line while while the closure returns `Ok(true)`.
    ///
    /// If either the reader or the closure return an error, iteration ends and the error is returned.
    ///
    /// ```no_run
    /// # use linereader::LineReader;
    /// # use std::fs::File;
    /// # use std::io;
    /// # fn x() -> io::Result<()> {
    /// let buf: &[u8] = b"foo\nbar\nbaz";
    /// let mut reader = LineReader::new(buf);
    /// let mut lines = vec![];
    /// reader.for_each(|line| {
    ///     lines.push(line.to_vec());
    ///     Ok(true)
    /// })?;
    /// assert_eq!(lines.len(), 3);
    /// assert_eq!(lines[0], b"foo\n");
    /// assert_eq!(lines[1], b"bar\n");
    /// assert_eq!(lines[2], b"baz");
    /// # Ok(())
    /// # }
    /// ```
    pub fn for_each<F: FnMut(&[u8]) -> io::Result<bool>>(&mut self, mut f: F) -> io::Result<()> {
        while let Some(line) = self.next_line() {
            if !f(line?)? {
                break;
            }
        }

        Ok(())
    }

    /// Get the next line from the reader, an IO error, or `None` on EOF.  The delimiter
    /// is included in any returned slice, unless the file ends without one or a line was
    /// truncated to the buffer size due to length.
    ///
    /// ```no_run
    /// # use linereader::LineReader;
    /// # use std::fs::File;
    /// # use std::io;
    /// # fn x() -> io::Result<()> {
    /// # let mut reader = LineReader::new(File::open("myfile.txt")?);
    /// while let Some(line) = reader.next_line() {
    ///     let line = line?;  // unwrap io::Result to &[u8]
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn next_line(&mut self) -> Option<io::Result<&[u8]>> {
        if self.pos < self.end_of_complete {
            let lastpos = self.pos;
            self.pos = cmp::min(
                1 + lastpos
                    + memchr(self.delimiter, &self.buf[lastpos..self.end_of_complete])
                        .unwrap_or(self.end_of_complete),
                self.end_of_complete,
            );

            return Some(Ok(&self.buf[lastpos..self.pos]));
        }

        match self.refill() {
            Ok(true) => self.next_line(),
            Ok(false) => {
                if self.end_of_buffer == self.pos {
                    None
                } else {
                    self.pos = self.end_of_buffer;
                    Some(Ok(&self.buf[..self.end_of_buffer]))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }

    /// Return a slice of complete lines, up to the size of the internal buffer.
    ///
    /// This is functionally identical to next_line, only instead of getting up
    /// to the *first* instance of the delimiter, you get up to the *last*.
    ///
    /// ```no_run
    /// # use linereader::LineReader;
    /// # use std::fs::File;
    /// # use std::io;
    /// # fn x() -> io::Result<()> {
    /// # let mut reader = LineReader::new(File::open("myfile.txt")?);
    /// while let Some(lines) = reader.next_batch() {
    ///     let lines = lines?;  // unwrap io::Result to &[u8]
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn next_batch(&mut self) -> Option<io::Result<&[u8]>> {
        if self.pos < self.end_of_complete {
            let ret = &self.buf[self.pos..self.end_of_complete];
            self.pos = self.end_of_complete;
            return Some(Ok(ret));
        }

        match self.refill() {
            Ok(true) => self.next_batch(),
            Ok(false) => {
                if self.end_of_buffer == self.pos {
                    None
                } else {
                    self.pos = self.end_of_buffer;
                    Some(Ok(&self.buf[..self.end_of_buffer]))
                }
            }
            Err(e) => Some(Err(e)),
        }
    }

    fn refill(&mut self) -> io::Result<bool> {
        assert!(self.pos == self.end_of_complete);
        assert!(self.end_of_complete <= self.end_of_buffer);

        self.pos = 0;

        // Move the start of the next line, if any, to the start of buf
        let fragment_len = self.end_of_buffer - self.end_of_complete;
        if fragment_len > 0 {
            // unsafe variants of these using ptr::copy/copy_nonoverlapping can
            // be found in 5ccea2c - they made no appreciable difference.
            if fragment_len > self.end_of_complete {
                self.buf.drain(..self.end_of_complete);
                self.buf.extend(vec![0_u8; self.end_of_complete]);
            } else {
                let (start, rest) = self.buf.split_at_mut(self.end_of_complete);
                start[0..fragment_len].copy_from_slice(&rest[0..fragment_len]);
            }
            self.end_of_buffer = fragment_len;
        } else {
            self.end_of_buffer = 0;
        }

        // Fill the rest of buf from the underlying IO
        while self.end_of_buffer < self.buf.len() {
            // Loop until we find a delimiter or read zero bytes.
            match self.inner.read(&mut self.buf[self.end_of_buffer..]) {
                Ok(0) => {
                    self.end_of_complete = self.end_of_buffer;
                    return Ok(false);
                }
                Ok(n) => {
                    let lastpos = self.end_of_buffer;
                    self.end_of_buffer += n;
                    if let Some(nl) =
                        memrchr(self.delimiter, &self.buf[lastpos..self.end_of_buffer])
                    {
                        self.end_of_complete = cmp::min(self.end_of_buffer, 1 + lastpos + nl);
                        return Ok(true);
                    } else {
                        // No delimiter - see if we can read any more.
                        self.end_of_complete = self.end_of_buffer;
                    }
                }
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }

        // We read through until the end of the buffer.
        Ok(true)
    }

    /// Reset the internal state of the buffer.  Next lines are read from wherever
    /// the reader happens to be.
    pub fn reset(&mut self) {
        self.pos = 0;
        self.end_of_buffer = 0;
        self.end_of_complete = 0;
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

#[cfg(test)]
mod tests {
    use LineReader;

    #[test]
    fn test_next_line() {
        let buf: &[u8] = b"0a0\n1bb1\n2ccc2\n3dddd3\n4eeeee4\n5ffffffff5\n6ggggg6\n7hhhhhh7";
        let mut reader = LineReader::with_capacity(8, buf);

        assert_eq!(b"0a0\n", reader.next_line().unwrap().unwrap());
        assert_eq!(b"1bb1\n", reader.next_line().unwrap().unwrap());
        assert_eq!(b"2ccc2\n", reader.next_line().unwrap().unwrap());
        assert_eq!(b"3dddd3\n", reader.next_line().unwrap().unwrap());
        assert_eq!(b"4eeeee4\n", reader.next_line().unwrap().unwrap());
        assert_eq!(b"5fffffff", reader.next_line().unwrap().unwrap());
        assert_eq!(b"f5\n", reader.next_line().unwrap().unwrap());
        assert_eq!(b"6ggggg6\n", reader.next_line().unwrap().unwrap());
        assert_eq!(b"7hhhhhh7", reader.next_line().unwrap().unwrap());
        assert!(reader.next_line().is_none());
    }

    #[test]
    fn test_next_batch() {
        let buf: &[u8] = b"0a0\n1bb1\n2ccc2\n3dddd3\n4eeeee4\n5ffffffff5\n6ggggg6\n7hhhhhh7";
        let mut reader = LineReader::with_capacity(19, buf);

        assert_eq!(b"0a0\n1bb1\n2ccc2\n", reader.next_batch().unwrap().unwrap());
        assert_eq!(b"3dddd3\n4eeeee4\n", reader.next_batch().unwrap().unwrap());
        assert_eq!(
            b"5ffffffff5\n6ggggg6\n",
            reader.next_batch().unwrap().unwrap()
        );
        assert_eq!(b"7hhhhhh7", reader.next_batch().unwrap().unwrap());
    }

    #[test]
    fn test_for_each() {
        let buf: &[u8] = b"f\nba\nbaz\n";
        let mut reader = LineReader::new(buf);

        let mut len = 2;
        reader.for_each(|l| { assert_eq!(len, l.len()); len += 1; Ok(true) }).unwrap();

        let buf: &[u8] = b"f\nba\nbaz\n";
        let mut reader = LineReader::new(buf);

        reader.for_each(|l| { assert_eq!(l.len(), 2); Ok(false) }).unwrap();
    }

    extern crate rand;
    use std::io::BufRead;
    use std::io::{Cursor, Read};
    use tests::rand::prelude::*;

    #[test]
    fn test_next_line_randomly() {
        let mut rng = thread_rng();

        for _ in 1..128 {
            let mut buf = [0u8; 65535];
            rng.fill(&mut buf[..]);
            let delimiter = rng.gen::<u8>();
            let max_line = rng.gen::<u8>().saturating_add(8) as usize;

            let mut reader =
                LineReader::with_delimiter_and_capacity(delimiter, max_line, Cursor::new(&buf[..]));
            let mut cursor = Cursor::new(&buf[..]);
            let mut expected = vec![];

            while cursor
                .by_ref()
                .take(max_line as u64)
                .read_until(delimiter, &mut expected)
                .unwrap() > 0
            {
                assert_eq!(expected, reader.next_line().unwrap().unwrap());
                expected.clear();
            }

            assert!(reader.next_line().is_none());
        }
    }
}
