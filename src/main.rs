use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::str;
use std::time::{Duration, Instant};

extern crate linereader;
use linereader::LineReader;

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
    while let Ok(r) = infile.read(&mut buf[..]) {
        if r == 0 {
            break;
        }
        bytes += r;
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

fn main() {
    use std::env;
    for file in env::args().skip(1) {
        try_baseline(&file);
        try_linereader(&file);
        try_read_until(&file);
        try_read_line(&file);
        try_lines_iter(&file);
    }
}
