use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;
use std::str;

mod line_reader;
use line_reader::*;

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

    let elapsed = start.elapsed();
    let elapsed =
        (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0);
    println!(
        "128k blocks: {} bytes in {:.2} ({:.2} MB/s)",
        bytes,
        elapsed,
        ((bytes as f64) / elapsed) / (1024.0 * 1024.0)
    );
}

fn try_linereader(filename: &str) {
    let infile = File::open(filename).expect("open");

    let mut reader = LineReader::new(infile);

    let start = Instant::now();
    let mut count = 0;
    let mut bytes = 0;
    while let Some(line) = reader.next_line() {
        // let s = str::from_utf8(&line).unwrap();
        // println!("Line:{}:eniL", s);
        bytes += line.unwrap().len();
        count += 1;
    }

    let elapsed = start.elapsed();
    let elapsed =
        (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0);
    println!(
        "LineReader: {} lines {} bytes in {:.2} ({:.2} MB/s)",
        count,
        bytes,
        elapsed,
        ((bytes as f64) / elapsed) / (1024.0 * 1024.0)
    );
}

fn try_lines_read_until(filename: &str) {
    let infile = File::open(filename).expect("open");
    let mut infile = BufReader::new(infile);

    let start = Instant::now();
    let mut count = 0;
    let mut bytes = 0;
    let mut line: Vec<u8> = Vec::with_capacity(128);
    while infile.read_until(b'\n', &mut line).unwrap_or(0) > 0 {
        bytes += line.len();
        count += 1;
        line.clear();
    }

    let elapsed = start.elapsed();
    let elapsed =
        (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0);
    println!(
        "read_until: {} lines {} bytes in {:.2} ({:.2} MB/s)",
        count,
        bytes,
        elapsed,
        ((bytes as f64) / elapsed) / (1024.0 * 1024.0)
    );
}

fn try_lines_iter(filename: &str) {
    let infile = File::open(filename).expect("open");
    let infile = BufReader::new(infile);

    let start = Instant::now();
    let mut bytes = 0;
    let mut count = 0;
    for line in infile.lines() {
        bytes += line.unwrap().len();
        count += 1;
    }

    let elapsed = start.elapsed();
    let elapsed =
        (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0);
    println!(
        "lines(): {} lines {} bytes in {:.2} ({:.2} MB/s)",
        count,
        bytes,
        elapsed,
        ((bytes as f64) / elapsed) / (1024.0 * 1024.0)
    );
}

const TESTFILE: &str = "/dump/wordlists/pwned-passwords-2.0.txt";
// const TESTFILE: &str = "/dump/wordlists/rockyou-withcount.txt";
// const TESTFILE: &str = "testdata";

fn main() {
    try_baseline(TESTFILE);
    try_linereader(TESTFILE);
    try_lines_read_until(TESTFILE);
    try_lines_iter(TESTFILE);
}
