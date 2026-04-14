//! WordListManager — create, read, and edit the Wiktionary WOTD JSON word list.
//!
//! # Usage
//! ```rust
//! let mut manager = WordListManager::new("words.json")?;
//! manager.add_entry(entry)?;
//! let random = manager.get_random_by_length(5);
//! let range  = manager.get_date_range();
//! ```

use chrono::NaiveDate;
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs;
use std::path::Path;

// ─────────────────────────────────────────────────────────────────────────────
// Public types
// ─────────────────────────────────────────────────────────────────────────────

/// A single Wiktionary Word-of-the-Day record.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct WordEntry {
    /// The featured word (always lower-case).
    pub word: String,
    /// Character length of the word (unicode-aware).
    pub length: usize,
    /// The calendar date on which it was Word of the Day.
    pub date_featured: NaiveDate,
    /// Part of speech (e.g. "noun", "verb").
    pub part_of_speech: String,
    /// Primary definition text (citations stripped).
    pub definition: String,
}

impl WordEntry {
    /// Convenience constructor – computes `length` automatically.
    pub fn new(
        word: impl Into<String>,
        date_featured: NaiveDate,
        part_of_speech: impl Into<String>,
        definition: impl Into<String>,
    ) -> Self {
        let word = word.into().to_lowercase();
        let length = word.chars().count();
        WordEntry {
            word,
            length,
            date_featured,
            part_of_speech: part_of_speech.into(),
            definition: definition.into(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Error type
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum WordListError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Message(String),
}

impl fmt::Display for WordListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WordListError::Io(e) => write!(f, "I/O error: {e}"),
            WordListError::Json(e) => write!(f, "JSON error: {e}"),
            WordListError::Message(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for WordListError {}

impl From<std::io::Error> for WordListError {
    fn from(e: std::io::Error) -> Self {
        WordListError::Io(e)
    }
}

impl From<serde_json::Error> for WordListError {
    fn from(e: serde_json::Error) -> Self {
        WordListError::Json(e)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// WordListManager
// ─────────────────────────────────────────────────────────────────────────────

/// Manages the JSON word list file on disk.
///
/// Entries are kept sorted by `(length, date_featured)` at all times.
pub struct WordListManager {
    file_path: String,
    entries: Vec<WordEntry>,
}

impl WordListManager {
    // ── Construction ──────────────────────────────────────────────────────────

    /// Open (or create) a word-list file at `file_path`.
    ///
    /// If the file already exists its contents are loaded into memory;
    /// if it does not exist an empty list is prepared (the file is only
    /// written when entries are first added).
    pub fn new(file_path: &str) -> Result<Self, WordListError> {
        let entries = if Path::new(file_path).exists() {
            let raw = fs::read_to_string(file_path)?;
            serde_json::from_str::<Vec<WordEntry>>(&raw)?
        } else {
            Vec::new()
        };

        Ok(WordListManager {
            file_path: file_path.to_string(),
            entries,
        })
    }

    // ── Writing ───────────────────────────────────────────────────────────────

    /// Insert one entry.
    ///
    /// If an entry with the same `date_featured` already exists the new entry
    /// is silently ignored (returns `false`).  Otherwise the entry is added,
    /// the list is re-sorted, the file is saved, and the method returns `true`.
    pub fn add_entry(&mut self, entry: WordEntry) -> Result<bool, WordListError> {
        if self.has_date(entry.date_featured) {
            return Ok(false);
        }
        self.entries.push(entry);
        self.sort_entries();
        self.persist()?;
        Ok(true)
    }

    /// Insert a batch of entries efficiently (one file write for the whole batch).
    ///
    /// Duplicate dates are skipped.  Returns the number of entries actually added.
    pub fn add_entries(&mut self, new_entries: Vec<WordEntry>) -> Result<usize, WordListError> {
        let before = self.entries.len();
        for entry in new_entries {
            if !self.has_date(entry.date_featured) {
                self.entries.push(entry);
            }
        }
        let added = self.entries.len() - before;
        if added > 0 {
            self.sort_entries();
            self.persist()?;
        }
        Ok(added)
    }

    // ── Reading ───────────────────────────────────────────────────────────────

    /// Return a random [`WordEntry`] whose `length` equals `word_length`.
    ///
    /// Returns `None` when no matching entries exist.
    pub fn get_random_by_length(&self, word_length: usize) -> Option<WordEntry> {
        let pool: Vec<&WordEntry> = self
            .entries
            .iter()
            .filter(|e| e.length == word_length)
            .collect();
        pool.choose(&mut rand::thread_rng()).map(|e| (*e).clone())
    }

    /// Return the most recent [`WordEntry`] according to `date_featured`.
    ///
    /// Returns `None` when no entries exist.
    pub fn get_most_recent(&self) -> Option<WordEntry> {
        self
            .entries
            .iter()
            .max_by_key(|e| e.date_featured)
            .cloned()
    }

    /// Return the `(earliest_date, latest_date)` of all records on file.
    ///
    /// Returns `None` when the list is empty.
    pub fn get_date_range(&self) -> Option<(NaiveDate, NaiveDate)> {
        if self.entries.is_empty() {
            return None;
        }
        let min = self.entries.iter().map(|e| e.date_featured).min()?;
        let max = self.entries.iter().map(|e| e.date_featured).max()?;
        Some((min, max))
    }

    /// Returns `true` when the list already contains an entry for `date`.
    pub fn has_date(&self, date: NaiveDate) -> bool {
        self.entries.iter().any(|e| e.date_featured == date)
    }

    /// Returns `true` when the backing JSON file exists on disk.
    pub fn file_exists(file_path: &str) -> bool {
        Path::new(file_path).exists()
    }

    /// Total number of entries currently held.
    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Read-only slice of all entries (sorted by length, then date).
    pub fn all_entries(&self) -> &[WordEntry] {
        &self.entries
    }

    /// Entries filtered to exactly `word_length` characters.
    pub fn entries_by_length(&self, word_length: usize) -> Vec<&WordEntry> {
        self.entries
            .iter()
            .filter(|e| e.length == word_length)
            .collect()
    }

    // ── Internals ─────────────────────────────────────────────────────────────

    /// Sort entries by length ascending, then date_featured ascending.
    fn sort_entries(&mut self) {
        self.entries.sort_by(|a, b| {
            a.length
                .cmp(&b.length)
                .then_with(|| a.date_featured.cmp(&b.date_featured))
        });
    }

    /// Serialise entries to the JSON file (pretty-printed).
    fn persist(&self) -> Result<(), WordListError> {
        let json = serde_json::to_string_pretty(&self.entries)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    // #[default("Word")] word: impl Into<String>,
    // #[default("2000-01-01")] date: &str,
    // #[default("Part")] part_of_speech: impl Into<String>,
    // #[default("Definition")] definition: impl Into<String>,
    fn word_entry(
        word: impl Into<String>,
        date: &str,
        part_of_speech: impl Into<String>,
        definition: impl Into<String>,
    ) -> WordEntry {
        WordEntry::new(word, make_naive_date(date), part_of_speech, definition)
    }

    fn make_naive_date(date: &str) -> NaiveDate {
        match NaiveDate::parse_from_str(date, "%Y-%m-%d") {
            Ok(date) => date,
            Err(_error) => NaiveDate::default(),
        }
    }

    /// Create a manager, the path doesn't matter, since I create it directly from the struct definition to avoid file weirdness
    fn word_list_manager(word_entries: Vec<WordEntry>) -> WordListManager {
        // match WordListManager::new(path) {
        //     Ok(manager) => manager,
        //     Err(_error) => {
        //         assert!(false);
        //         panic!("Unable to generate WordListManager for test");
        //     }
        // }

        WordListManager {
            file_path: "test_words.json".to_string(),
            entries: word_entries,
        }
    }

    #[rstest]
    #[case::no_file_available("asdoaisfjaois.asfjas", false)] // No valid path is given, no words should be loaded
    #[case::file_given("wotd_words.json", true)] // Words are given, list manager will be populated
    #[case::invalid_file_given("Cargo.lock", false)] // read invalid file, no words will be read
    fn test_create_word_list(#[case] path: &str, #[case] expect_filled: bool) {
        match WordListManager::new(path) {
            Ok(manager) => match expect_filled {
                true => {
                    assert!(manager.entries.len() > 0);
                }
                false => {
                    assert_eq!(manager.entries.len(), 0);
                }
            },
            Err(_error) => {
                assert!(false);
                panic!("Unable to generate WordListManager for test");
            }
        }
    }

    #[rstest]
    #[case::add_one_entry(1, vec![ word_entry("Word1", "2000-01-01", "Part", "Definition") ]) ]
    #[case::add_multiple_entries(5, vec![ word_entry("Word1", "2000-01-01", "Part", "Definition"), word_entry("Word2", "2000-01-02", "Part", "Definition"), word_entry("Word3", "2000-01-03", "Part", "Definition"), word_entry("Word4", "2000-01-04", "Part", "Definition"), word_entry("Word5", "2000-01-05", "Part", "Definition") ])]
    #[case::add_5_entries_with_same_date(1, vec![ word_entry("What", "2000-01-01", "Part", "Definition"), word_entry("Is", "2000-01-01", "Part", "Definition"), word_entry("This", "2000-01-01", "Part", "Definition"), word_entry("Doing", "2000-01-01", "Part", "Definition"), word_entry("Man", "2000-01-01", "Part", "Definition") ])]
    #[case::add_duplicate_entries_then_new_entry(2, vec![ word_entry("Word1", "2000-01-01", "Part", "Definition"), word_entry("Word2", "2000-01-01", "Part", "Definition"), word_entry("Word3", "2000-01-01", "Part", "Definition"), word_entry("Word4", "2000-01-01", "Part", "Definition"), word_entry("Word5", "2000-01-04", "Part", "Definition") ])]
    fn test_add_entry(#[case] expected_count: usize, #[case] entries: Vec<WordEntry>) {
        let mut manager = word_list_manager(vec![]);

        for entry in entries {
            match manager.add_entry(entry) {
                Ok(true) => {
                    assert!(true);
                }
                Ok(false) => {
                    assert!(true);
                }
                Err(_error) => {
                    assert!(false);
                }
            }
        }

        assert_eq!(manager.count(), expected_count);
    }

    /// This is the same exact as add entry, but with the add entries method.
    #[rstest]
    #[case::add_one_entry(1, vec![ word_entry("Word1", "2000-01-01", "Part", "Definition") ]) ]
    #[case::add_multiple_entries(5, vec![ word_entry("Word1", "2000-01-01", "Part", "Definition"), word_entry("Word2", "2000-01-02", "Part", "Definition"), word_entry("Word3", "2000-01-03", "Part", "Definition"), word_entry("Word4", "2000-01-04", "Part", "Definition"), word_entry("Word5", "2000-01-05", "Part", "Definition") ])]
    #[case::add_5_entries_with_same_date(1, vec![ word_entry("What", "2000-01-01", "Part", "Definition"), word_entry("Is", "2000-01-01", "Part", "Definition"), word_entry("This", "2000-01-01", "Part", "Definition"), word_entry("Doing", "2000-01-01", "Part", "Definition"), word_entry("Man", "2000-01-01", "Part", "Definition") ])]
    #[case::add_duplicate_entries_then_new_entry(2, vec![ word_entry("Word1", "2000-01-01", "Part", "Definition"), word_entry("Word2", "2000-01-01", "Part", "Definition"), word_entry("Word3", "2000-01-01", "Part", "Definition"), word_entry("Word4", "2000-01-01", "Part", "Definition"), word_entry("Word5", "2000-01-04", "Part", "Definition") ])]
    fn test_add_entries(#[case] expected_count: usize, #[case] entries: Vec<WordEntry>) {
        let mut manager = word_list_manager(vec![]);

        match manager.add_entries(entries.clone()) {
            Ok(_size) => {
                assert!(true);
            }
            Err(_error) => {
                assert!(false);
            }
        }

        assert_eq!(manager.count(), expected_count);
    }

    #[rstest]
    #[case::length_of_0(0, true)]
    #[case::length_of_1(1, false)]
    #[case::length_of_2(2, false)]
    #[case::length_of_3(3, false)]
    #[case::length_of_4(4, false)]
    #[case::length_of_5(5, true)]
    #[case::length_of_100000(100000, true)] // absurd length, expect nothing
    fn test_get_random_by_length(#[case] length: usize, #[case] expect_none: bool) {
        let word4: WordEntry = word_entry("4rrr", "", "", "");
        let word3: WordEntry = word_entry("3ee", "", "", "");
        let word2: WordEntry = word_entry("2u", "", "", "");
        let word1: WordEntry = word_entry("1", "", "", "");
        let manager = word_list_manager(vec![word1, word2, word3, word4]);

        match manager.get_random_by_length(length) {
            Some(word) => {
                assert_eq!(word.length, length)
            }
            None => {
                assert!(expect_none)
            }
        }
    }

    #[rstest]
    #[case::no_entries("", "", true, vec![])]
    #[case::normal_entries("2001-01-01", "2004-01-01", false, vec![ word_entry("", "2001-01-01", "", ""), word_entry("", "2004-01-01", "", "")])]
    #[case::entry_between_start_and_end("2001-01-01", "2004-01-01", false, vec![ word_entry("", "2001-01-01", "", ""), word_entry("", "2002-01-01", "", "") , word_entry("", "2004-01-01", "", "")])]
    #[case::one_entry("2001-01-01", "2001-01-01", false, vec![ word_entry("", "2001-01-01", "", "")])]
    fn test_get_date_range(
        #[case] expected_start: &str,
        #[case] expected_end: &str,
        #[case] expect_none: bool,
        #[case] entries: Vec<WordEntry>,
    ) {
        let manager = word_list_manager(entries);

        match manager.get_date_range() {
            Some((start_date, end_date)) => {
                assert_eq!(start_date, make_naive_date(expected_start));
                assert_eq!(end_date, make_naive_date(expected_end));
            }
            None => assert!(expect_none),
        }
    }

    #[rstest]
    #[case::no_entries("2001-01-01", false, vec![])]
    #[case::with_entries_but_wrong_date("2001-01-01", false, vec![ word_entry("", "2001-02-01", "", "")])]
    #[case::with_entries_but_correct_date("2001-01-01", true, vec![ word_entry("", "2001-01-01", "", "")])]
    fn test_has_date(
        #[case] date: &str,
        #[case] expect_contains_date: bool,
        #[case] entries: Vec<WordEntry>,
    ) {
        let manager = word_list_manager(entries);

        assert_eq!(
            manager.has_date(make_naive_date(date)),
            expect_contains_date
        );
    }

    #[rstest]
    #[case::no_entries(0, vec![])]
    #[case::one_entries(1, vec![ word_entry("", "2001-02-01", "", "") ])]
    #[case::four_entries(4, vec![ word_entry("", "2001-02-01", "", ""), word_entry("", "2001-02-01", "", ""), word_entry("", "2001-02-01", "", ""), word_entry("", "2001-02-01", "", "") ])]
    fn test_count_entries(#[case] expected_count: usize, #[case] entries: Vec<WordEntry>) {
        let manager = word_list_manager(entries);

        assert_eq!(manager.count(), expected_count);
    }

    #[rstest]
    #[case::file_exists(true, "wotd_words.json")]
    #[case::file_doesnt_exist(false, "")]
    fn test_file_exists(#[case] expected_exists: bool, #[case] path: &str) {
        // wotd_words.json should always exist for program functionality so I used that to test that file exists
        assert_eq!(WordListManager::file_exists(path), expected_exists);
    }

    /// This test fails, all_entries specifies it comes in sorted order, by length, then by date
    #[rstest]
    #[case::no_entries(vec![])]
    #[case::in_order_entries(vec![ word_entry("a", "2001-01-01", "", ""), word_entry("ab", "2001-02-01", "", ""), word_entry("abc", "2001-03-01", "", "")])]
    #[case::out_of_order_entries(vec![ word_entry("ab", "2001-02-01", "", ""), word_entry("a", "2001-03-01", "", ""), word_entry("abc", "2001-01-01", "", "")])]
    #[case::same_length_out_of_order_date(vec![ word_entry("a", "2001-02-01", "", ""), word_entry("b", "2001-03-01", "", ""), word_entry("c", "2001-01-01", "", "")])]
    #[case::same_date_out_of_order_length(vec![ word_entry("abc", "2001-01-01", "", ""), word_entry("c", "2001-01-01", "", ""), word_entry("ab", "2001-01-01", "", "")])]
    fn test_all_entries(#[case] entries: Vec<WordEntry>) {
        let manager = word_list_manager(entries);

        let mut previous_date = make_naive_date("0000-00-00");
        let mut previous_length: usize = 0;
        // sorted by length, then by date
        for entry in manager.all_entries() {
            // Reset date of length changes
            if entry.length > previous_length {
                previous_date = entry.date_featured;
            }

            assert!(entry.length >= previous_length);
            assert!(entry.date_featured >= previous_date);

            previous_date = entry.date_featured;
            previous_length = entry.length;
        }
    }

    #[rstest]
    #[case::no_entries(0, 0, vec![])]
    #[case::one_entry(1, 1, vec![ word_entry("1", "", "", "")])]
    #[case::one_entry_wrong_length(1, 0, vec![ word_entry("22", "", "", "")])]
    #[case::multiple_entries_different_length(2, 2, vec![ word_entry("1", "", "", ""), word_entry("22", "", "", ""), word_entry("22", "", "", ""), word_entry("333", "", "", ""), word_entry("333", "", "", ""), word_entry("333", "", "", "")])]
    fn test_entries_by_length(
        #[case] target_length: usize,
        #[case] expected_count: usize,
        #[case] entries: Vec<WordEntry>,
    ) {
        let manager = word_list_manager(entries);

        let words = manager.entries_by_length(target_length);

        assert_eq!(words.len(), expected_count);
    }

    #[rstest]
    #[case::no_entries(vec![])]
    #[case::in_order_entries(vec![ word_entry("a", "2001-01-01", "", ""), word_entry("ab", "2001-02-01", "", ""), word_entry("abc", "2001-03-01", "", "")])]
    #[case::out_of_order_entries(vec![ word_entry("ab", "2001-02-01", "", ""), word_entry("a", "2001-03-01", "", ""), word_entry("abc", "2001-01-01", "", "")])]
    #[case::same_length_out_of_order_date(vec![ word_entry("a", "2001-02-01", "", ""), word_entry("b", "2001-03-01", "", ""), word_entry("c", "2001-01-01", "", "")])]
    #[case::same_date_out_of_order_length(vec![ word_entry("abc", "2001-01-01", "", ""), word_entry("c", "2001-01-01", "", ""), word_entry("ab", "2001-01-01", "", "")])]
    fn test_sort_entries(#[case] entries: Vec<WordEntry>) {
        let mut manager = word_list_manager(entries);

        let mut previous_date = make_naive_date("0000-00-00");
        let mut previous_length: usize = 0;
        // sorted by length, then by date
        manager.sort_entries();
        for entry in manager.all_entries() {
            // Reset date of length changes
            if entry.length > previous_length {
                previous_date = entry.date_featured;
            }

            assert!(entry.length >= previous_length);
            assert!(entry.date_featured >= previous_date);

            previous_date = entry.date_featured;
            previous_length = entry.length;
        }
    }

    fn test_persist() {}
}
