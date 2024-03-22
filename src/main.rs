#![warn(
    clippy::correctness,
    clippy::nursery,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::suspicious
)]

//! A hangman solver. Simple as.
//! Positional argument is words format, in format {word 1 len}-{word 2 len}-...-{word n len}.
//! Arguments are:
//! -f, --format <WORD FORMAT>  Format of the words, in format L1-L2-...-Ln.
//! -l, --language <LANGUAGE>   Language to solve in.
//! -u, --update                Update the language file.
//! -h, --help                  Print help.

use clap::builder::Str;
use clap::Parser;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    pos_format: Option<String>,
    #[arg(short, long, default_value = "english")]
    language: String,
    #[arg(short, long)]
    update: bool,
}

fn get_words_of_len<'a>(length: usize, words: &[&'a str]) -> Vec<&'a str> {
    words
        .iter()
        .filter(|&&word| word.len() == length)
        .copied()
        .collect()
}

const VALID_CHARS: [char; 11] = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-'];
fn format_to_vec(s: &str) -> Result<Vec<u8>, String> {
     if s.chars().all(|c| VALID_CHARS.contains(&c)) {
         Ok(s.split('-').map(|c| c.parse().unwrap()).collect())
     } else { 
         Err("Invalid character in string".to_string())
     }
}

fn main() -> Result<(), &'static str> {
    let args = Args::parse();

    println!("{args:?}");

    Ok(())
}
