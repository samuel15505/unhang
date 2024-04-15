#![warn(
    clippy::correctness,
    clippy::nursery,
    clippy::pedantic,
    clippy::perf,
    clippy::style,
    clippy::suspicious
)]

//! A hangman solver. Simple as.

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::str::FromStr;

use clap::Parser;
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(value_parser = get_hangman)]
    pos_format: Hangman,
    #[arg(short, long, default_value = "english", value_delimiter = ',')]
    /// Languages to find words in
    language: Vec<String>,
    #[arg(short, long, value_delimiter = ',')]
    /// Update the selected language from a line-delimited file found at designated path
    update: Option<Vec<PathBuf>>,
}

fn get_hangman(s: &str) -> Result<Hangman, String> {
    Hangman::try_from(s).map_err(|_| "invalid character in format string".to_string())
}

// these take into account words containing apostrophes (that's -> 4'1)
// and dashes (mind-blown -> 4-5)
const VALID_CHARS: [char; 13] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '-', '\'', '_',
];

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Fragment {
    Letter(Option<char>),
    Punctuation(char),
}

impl Display for Fragment {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Letter(c) => write!(f, "{}", c.unwrap_or('_')),
            Self::Punctuation(c) => write!(f, "{c}"),
        }
    }
}

impl PartialEq<char> for Fragment {
    fn eq(&self, other: &char) -> bool {
        match self {
            Self::Punctuation(c) => other == c,
            Self::Letter(Some(letter)) => other == letter,
            Self::Letter(None) => other.is_alphanumeric(),
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct Word(Vec<Fragment>);

#[derive(Debug, Eq, PartialEq)]
struct WordParseError;

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
        if self.len() != other.len() {
            return false;
        }
        for (i, ch) in other.chars().enumerate() {
            if self[i] != ch {
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

impl FromStr for Word {
    type Err = WordParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = Vec::new();

        for c in s.chars() {
            match c {
                '_' => res.push(Fragment::Letter(None)),
                c if c.is_alphanumeric() => res.push(Fragment::Letter(Some(c))),
                c => res.push(Fragment::Punctuation(c)),
            }
        }

        Ok(Self(res))
    }
}

impl Word {
    fn add_letter(&mut self, letter: char, positions: &[usize]) -> Result<(), &'static str> {
        for pos in positions {
            // self[*pos] = Fragment::Letter(Some(letter));
            match self[*pos] {
                Fragment::Letter(_) => self[*pos] = Fragment::Letter(Some(letter)),
                _ => return Err("can't change a dash or apostrophe"),
            }
        }

        Ok(())
    }

    fn from_format_string(value: &str) -> Self {
        let mut res = Vec::new();

        for c in value.chars() {
            assert!(VALID_CHARS.contains(&c));
            match c {
                c if c.is_digit(10) => {
                    (0..c.to_digit(10).unwrap_or_default())
                        .for_each(|_| res.push(Fragment::Letter(None)));
                }
                c => res.push(Fragment::Punctuation(c)),
            }
        }

        Self(res)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Default)]
struct Hangman {
    words: Vec<Word>,
}

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

        Ok(Self {
            words: value.split('_').map(Word::from_format_string).collect(),
        })
    }
}

impl Deref for Hangman {
    type Target = Vec<Word>;

    fn deref(&self) -> &Self::Target {
        &self.words
    }
}

impl DerefMut for Hangman {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.words
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

#[derive(Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
struct LangDict(HashMap<String, HashSet<char>>);

#[derive(Debug, Eq, PartialEq)]
struct LangDictParseError {
    source: InvalidCharError,
}

impl Display for LangDictParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to parse file into LangDict")
    }
}

impl Error for LangDictParseError {}

#[derive(Debug, Eq, PartialEq)]
struct InvalidCharError {
    char: char,
}

impl Display for InvalidCharError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unexpected char {} found", self.char)
    }
}

impl Error for InvalidCharError {}

impl FromStr for LangDict {
    type Err = LangDictParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut res = Self::new();

        for line in s.lines() {
            let mut set: HashSet<char> = HashSet::new();
            for char in line.to_lowercase().chars() {
                if char.is_alphanumeric() || "\'-&,./!".contains(char) {
                    set.insert(char);
                } else {
                    return Err(LangDictParseError {
                        source: InvalidCharError { char },
                    });
                }
            }
            res.insert(line.to_lowercase().to_string(), set);
        }

        Ok(res)
    }
}

impl LangDict {
    fn new() -> Self {
        Self::default()
    }

    fn get_matching(&self, value: &Word) -> Self {
        let mut res = Self::new();

        for (key, val) in self.iter() {
            if key.as_str() == value {
                res.insert(key.clone(), val.clone());
            }
        }

        res
    }

    fn remove_with_letter(&mut self, val: char) {
        self.retain(|k, _| !k.contains(val));
        self.shrink_to_fit();
    }

    fn rank_letters(&self) -> Vec<char> {
        let res = self.count_letters();
        let mut res: Vec<_> = res.into_iter().collect();
        res.sort_by(|a, b| b.1.cmp(&a.1));
        res.iter().map(|&(ch, _)| ch).collect()
    }

    fn count_letters(&self) -> HashMap<char, i32> {
        let mut res = HashMap::new();
        for entry in self.values() {
            for &char in entry {
                res.entry(char)
                    .and_modify(|e| *e += 1)
                    .or_insert(1);
            }
        }
        res
    }
}

impl Deref for LangDict {
    type Target = HashMap<String, HashSet<char>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for LangDict {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn main() {
    let args = Args::parse();
    println!("{args:?}");
    let mut langdicts = Vec::new();
    if let Some(path_vec) = args.update {
        for (i, lang) in args.language.iter().enumerate() {
            let text = fs::read_to_string(&path_vec[i]).unwrap();
            langdicts.push(LangDict::from_str(&text).unwrap());
            let path: PathBuf = ["data", &format!("{lang}.ron")].iter().collect();
            fs::write(path, ron::to_string(langdicts.last().unwrap()).unwrap()).unwrap();
        }
    } else {
        for lang in args.language {
            let path: PathBuf = ["data", &format!("{lang}.ron")].iter().collect();
            let text = fs::read_to_string(path).unwrap();
            langdicts.push(ron::from_str(&text).unwrap());
        }
    };
    let mut word = args.pos_format[0].clone();
    let mut matches = langdicts[0].get_matching(&word);
    let mut attempts = Vec::new();
    loop {
        let mut ranked = matches.rank_letters();
        println!("{ranked:?}");
        let mut suggestion = None;

        while let Some(char) = ranked.pop() {
            if !&attempts.contains(&char) {
                suggestion = Some(char);
            }
        }
        if let Some(char) = suggestion {
            println!("word is {word}");
            println!("best option: {char}");
            println!("enter your guess here as (char,(pos,pos,pos))");
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            let (char, pos) = buf.trim().split_once(',').unwrap();
            let char = char.chars().next().unwrap();
            attempts.push(char);
            let pos: Vec<_> = pos
                .strip_prefix('(')
                .unwrap()
                .strip_suffix(')')
                .unwrap()
                .split(',')
                .filter(|e| !e.is_empty())
                .map(|e| e.parse().unwrap())
                .collect();
            if pos.is_empty() {
                matches.remove_with_letter(char);
            };
            word.add_letter(char, &pos).unwrap();
            println!("{:?}", matches.count_letters());
            matches = dbg!(matches.get_matching(&word));
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frag_eq() {
        assert_eq!(Fragment::Letter(Some('c')), 'c');
        assert_eq!(Fragment::Letter(None), 'c');
        assert_ne!(Fragment::Punctuation('-'), 'a');
    }

    #[test]
    fn word_eq() {
        assert_eq!(&Word::from_str("____").unwrap(), "test");
        assert_ne!(&Word::from_str("__-_").unwrap(), "test");
        assert_ne!(&Word::from_str("___").unwrap(), "ad-");
    }

    #[test]
    fn word_from() {
        let word = vec![Fragment::Letter(None); 3];
        assert_eq!(&Word::from_str("___").unwrap(), &Word(word));
    }

    #[test]
    fn lang_dict_parse() {
        let count = ['f', 'i', 'm', 'n', 'u'];
        let count: HashSet<char> = HashSet::from(count);
        let lang_dict = HashMap::from([("muffin".to_string(), count)]);
        assert_eq!(LangDict::from_str("muffin").unwrap(), LangDict(lang_dict));
    }
}
