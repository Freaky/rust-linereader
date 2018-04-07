LineReader
----------
A fast line-oriented reader for Rust.

## Summary

In my tests LineReader is 10-25% faster than the typically-recommended fastest
stdlib option: `BufReader::read_until()`. It achieves this by avoiding copying
from its own internal buffer, instead returning immutable slices of its own.

Like `read_until`, it does *not* perform UTF-8 processing - you get a slice of
raw u8's, including the delimiter, and nothing more.

Lines are limited to the size of the internal buffer (default 1MB).

## Performance

Comparison with using typical BufReader methods against pwned-passwords-2.0.txt:

Westmere Xeon 2.1GHz, FreeBSD/ZFS, 29GB, 501.6 million lines:

| Method   | Time | Lines/sec | Bandwidth |
|----------|------:|----------:|----------:|
|128k read | 36.85s| 13,612,940|817.92 MB/s|
|LineReader| 73.96s|  6,782,542|407.52 MB/s|
|read_until|119.30s|  4,204,835|252.62 MB/s|
|read_line |139.14s|  3,605,267|216.61 MB/s|
|lines()   |167.17s|  3,000,759|174.57 MB/s|

Haswell Xeon 3.4GHz, Windows 10 Subystem for Linux, 5.9GB, 100 million lines:

| Method   | Time | Lines/sec | Bandwidth |
|----------|-----:|----------:|------------:|
|128k read | 1.83s| 54,644,809|3,282.17 MB/s|
|LineReader| 2.98s| 33,557,047|2,016.28 MB/s|
|read_until| 3.43s| 29,154,519|1,752.24 MB/s|
|read_line | 5.17s| 19,342,360|1,162.51 MB/s|
|lines()   | 7.83s| 12,771,392|  742.52 MB/s|

It's also surprisingly fast on debug builds (or stdlib is surprisingly slow):

| Method   | Time | Lines/sec | Bandwidth |
|----------|-------:|----------:|------------:|
|128k read |   1.82s| 54,945,055|3,296.37 MB/s|
|LineReader|  29.17s|  3,428,180|  205.98 MB/s|
|read_until| 368.02s|    271,724|   16.33 MB/s|
|read_line | 383.00s|    261,097|   15.69 MB/s|
|lines()   | 220.28s|    453,968|   26.41 MB/s|

Hmmm.

## Usage

No crate or anything yet - I want a test suite first.  And an iterator version
would be nice.

    // Note BufReader will result in unnecessary copying, so, er, don't do that.
    let mut file = File::open(myfile).expect("open");

    // or LineReader::with_capacity(usize);
    let reader = LineReader::new(file);

    // optional
    reader.set_delimiter(b'\n');

    while let Some(line) = reader.next_line() {
      line.expect("oh noes, an IO error");
    }
