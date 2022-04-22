use std::collections::HashSet;

use crate::words::{EXTRA_WORDS, VALID_WORDS};
use lazy_static::lazy_static;

lazy_static! {
    static ref EXTRA_WORDS_CHARS: Vec<PreparsedWord> = must_convert_list_to_char_list(EXTRA_WORDS);
    static ref VALID_WORDS_CHARS: Vec<PreparsedWord> = must_convert_list_to_char_list(VALID_WORDS);
}

pub struct PreparsedWord {
    word: [char; 5],
    chars: HashSet<char>,
}

impl PreparsedWord {
    fn new(word: [char; 5]) -> PreparsedWord {
        let mut wordletters: HashSet<char> = HashSet::new();
        for i in 0..5 {
            wordletters.insert(word[i]);
        }
        PreparsedWord {
            word,
            chars: wordletters,
        }
    }
}

pub fn must_convert_list_to_char_list(words: &'static [&'static str]) -> Vec<PreparsedWord> {
    words
        .into_iter()
        .map(|word| {
            let ch = wordle_word_to_char_array(*word).unwrap();
            PreparsedWord::new(ch)
        })
        .collect()
}

#[derive(Clone)]
struct Position {
    found_letter: Option<char>,
    invalid_letters: HashSet<char>,
}

impl Position {
    fn new() -> Position {
        Position {
            found_letter: None,
            invalid_letters: HashSet::new(),
        }
    }
}

struct Validator {
    wordleword: PreparsedWord,
    letter_positions: [Position; 5],
    missed_letters: HashSet<char>,
    wrong_pos_letters: HashSet<char>,
}

impl Validator {
    fn new(wordleword: [char; 5]) -> Validator {
        Validator {
            wordleword: PreparsedWord::new(wordleword),
            letter_positions: [
                Position::new(),
                Position::new(),
                Position::new(),
                Position::new(),
                Position::new(),
            ],
            missed_letters: HashSet::new(),
            wrong_pos_letters: HashSet::new(),
        }
    }

    fn injest_word(&mut self, word: [char; 5]) {
        for i in 0..5 {
            let letter = word[i];
            if letter == self.wordleword.word[i] {
                // MATCH!
                self.letter_positions[i].found_letter = Some(letter)
            } else if self.wordleword.chars.contains(&letter) {
                // MISS BUT IN WORD!
                self.letter_positions[i].invalid_letters.insert(letter);
                self.wrong_pos_letters.insert(letter);
            } else {
                // COMPLETE MISS!
                self.missed_letters.insert(letter);
            }
        }
    }

    fn valid_for_word(&self, word: &PreparsedWord) -> bool {
        // Filter out words with letters we know are _not_ in the word
        for ch in self.missed_letters.iter() {
            if word.chars.contains(ch) {
                return false;
            }
        }

        // Filter out words that don't have the letters we know are in the word
        for ch in self.wrong_pos_letters.iter() {
            if !word.chars.contains(ch) {
                return false;
            }
        }

        // Filter out individual letters positions
        for i in 0..5 {
            let letter = word.word[i];
            // Check if the letter matches our _exact_ need
            if let Some(found_letter) = self.letter_positions[i].found_letter {
                if found_letter != letter {
                    return false;
                }
            }

            // Check if the letter is invalid based on previous guesses
            if self.letter_positions[i].invalid_letters.contains(&letter) {
                return false;
            }
        }
        true
    }
}

pub fn parse_words_list(words: &str) -> anyhow::Result<Vec<[char; 5]>> {
    let splitwords: Vec<_> = words.split("\n").collect();
    splitwords
        .into_iter()
        .map(|word| wordle_word_to_char_array(word))
        .collect()
}

// Taking in a list of wordle words, calculate how many "valid" guesses were possible
// at each step.
pub fn calculate_word_possibilities(words: &mut Vec<[char; 5]>) -> anyhow::Result<Vec<(u32, u32)>> {
    let wordleword = words
        .pop()
        .ok_or_else(|| anyhow::anyhow!("wordle words passed in!"))?;

    let mut validator = Validator::new(wordleword);

    let mut num_word_chances: Vec<(u32, u32)> = vec![];
    for word in words {
        validator.injest_word(*word);
        let num_valid_words = (*VALID_WORDS_CHARS)
            .iter()
            .filter(|word| validator.valid_for_word(*word))
            .count() as u32;
        let num_extra_words = (*EXTRA_WORDS_CHARS)
            .iter()
            .filter(|word| validator.valid_for_word(*word))
            .count() as u32;
        num_word_chances.push((num_valid_words, num_valid_words + num_extra_words))
    }

    Ok(num_word_chances)
}

pub fn wordle_word_to_char_array(word: &str) -> anyhow::Result<[char; 5]> {
    if word.len() != 5 {
        return Err(anyhow::anyhow!(
            "word needs to be 5 characters, got {}",
            word
        ));
    }
    let chars = word.to_lowercase();
    let mut arr = ['a'; 5];
    for (i, ch) in chars.chars().enumerate() {
        arr[i] = ch;
    }
    Ok(arr)
}

#[cfg(test)]
mod tests {
    use super::{calculate_word_possibilities, parse_words_list};

    #[test]
    fn test_calculate_word_possibilities() {
        let mut words = parse_words_list(
            "train
weigh
slide
oxide",
        )
        .unwrap();
        let pos = calculate_word_possibilities(&mut words).unwrap();
        println!("{:?}", pos)
    }
}
