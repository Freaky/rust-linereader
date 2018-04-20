//! LineReader
//!
//! A fast byte-delimiter-oriented buffered reader, offering a faster alternative
//! to `read_until` that returns byte slices into its internal buffer rather than
//! copying them out to one you provide.

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
const DEFAULT_CAPACITY: usize = 1024 * 1024;

/// The `Linereader` struct adds buffered, byte-delimited (default: b'\n')
/// reading to any io::Reader.
pub struct LineReader<R> {
    inner: R,
    delimiter: u8,
    buf: Vec<u8>,
    pos: usize,
    end_of_complete: usize,
    end_of_buffer: usize,
}

impl<R: io::Read> LineReader<R> {
    /// Create a new `LineReader` around the reader with a default capacity
    /// and delimiter of 1MiB and b'\n'
    ///
    /// ```no_run
    /// # use std::fs::file;
    /// let reader = LineReader::new(File::new("myfile.txt")?);
    /// ```
    pub fn new(inner: R) -> Self {
        Self::with_delimiter_and_capacity(NEWLINE, DEFAULT_CAPACITY, inner)
    }

    /// Create a new `LineReader` around the reader with a given capacity and
    /// delimiter of b'\n'.  Line length is limited to the capacity.
    ///
    /// ```no_run
    /// # use std::fs::file;
    /// let reader = LineReader::with_capacity(1024*64, File::new("myfile.txt")?);
    /// ```
    pub fn with_capacity(capacity: usize, inner: R) -> Self {
        Self::with_delimiter_and_capacity(NEWLINE, capacity, inner)
    }

    /// Create a new `LineReader` around the reader with a default capacity of
    /// 1MiB and the given delimiter.  Line length is limited to the capacity.
    ///
    /// ```no_run
    /// # use std::fs::file;
    /// let reader = LineReader::with_delimiter(b'\t', File::new("myfile.txt")?);
    /// ```
    pub fn with_delimiter(delimiter: u8, inner: R) -> Self {
        Self::with_delimiter_and_capacity(delimiter, DEFAULT_CAPACITY, inner)
    }

    /// Create a new `LineReader` around the reader with a given capacity and
    /// delimiter.  Line length is limited to the capacity.
    ///
    /// ```no_run
    /// # use std::fs::file;
    /// let reader = LineReader::with_delimiter_and_capacity(b'\t', 1024*64, File::new("myfile.txt")?);
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

    /// Get the next line from the reader, an IO error, or `None` on EOF.  The delimiter
    /// is included in any returned slice, unless the file ends without one or a line was
    /// truncated to the buffer size due to length.
    ///
    /// ```no_run
    /// # use std::fs::file;
    /// # let reader = LineReader::new(File::new("myfile.txt")?);
    /// while let Some(line) = reader.next_line() {
    ///     let line = line?;  // unwrap io::Result to &[u8]
    /// }
    /// ```
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

        // Fill the rest of buf from the underlying IO
        while self.end_of_buffer < self.buf.len() {
            // Loop until we find a delimiter or read zero bytes.
            match self.inner.read(&mut self.buf[self.end_of_buffer..]) {
                Ok(0) => {
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
