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

use clap::Parser;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::{fs, io};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_parser = get_hangman)]
    pos_format: Hangman,
    #[arg(short, long, default_value = "english", value_delimiter = ',')]
    /// Languages to find words in
    language: Vec<String>,
    #[arg(short, long)]
    /// Update the selected <LANGUAGE> from a line-delimited file found at <UPDATE>
    update: Option<PathBuf>,
}

fn get_hangman(s: &str) -> Result<Hangman, String> {
    Hangman::try_from(s).map_err(|_| "invalid character in format string".to_string())
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

impl PartialEq<char> for Fragment {
    fn eq(&self, other: &char) -> bool {
        match self {
            Self::Apostrophe => other == &'\'',
            Self::Dash => other == &'-',
            Self::Letter(None) => true,
            Self::Letter(Some(letter)) => other == letter,
        }
    }
}

impl PartialEq<Fragment> for char {
    fn eq(&self, other: &Fragment) -> bool {
        // match other {
        //     Fragment::Apostrophe => self == &'\'',
        //     Fragment::Dash => self == &'-',
        //     Fragment::Letter(None) => true,
        //     Fragment::Letter(Some(letter)) => letter == other,
        // }
        other == self
    }
}

#[derive(Clone, Debug)]
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

impl PartialEq<str> for Word {
    fn eq(&self, other: &str) -> bool {
        for (i, ch) in other.chars().enumerate() {
            if self[i] == ch {
                return false;
            }
        }

        true
    }
}

impl PartialEq<Word> for str {
    fn eq(&self, other: &Word) -> bool {
        other == self
    }
}

impl Deref for Word {
    type Target = Vec<Fragment>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Word {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<&str> for Word {
    fn from(value: &str) -> Self {
        let mut res = Vec::new();

        for c in value.chars() {
            assert!(VALID_CHARS.contains(&c));
            match c {
                '\'' => res.push(Fragment::Apostrophe),
                '-' => res.push(Fragment::Dash),
                c => {
                    (0..c.to_digit(10).unwrap_or_default())
                        .for_each(|_| res.push(Fragment::Letter(None)));
                }
            }
        }

        Self(res)
    }
}

impl Word {
    fn add_letter(&mut self, letter: char, positions: &[usize]) -> Result<(), &'static str> {
        for pos in positions {
            self[*pos] = Fragment::Letter(Some(letter));
            match self[*pos] {
                Fragment::Letter(None) => self[*pos] = Fragment::Letter(Some(letter)),
                _ => return Err("can't change a dash or apostrophe"),
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
struct Hangman(Vec<Word>);

impl Hangman {}

// impl From<&str> for Hangman {
//     fn from(value: &str) -> Self {
//         for c in value.chars() {
//             assert!(VALID_CHARS.contains(&c));
//         }
//         Self(value.split('_').map(Word::from).collect())
//     }
// }

#[derive(Debug, Clone)]
struct InvalidChar;

impl Display for InvalidChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unexpected character in source string")
    }
}

impl Error for InvalidChar {}

impl TryFrom<&str> for Hangman {
    type Error = InvalidChar;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        for c in value.chars() {
            if !VALID_CHARS.contains(&c) {
                return Err(InvalidChar);
            }
        }

        Ok(Self(value.split('_').map(Word::from).collect()))
    }
}

impl Deref for Hangman {
    type Target = Vec<Word>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Hangman {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Hangman {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for word in self.iter() {
            write!(f, "{word} ")?;
        }
        Ok(())
    }
}

#[derive(Default)]
struct LangDict(HashMap<String, HashMap<char, u8>>);

impl LangDict {
    fn new() -> Self {
        Self::default()
    }
}

impl Deref for LangDict {
    type Target = HashMap<String, HashMap<char, u8>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LangDict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn update<T>(path: T, lang: &str) -> Result<(), io::Error>
where
    T: AsRef<Path>,
{
    let mut results: HashMap<String, HashMap<char, u8>> = HashMap::new();
    let text = fs::read_to_string(path)?;
    for word in text.lines() {
        let mut counts: HashMap<char, u8> = HashMap::new();
        for char in word.to_lowercase().chars() {
            counts.entry(char).and_modify(|e| *e += 1).or_insert(1);
        }
        results.insert(word.to_lowercase().to_string(), counts);
    }

    let write_path: PathBuf = ["data", &format!("{}.ron", lang.to_lowercase())]
        .iter()
        .collect();

    fs::write(
        write_path,
        ron::to_string(&results).expect("serialization unsuccessful"),
    )?;

    Ok(())
}

fn try_load(lang: &str) -> io::Result<HashMap<String, HashMap<char, u8>>> {
    let read_path: PathBuf = ["data", &format!("{}.ron", lang.to_lowercase())]
        .iter()
        .collect();
    Ok(ron::from_str(&fs::read_to_string(read_path)?).expect("deserialization unsuccessful"))
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");
    if let Some(path) = args.update {
        update(path, &args.language[0]).unwrap();
    }
    match try_load(&args.language[0]) {
        Ok(lang) => {
            println!("{:?}", lang.get("muffin"));
        }
        Err(e) => println!("{e}"),
    }
}
