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
use std::fmt::{Display, Formatter};
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_parser = format_to_vec)]
    pos_format: Vec<Vec<Fragment>>,
    #[arg(short, long, default_value = "english", value_delimiter = ',')]
    /// Languages to find words in
    language: Vec<String>,
    #[arg(short, long)]
    /// Update the selected <LANGUAGE> from a line-delimited file found at <UPDATE>
    update: Option<PathBuf>,
}

fn get_words_of_len<'a>(length: usize, words: &[&'a str]) -> Vec<&'a str> {
    words
        .iter()
        .filter(|&&word| word.len() == length)
        .copied()
        .collect()
}

// these take into account words containing apostrophes (that's -> 4'1)
// and dashes (mind-blown -> 4-5)
const VALID_CHARS: [char; 13] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '\'', '_',
];

fn format_to_vec(s: &str) -> Result<Vec<Vec<Fragment>>, String> {
    if s.chars().all(|c| VALID_CHARS.contains(&c)) {
        Ok(s.split('_')
            .map(|s| {
                s.chars()
                    .map(|c| match c {
                        '\'' => Fragment::Apostrophe,
                        '-' => Fragment::Dash,
                        _ => Fragment::Letter(None),
                    })
                    .collect()
            })
            .collect())
    } else {
        Err("Invalid character in string".to_string())
    }
}

#[derive(Copy, Clone, Debug)]
enum Fragment {
    Letter(Option<char>),
    Dash,
    Apostrophe,
}

impl Display for Fragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Letter(c) => write!(f, "{}", c.unwrap_or('_')),
            Self::Apostrophe => write!(f, "'"),
            Self::Dash => write!(f, "-"),
        }
    }
}

#[derive(Debug)]
struct Word(Vec<Fragment>);

impl Display for Word {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in &self.0 {
            match write!(f, "{c}") {
                Ok(()) => continue,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

impl From<&str> for Word {
    fn from(value: &str) -> Self {
        let mut res = Vec::new();

        for c in value.chars() {
            match c {
                '\'' => res.push(Fragment::Apostrophe),
                '-' => res.push(Fragment::Dash),
                c => {
                    (0..c.to_digit(10).unwrap()).for_each(|_| res.push(Fragment::Letter(None)));
                }
            }
        }

        Self(res)
    }
}

fn update(path: &Path, lang: &str) -> Result<(), io::Error> {
    Ok(())
}

fn main() {
    let args = Args::parse();

    println!("{args:?}");
    println!("{}", Word::from("2-7'1"));
}