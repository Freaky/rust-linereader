use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use std::str;

extern crate memchr;

mod line_reader;
use line_reader::*;

const BUFFER_SIZE: usize = 1024 * 1024;

fn report(name: &str, lines: usize, bytes: usize, elapsed: Duration) {
    let elapsed =
        (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0);
    println!(
        "{}: {} lines {} bytes in {:.2}s ({:.2} MB/s)",
        name,
        lines,
        bytes,
        elapsed,
        ((bytes as f64) / elapsed) / (1024.0 * 1024.0)
    );
}

fn try_baseline(filename: &str) {
    let mut infile = File::open(filename).expect("open");

    let start = Instant::now();
    let mut bytes = 0;

    let mut buf = [0; 1024 * 128];
    loop {
        let r = infile.read(&mut buf[..]).unwrap_or(0);
        bytes += r;
        if r == 0 {
            break;
        }
    }

    report("128k blocks", 0, bytes, start.elapsed());
}

fn try_linereader(filename: &str) {
    let infile = File::open(filename).expect("open");

    let mut reader = LineReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut count = 0;
    let mut bytes = 0;
    while let Some(line) = reader.next_line() {
        bytes += line.unwrap().len();
        count += 1;
    }

    report("LineReader", count, bytes, start.elapsed());
}

fn try_read_until(filename: &str) {
    let infile = File::open(filename).expect("open");
    let mut infile = BufReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut count = 0;
    let mut bytes = 0;
    let mut line: Vec<u8> = Vec::with_capacity(128);
    while infile.read_until(b'\n', &mut line).unwrap_or(0) > 0 {
        bytes += line.len();
        count += 1;
        line.clear();
    }

    report("read_until", count, bytes, start.elapsed());
}

fn try_read_line(filename: &str) {
    let infile = File::open(filename).expect("open");
    let mut infile = BufReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut count = 0;
    let mut bytes = 0;
    let mut line = String::new();
    while infile.read_line(&mut line).unwrap_or(0) > 0 {
        bytes += line.len();
        count += 1;
        line.clear();
    }

    report("read_line", count, bytes, start.elapsed());
}

fn try_lines_iter(filename: &str) {
    let infile = File::open(filename).expect("open");
    let infile = BufReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut bytes = 0;
    let mut count = 0;
    for line in infile.lines() {
        bytes += line.unwrap().len();
        count += 1;
    }

    report("lines()", count, bytes, start.elapsed());
}

const TESTFILE: &str = "Dickens_Charles_Pickwick_Papers.xml";
// const TESTFILE: &str = "/dump/wordlists/pwned-passwords-2.0.txt";
// const TESTFILE: &str = "/dump/wordlists/rockyou-withcount.txt";
// const TESTFILE: &str = "testdata";

fn main() {
    try_baseline(TESTFILE);
    try_linereader(TESTFILE);
    try_read_until(TESTFILE);
    try_read_line(TESTFILE);
    try_lines_iter(TESTFILE);
}
