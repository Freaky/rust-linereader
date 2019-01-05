# LineReader [![Build Status](https://travis-ci.org/Freaky/rust-linereader.svg?branch=master)](https://travis-ci.org/Freaky/rust-linereader)

## Synopsis

`LineReader` is a byte-delimiter-focused buffered reader for Rust, meant as a
faster, less error-prone alternative to `BufRead::read_until`.

It provides three main functions:


### `next_line()`

Returns `Option<io::Result<&[u8]>>` - `None` on end-of-file, an IO error from the
wrapped reader, or an immutable byte slice ending on and including any delimiter.

Line length is limited to the size of the internal buffer - longer lines will be
spread across multiple reads.

In contrast with `read_until`, detecting end-of-file is more natural with the
use of `Option`; line length is naturally limited to some sensible value without
the use of `by_ref().take(limit)`; copying is minimised by returning borrowed
slices; you'll never forget to call `buf.clear()`.


### `next_batch()`

Behaves identically to `next_line()`, except it returns a slice of *all* the
complete lines in the buffer.


### `for_each()`

Calls a closure on each line of the input, while the closure returns true and
no IO errors are detected.  Such errors terminate iteration and are returned
from the function.


## Example

```rust
extern crate linereader;
use linereader::LineReader;

let mut file = File::open(myfile).expect("open");

// Defaults to a 64 KiB buffer and b'\n' delimiter; change with one of:
//  * LineReader::with_capacity(usize);
//  * LineReader::with_delimiter(u8);
//  * LineReader::with_delimiter_and_capacity(u8, usize)
let mut reader = LineReader::new(file);

while let Some(line) = reader.next_line() {
    let line = line.expect("read error");
    // line is a &[u8] owned by reader.
}
```

## Performance

Tests performed using ['Dickens_Charles_Pickwick_Papers.xml'](http://hur.st/Dickens_Charles_Pickwick_Papers.xml.xz),
concatinated to itself 480 times.  The resulting file is 976 MB and 10.3 million lines long.

### Westmere Xeon 2.1GHz, FreeBSD/ZFS.

| Method           |  Time   |  Lines/sec  |   Bandwidth   |
|------------------|--------:|------------:|--------------:|
| read()           |   0.25s |  41429738/s |  3907.62 MB/s |
| LR::next_batch() |   0.27s |  38258946/s |  3608.55 MB/s |
| LR::next_line()  |   1.51s |   6874006/s |   648.35 MB/s |
| read_until()     |   1.94s |   5327387/s |   502.47 MB/s |
| read_line()      |   2.54s |   4081562/s |   384.97 MB/s |
| lines()          |   3.23s |   3199491/s |   301.77 MB/s |
