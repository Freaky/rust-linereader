use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::str;
use std::time::{Duration, Instant};

extern crate linereader;
use linereader::LineReader;
extern crate memchr;

use memchr::Memchr;

const BUFFER_SIZE: usize = 1024 * 64;

struct Report {
    lines: u64,
    bytes: u64,
}

impl Report {
    fn new(filename: &str) -> Self {
        let mut infile = File::open(filename).expect("open");

        let mut lines = 0_u64;
        let mut bytes = 0_u64;
        let mut buf = [0; BUFFER_SIZE];
        while let Ok(r) = infile.read(&mut buf[..]) {
            if r == 0 {
                break;
            }
            bytes += r as u64;
            lines += Memchr::new(b'\n', &buf[..r]).count() as u64;
        }

        println!("File: {}, bytes: {}, lines: {}", filename, bytes, lines);

        println!(
            "| {:16} | {:^7} | {:^11} | {:^13} |",
            "Method", "Time", "Lines/sec", "Bandwidth"
        );
        println!("|------------------|--------:|------------:|--------------:|");
        Self { lines, bytes }
    }

    fn report(&self, name: &str, bytes: Option<u64>, lines: Option<u64>, elapsed: Duration) {
        if let Some(bytes) = bytes {
            if bytes != self.bytes {
                println!("Warning: expected {} bytes, read {}", self.bytes, bytes);
            }
        }

        if let Some(lines) = lines {
            if lines != self.lines {
                println!("Warning: expected {} lines, read {}", self.lines, lines);
            }
        }

        let elapsed =
            (elapsed.as_secs() as f64) + (f64::from(elapsed.subsec_nanos()) / 1_000_000_000.0);
        println!(
            "| {:16} | {: >6.2}s | {:>9.0}/s | {:>8.2} MB/s |",
            name,
            elapsed,
            (self.lines as f64) / elapsed,
            ((self.bytes as f64) / elapsed) / (1024.0 * 1024.0)
        );
    }
}

fn try_baseline(report: &Report, filename: &str) {
    let mut infile = File::open(filename).expect("open");

    let start = Instant::now();
    let mut bytes = 0_u64;

    let mut buf = [0; BUFFER_SIZE];
    while let Ok(r) = infile.read(&mut buf[..]) {
        if r == 0 {
            break;
        }
        bytes += r as u64;
    }

    report.report("read()", Some(bytes), None, start.elapsed());
}

fn try_linereader_batch(report: &Report, filename: &str) {
    let infile = File::open(filename).expect("open");

    let mut reader = LineReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut bytes = 0_u64;
    while let Some(batch) = reader.next_batch() {
        let batch = batch.unwrap();
        bytes += batch.len() as u64;
    }

    report.report("LR::next_batch()", Some(bytes), None, start.elapsed());
}

fn try_linereader(report: &Report, filename: &str) {
    let infile = File::open(filename).expect("open");

    let mut reader = LineReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut lines = 0_u64;
    let mut bytes = 0_u64;
    while let Some(line) = reader.next_line() {
        bytes += line.unwrap().len() as u64;
        lines += 1;
    }

    report.report("LR::next_line()", Some(bytes), Some(lines), start.elapsed());
}

fn try_read_until(report: &Report, filename: &str) {
    let infile = File::open(filename).expect("open");
    let mut infile = BufReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut lines = 0_u64;
    let mut bytes = 0_u64;
    let mut line: Vec<u8> = Vec::with_capacity(128);
    while infile.read_until(b'\n', &mut line).unwrap_or(0) > 0 {
        bytes += line.len() as u64;
        lines += 1;
        line.clear();
    }

    report.report("read_until()", Some(bytes), Some(lines), start.elapsed());
}

fn try_read_line(report: &Report, filename: &str) {
    let infile = File::open(filename).expect("open");
    let mut infile = BufReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut lines = 0_u64;
    let mut bytes = 0_u64;
    let mut line = String::new();
    while infile.read_line(&mut line).unwrap_or(0) > 0 {
        bytes += line.len() as u64;
        lines += 1;
        line.clear();
    }

    report.report("read_line()", Some(bytes), Some(lines), start.elapsed());
}

fn try_lines_iter(report: &Report, filename: &str) {
    let infile = File::open(filename).expect("open");
    let infile = BufReader::with_capacity(BUFFER_SIZE, infile);

    let start = Instant::now();
    let mut lines = 0_u64;
    for _line in infile.lines() {
        lines += 1;
    }

    report.report("lines()", None, Some(lines), start.elapsed());
}

fn main() {
    use std::env;

    for file in env::args().skip(1) {
        let report = Report::new(&file);
        try_baseline(&report, &file);
        try_linereader_batch(&report, &file);
        try_linereader(&report, &file);
        try_read_until(&report, &file);
        try_read_line(&report, &file);
        try_lines_iter(&report, &file);
    }
}
