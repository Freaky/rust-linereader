# LineReader [![Build Status](https://travis-ci.org/Freaky/rust-linereader.svg?branch=master)](https://travis-ci.org/Freaky/rust-linereader)

## Synopsis

The `LineReader` struct is a byte-delimiter-focused buffered reader meant as a
faster, less error-prone alternative to `BufRead::read_until`.

It provides two main functions:


### `next_line()`

Returns `Option<io::Result<&[u8]>>` - `None` on end-of-file, an IO error from the
wrapped reader, or an immutable byte slice ending on and including any delimiter.

Line length is limited to the size of the internal buffer.

In contrast with `read_until`, detecting end-of-file is more natural with the
use of `Option`; line length is naturally limited to some sensible value without
the use of `by_ref().take(limit)`; copying is minimised by returning borrowed
slices; you'll never forget to call `buf.clear()`.


### `next_batch()`

Behaves identically to `next_line()`, except it returns a slice of *all* the complete
lines in the buffer.


## Example

    extern crate linereader;
    use linereader::LineReader;

    let mut file = File::open(myfile).expect("open");

    // Defaults to a 1 MiB buffer and b'\n' delimiter; change with one of:
    //  * LineReader::with_capacity(usize);
    //  * LineReader::with_delimiter(u8);
    //  * LineReader::with_delimiter_and_capacity(u8, usize)
    let mut reader = LineReader::new(file);

    while let Some(line) = reader.next_line() {
        let line = line.expect("read error");
        // line is a &[u8] owned by reader.
    }

Lines can also be read in batches for group processing - e.g. in threads:

    while let Some(lines) = reader.next_batch() {
        send(&chan, lines.expect("read error").to_vec());
    }

This should be more efficient than finding each intermediate delimiter in the main
thread, and allocating and sending each individual line.  Any line fragments at
the end of the internal buffer will be copied to the start in the next call.


## Performance

Tests performed using ['Dickens_Charles_Pickwick_Papers.xml'](http://hur.st/Dickens_Charles_Pickwick_Papers.xml.xz),
concatinated to itself 480 times.  The resulting file is 976 MB and 10.3 million lines long.

Buffers in each test are set to 1 MiB.

### Westmere Xeon 2.1GHz, FreeBSD/ZFS.

| Method           |  Time   |  Lines/sec  |   Bandwidth   |
|------------------|--------:|------------:|--------------:|
| read()           |   1.82s |   5,674,452/s |   535.21 MB/s |
| LR::next_batch() |   1.83s |   5,650,387/s |   532.94 MB/s |
| LR::next_line()  |   3.10s |   3,341,796/s |   315.20 MB/s |
| read_until()     |   3.62s |   2,861,864/s |   269.93 MB/s |
| read_line()      |   4.25s |   2,432,505/s |   229.43 MB/s |
| lines()          |   4.88s |   2,119,837/s |   199.94 MB/s |

### Haswell Xeon 3.4GHz, Windows 10 Subystem for Linux.

| Method           |  Time   |  Lines/sec  |   Bandwidth   |
|------------------|--------:|------------:|--------------:|
| read()           |   0.26s |  39,253,494/s |  3702.36 MB/s |
| LR::next_batch() |   0.26s |  39,477,365/s |  3723.47 MB/s |
| LR::next_line()  |   0.50s |  20,672,784/s |  1949.84 MB/s |
| read_until()     |   0.60s |  17,303,147/s |  1632.02 MB/s |
| read_line()      |   0.84s |  12,293,247/s |  1159.49 MB/s |
| lines()          |   1.53s |   6,783,849/s |   639.85 MB/s |

It's also surprisingly fast on debug builds (or stdlib is surprisingly slow):

| Method           |  Time   |  Lines/sec  |   Bandwidth   |
|------------------|--------:|------------:|--------------:|
| read()           |   0.27s |  38,258,105/s |  3608.47 MB/s |
| LR::next_batch() |   0.28s |  36,896,353/s |  3480.04 MB/s |
| LR::next_line()  |   2.99s |   3,463,911/s |   326.71 MB/s |
| read_until()     |  57.01s |     181,505/s |    17.12 MB/s |
| read_line()      |  58.36s |     177,322/s |    16.72 MB/s |
| lines()          |  21.06s |     491,320/s |    46.34 MB/s |
