#![warn(
    clippy::correctness,
    clippy::nursery,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::suspicious,
)]

//! A hangman solver. Simple as.
//! Positional argument is words format, in format {word 1 len}-{word 2 len}-...-{word n len}.
//! Arguments are:
//! -f, --format <WORD FORMAT>  Format of the words, in format L1-L2-...-Ln.
//! -l, --language <LANGUAGE>   Language to solve in.
//! -u, --update                Update the language file.
//! -h, --help                  Print help.

use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Instant;

fn get_words_of_len<'a>(length: usize, words: &[&'a str]) -> Vec<&'a str> {
    words.iter().filter(|&&word| word.len() == length).copied().collect()
}

fn main() {
    let words_path: PathBuf = ["data", "words.txt"].iter().collect();
    let file = read_to_string(words_path).unwrap();
    let words: Vec<_> = file.lines().collect();
    
    let now = Instant::now();
    let lens = get_words_of_len(5, &words);
    let elapsed = now.elapsed();
    
    println!("{} {:?} in {:?}", &lens.len(), &lens[0..10], elapsed);
}
