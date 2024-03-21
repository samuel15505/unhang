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

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    format: String,
    language: Option<String>,
    update: Option<bool>,
}

fn get_words_of_len<'a>(length: usize, words: &[&'a str]) -> Vec<&'a str> {
    words.iter().filter(|&&word| word.len() == length).copied().collect()
}

fn main() {
    let args = Args::parse();
    
    println!("{args:?}");
}
