use crate::engine::words::dictionary_words;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

/// Creates a dictionary from the words defined in words.rs
/// Iterates over all words and inserts them
pub fn dictionary() -> HashSet<String> {
    let mut dict = HashSet::new();
    for word in dictionary_words() {
        dict.insert(word);
    }
    dict
}

/// Select random word from the dictionary
pub fn get_random_word() -> String {
    let dict = dictionary();
    let list = Vec::from_iter(dict.iter());
    list.choose(&mut rand::thread_rng())
        .unwrap() // Use of this function is discouraged it may cause panic - Jeff
        .to_string()
}

/// Maps the letter in a word to the count of the letter found in the word
pub fn build_letter_counts(word: &str) -> HashMap<char, usize> {
    let mut counts = HashMap::new();

    for character in word.chars() {
        // If character is a key of counts
        // then increment it's value count by one
        // else insert character as a new key with an initial count of one
        match counts.get_mut(&character) {
            Some(count) => *count += 1,
            None => {
                counts.insert(character, 1);
                ()
            }
        };
    }

    counts
}

#[cfg(test)]
mod tests {
    use super::*; // import names of outer scope

    #[test]
    /// Test that dictionary functions as expected
    /// not doing much since I expect this to change
    fn test_dictionary() {
        let dict = dictionary();

        // This is true as long as the dictionary is not empty
        assert!(!dict.is_empty())
    }

    #[test]
    /// Test that get_random_word provides a valid word
    fn test_get_random_word() {
        let word = get_random_word();

        assert!(!word.is_empty()); // word can't be emtpy

        // Verify word is made of alphabetic characters
        for character in word.chars() {
            assert!(character.is_alphabetic())
        }
    }

    #[test]
    /// Test that build_letter_counts properly counts letters of word
    fn test_build_letter_counts() {
        let word = "aaaabbc";
        let character_counts = build_letter_counts(&word);

        for character in word.chars() {
            // filter word for current character and count it
            let character_count = word.chars().filter(|&c| c == character).count();

            match character_counts.get(&character) {
                Some(count) => assert_eq!(*count, character_count),
                _ => assert!(false),
            }
        }
    }

    #[test]
    /// Test that build_letter_counts properly counts letters of word
    fn test_build_letter_counts_no_word() {
        let word = "";
        let character_counts = build_letter_counts(&word);

        assert!(character_counts.is_empty());
    }
}
