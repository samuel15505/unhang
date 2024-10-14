#![warn(clippy::pedantic, clippy::nursery)]

use std::collections::HashMap;
// use std::fs;
use std::env;
use std::io;

const BLANK_CHAR: char = '_';
const WORDS: &str = include_str!("words.txt");

trait HangCompare<T> {
    fn compare(&self, rhs: T) -> bool;
}

impl HangCompare<&str> for &str {
    fn compare(&self, rhs: &str) -> bool {
        if self.len() == rhs.len() {
            self.chars()
                .zip(rhs.chars())
                .all(|(a, b)| a == b || a == BLANK_CHAR || b == BLANK_CHAR)
        } else {
            false
        }
    }
}

impl HangCompare<&Vec<Option<char>>> for &str {
    fn compare(&self, rhs: &Vec<Option<char>>) -> bool {
        if self.len() == rhs.len() {
            self.chars()
                .zip(rhs.iter())
                .all(|(a, b)| Some(a) == *b || a == BLANK_CHAR || b.is_none())
        } else {
            false
        }
    }
}

fn main() {
    let mut map = HashMap::new();

    for word in WORDS.lines() {
        let mut letters = HashMap::new();

        for c in word.chars() {
            letters
                .entry(c.to_ascii_lowercase())
                .and_modify(|e| *e += 1)
                .or_insert(1);
        }

        map.insert(word.to_lowercase(), letters);
    }

    let word_length = env::args()
        .nth(1)
        .expect("argument in pos 1")
        .parse()
        .expect("numeric argument");
    let mut word = vec![None; word_length];
    let mut missed = Vec::new();
    let mut buf = String::new();

    map.retain(|word, _| word.len() == word_length);

    while !word.iter().all(Option::is_some) {
        let current_words = map
            .iter()
            .filter(|(key, _)| {
                key.as_str().compare(&word) && missed.iter().all(|&c| !key.contains(c))
            })
            .map(|(_, v)| v)
            .collect::<Vec<_>>();
        let mut counts = HashMap::new();
        for count in current_words {
            for (c, num) in count {
                if !word.contains(&Some(*c)) {
                    counts.entry(*c).and_modify(|e| *e += *num).or_insert(*num);
                }
            }
        }
        let mut counts: Vec<_> = counts.into_iter().collect();
        counts.sort_by(|(_, lhs), (_, rhs)| lhs.cmp(rhs));
        // counts.reverse();
        println!(
            "{:?}",
            counts
                .iter()
                .map(|(c, _)| c)
                .rev()
                .take(3)
                .collect::<Vec<_>>()
        );
        buf.clear();
        for letter in &word {
            print!("{}", letter.unwrap_or(BLANK_CHAR));
        }
        println!();
        for i in 0..word_length {
            print!("{i}");
        }
        println!("\nEnter a letter, followed by positions");
        io::stdin().read_line(&mut buf).unwrap();
        let parts = buf.trim().split(' ').collect::<Vec<&str>>();
        if parts.is_empty() {
            println!("Not enough parts");
            continue;
        }

        let letter = parts[0].to_string().chars().next().unwrap();
        let numbers: Vec<usize> = parts
            .into_iter()
            .skip(1)
            .map(|e| e.parse().unwrap())
            .collect::<Vec<_>>();

        if numbers.is_empty() {
            missed.push(letter);
            continue;
        }

        for position in numbers {
            word[position] = Some(letter);
        }
    }

    print!(
        "The word was: {}",
        word.iter()
            .map(|c| c.expect("some character"))
            .collect::<String>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compare_str() {
        assert!("t__t".compare("test"));
        assert!(!"t__t".compare("max"));
        assert!(!"t__t".compare("naur"));
    }

    #[test]
    fn test_compare_vec() {
        assert!("test".compare(&vec![Some('t'), None, None, Some('t')]));
        assert!(!"test".compare(&vec![Some('t'), None, Some('t')]));
        assert!(!"test".compare(&vec![Some('s'), None, None, Some('t')]));
    }
}
