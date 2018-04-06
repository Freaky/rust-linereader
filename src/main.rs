
use std::io;
use std::io::prelude::*;
use std::fs::File;
use std::io::BufReader;
use std::time::Instant;

mod line_reader;
use line_reader::*;

fn try_linereader(filename: &str) {
    let infile = File::open(filename).expect("open");

    let mut reader = LineReader::new(infile);

    let start = Instant::now();
    let mut count = 0;
    while let Ok(line) = reader.next_line() {
        count += 1;
    }

    let elapsed = start.elapsed();
    println!("LineReader: {} lines in {}", count, (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0));
}

fn try_lines_read_until(filename: &str) {
    let infile = File::open(filename).expect("open");
    let mut infile = BufReader::new(infile);

    let start = Instant::now();
    let mut count = 0;
    let mut line: Vec<u8> = Vec::with_capacity(128);
    while infile.read_until(b'\n', &mut line).unwrap_or(0) > 0 {
        count += 1;
        line.clear();
    }

    let elapsed = start.elapsed();
    println!("read_until: {} lines in {}", count, (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0));
}

fn try_lines_iter(filename: &str) {
    let infile = File::open(filename).expect("open");
    let infile = BufReader::new(infile);

    let start = Instant::now();
    let mut count = 0;
    for line in infile.lines() {
        count += 1;
    }

    let elapsed = start.elapsed();
    println!("lines(): {} lines in {}", count, (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0));
}

const TESTFILE: &str = "/dump/wordlists/pwned-passwords-2.0.txt";

fn main() {
    try_linereader(TESTFILE);
    try_lines_iter(TESTFILE);
    try_lines_read_until(TESTFILE);
}
