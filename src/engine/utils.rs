use crate::engine::words::dictionary_words;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

use crate::word_list_manager::{WordEntry, WordListManager};

const WORD_LIST_PATH: &str = "wotd_words.json";

/// Creates a dictionary from the words defined in words.rs
pub fn dictionary() -> HashSet<String> {
    let mut dict = HashSet::new();
    for word in dictionary_words() {
        dict.insert(word);
    }
    dict
}

/// Get random WordEntry by length from dataset
pub fn get_random_word_by_length(length: usize) -> WordEntry {
    let manager = WordListManager::new(WORD_LIST_PATH)
        .expect("Failed to load word list");

    manager
        .get_random_by_length(length)
        .expect("No word found for requested length")
}

/// Original version (keeping until tests can be updated)
pub fn get_random_word() -> String {
    let dict = dictionary();
    let list = Vec::from_iter(dict.iter());
    list.choose(&mut rand::thread_rng())
        .unwrap()
        .to_string()
}

/// Maps the letter in a word to the count of the letter found in the word
pub fn build_letter_counts(word: &str) -> HashMap<char, usize> {
    let mut counts = HashMap::new();

    for character in word.chars() {
        // If character is a key of counts
        // then increment its value count by one
        // else insert character as a new key with an initial count of one
        match counts.get_mut(&character) {
            Some(count) => *count += 1,
            None => {
                counts.insert(character, 1);
            }
        };
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*; // import names of outer scope

    #[test]
    fn test_dictionary() {
        let dict = dictionary();
        assert!(!dict.is_empty())
    }

    #[test]
    fn test_get_random_word() {
        let word = get_random_word();
        assert!(!word.is_empty());

        for character in word.chars() {
            assert!(character.is_alphabetic())
        }
    }

    #[test]
    fn test_build_letter_counts() {
        let word = "aaaabbc";
        let character_counts = build_letter_counts(&word);

        for character in word.chars() {
            let character_count = word.chars().filter(|&c| c == character).count();

            match character_counts.get(&character) {
                Some(count) => assert_eq!(*count, character_count),
                None => assert!(false),
            }
        }
    }

    #[test]
    fn test_build_letter_counts_no_word() {
        let word = "";
        let character_counts = build_letter_counts(&word);
        assert!(character_counts.is_empty());
    }
}