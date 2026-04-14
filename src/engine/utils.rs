use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};
use std::fs;

use crate::word_list_manager::{WordEntry, WordListManager};

const WORD_LIST_PATH: &str = "wotd_words.json";
const EXTENDED_WORDS_PATH: &str = "extended_words.csv";

/// Creates a dictionary by combining
/// words from the Wiktionary WOTD JSON dataset
/// and words from extended_words.csv (one lowercase word per line)
pub fn dictionary() -> HashSet<String> {
    let mut dict = HashSet::new();

    // Load words from the JSON word list.
    if let Ok(manager) = WordListManager::new(WORD_LIST_PATH) {
        for entry in manager.all_entries() {
            dict.insert(entry.word.clone());
        }
    }

    // Load extra words from the plaintext CSV file (one word per line).
    if let Ok(raw) = fs::read_to_string(EXTENDED_WORDS_PATH) {
        for line in raw.lines() {
            let word = line.trim();
            if !word.is_empty() {
                dict.insert(word.to_string());
            }
        }
    }

    dict
}

/// Get a random WordEntry by length from the JSON dataset.
pub fn get_random_word_by_length(length: usize) -> WordEntry {
    let manager = WordListManager::new(WORD_LIST_PATH).expect("Failed to load word list");

    manager
        .get_random_by_length(length)
        .expect("No word found for requested length")
}

/// Get the most recent WordEntry from the JSON dataset.
pub fn get_most_recent_word() -> WordEntry {
    let manager = WordListManager::new(WORD_LIST_PATH).expect("Failed to load word list");

    manager
        .get_most_recent()
        .expect("Could not get most recent word")
}

/// Original version (keeping until tests can be updated)
pub fn get_random_word() -> String {
    let dict = dictionary();
    let list = Vec::from_iter(dict.iter());
    list.choose(&mut rand::thread_rng()).unwrap().to_string()
}

/// Maps the letter in a word to the count of the letter found in the word.
pub fn build_letter_counts(word: &str) -> HashMap<char, usize> {
    let mut counts = HashMap::new();

    for character in word.chars() {
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
    use rstest::*;

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
