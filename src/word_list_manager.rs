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
        pool.choose(&mut rand::thread_rng())
            .map(|e| (*e).clone())
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