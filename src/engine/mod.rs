use crate::engine::game_error::GameError;
use crate::word_list_manager::{WordEntry, WordListManager};

use std::collections::{HashMap, HashSet};

mod game_error;
mod utils;
mod words;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameStatus {
    Won,
    InProgress,
    Lost,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GuessResult {
    DoesNotIncludeRequiredLetter(char),
    LetterDoesNotMatch(char, usize),
    DuplicateGuess,
    GameIsAlreadyOver,
    IncorrectCharacterCount,
    NotInDictionary,
    Valid,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub enum HitAccuracy {
    InRightPlace,
    InWord,
    NotInWord,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum GameDifficulty {
    Easy,
    Hard,
}

/// Primary Game structure, representing a game of wordle being played
pub struct Game {
    guesses: Vec<WordGuess>,
    answer: WordEntry,
    difficulty: GameDifficulty,
    game_status: GameStatus,
    correct_positions: HashSet<usize>,
    dictionary: HashSet<String>,
    played_letters: HashMap<char, HitAccuracy>,
    row_states: Vec<RowState>,
}

#[derive(Debug, PartialEq)]
pub struct WordGuess {
    pub letters: Vec<GuessLetter>,
}

impl WordGuess {
    /// Convert the Guess from a Vector of Guess Letters to a String representation
    pub fn word(&self) -> String {
        self.letters
            .as_slice()
            .iter()
            .map(|gl| gl.letter)
            .collect()
    }

    /// Get all individual letters of the word
    pub fn letters(&self) -> &[GuessLetter] {
        self.letters.as_slice()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct GuessLetter {
    pub letter: char,
    pub accuracy: HitAccuracy,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RowState {
    Empty,
    Current,
    AlreadyGuessed,
}

pub struct GameOptions {
    pub answer: Option<String>,
    pub difficulty: GameDifficulty,
    pub word_length: usize,
}

impl Default for GameOptions {
    /// Defines default state for game option if no properties are set
    fn default() -> Self {
        GameOptions {
            answer: None,
            difficulty: GameDifficulty::Easy,
            word_length: 5,
        }
    }
}

impl Game {
    /// Constructor for a Game instance defining initial state and answer
    ///
    /// * `args` - defines the options used to configure the game when initially created
    pub fn new(args: GameOptions) -> Self {
        let answer = args.answer.map_or_else(
            || utils::get_random_word_by_length(args.word_length),
            |a| {
                WordEntry::new(
                    a,
                    chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
                    "unknown",
                    "",
                )
            },
        );

        Game {
            guesses: Vec::with_capacity(6),
            answer,
            difficulty: args.difficulty,
            game_status: GameStatus::InProgress,
            correct_positions: HashSet::new(),
            dictionary: utils::dictionary(),
            played_letters: HashMap::new(),
            row_states: vec![
                RowState::Current,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
            ],
        }
    }

    /// Obtain the current status of the game, whether it is won, lost, or inprogress
    pub fn game_status(&self) -> GameStatus {
        self.game_status
    }

    /// Obtain the answer of the game, if not possible return an error
    pub fn get_answer(&self) -> Result<String, GameError> {
        if self.game_status == GameStatus::Lost {
            Ok(self.answer.word.to_string())
        } else {
            Err(GameError::GameNotLostError)
        }
    }

    /// Obtain all guesses currently made
    pub fn guesses(&self) -> &[WordGuess] {
        self.guesses.as_slice()
    }

    /// Determine if word is present in the dictionary
    /// Temporarily always returns true because the dictionary only uses 5 letter words
    ///
    /// * `word` - word to find in dictionary
    fn in_dictionary(&self, word: &str) -> bool {
        // self.dictionary.get(word).is_some()
        true
    }

    /// get character of answer from a specified index
    ///
    /// * `index` - index of the answer to pick a letter from
    fn answer_char_at_index(&self, index: usize) -> char {
        self.answer.word.chars().nth(index).unwrap_or('\0')
    }

    /// Match letter at index of answer
    ///
    /// * `index` - index for specific letter of answer
    /// * `letter` - letter being compared to letter at index
    fn matches_answer_at_index(&self, index: usize, letter: char) -> bool {
        letter == self.answer_char_at_index(index)
    }

    /// Determine the state of the current row, for the sake of marking which cells
    /// contain letters in the right place, wrong place, or not in the word at all
    fn recalculate_row_states(&mut self) {
        let number_of_guesses_so_far = self.guesses().len();

        let row_states = vec![1, 2, 3, 4, 5, 6]
            .into_iter()
            .map(|i| {
                if number_of_guesses_so_far == 6 {
                    return RowState::AlreadyGuessed;
                }

                if i <= number_of_guesses_so_far {
                    return RowState::AlreadyGuessed;
                }

                if i == number_of_guesses_so_far + 1 {
                    if self.game_status == GameStatus::Won {
                        return RowState::Empty;
                    }
                    return RowState::Current;
                }

                RowState::Empty
            })
            .collect();

        self.row_states = row_states;
    }

    /// Update the registry of letters available for the answer depending on the
    /// previous guesses
    fn recalculate_played_letter_registry(&mut self, guess: &WordGuess) {
        for gl in guess.letters() {
            match self.played_letters.get_mut(&gl.letter) {
                None => {
                    self.played_letters.insert(gl.letter, gl.accuracy);
                }
                Some(accuracy_value) => {
                    if gl.accuracy < *accuracy_value {
                        *accuracy_value = gl.accuracy;
                    }
                }
            }
        }
    }

    /// Check for duplicate guesses to prevent the same word being entered twice
    ///
    /// * `guess_input` - guess input as a string to be check with previous guesses
    fn guess_already_exists(&self, guess_input: &str) -> bool {
        let existing_guesses: Vec<String> = self.guesses.iter().map(|g| g.word()).collect();
        existing_guesses.contains(&guess_input.to_string())
    }

    /// Make a guess to the game of a specific input.
    /// Returning game status after guess and result of guess.
    ///
    /// * `guess_input` - the guess made
    pub fn guess(&mut self, guess_input: &str, word_length: usize) -> (GameStatus, GuessResult) {
        if self.game_status == GameStatus::Won || self.game_status == GameStatus::Lost {
            return (self.game_status, GuessResult::GameIsAlreadyOver);
        }

        if guess_input.len() != word_length {
            return (self.game_status, GuessResult::IncorrectCharacterCount);
        }

        if self.guess_already_exists(guess_input) {
            return (self.game_status, GuessResult::DuplicateGuess);
        }

        if !self.in_dictionary(guess_input) {
            return (self.game_status, GuessResult::NotInDictionary);
        }

        if self.difficulty == GameDifficulty::Hard {
            for (index, letter) in guess_input.chars().enumerate() {
                if self.correct_positions.contains(&index) {
                    if !self.matches_answer_at_index(index, letter) {
                        let char_at_index = self.answer_char_at_index(index);
                        return (
                            self.game_status,
                            GuessResult::LetterDoesNotMatch(char_at_index, index + 1),
                        );
                    }
                }
            }

            for letter in self.answer.word.chars() {
                let is_discovered = self.is_letter_uncovered(letter);

                if is_discovered && !guess_input.contains(letter) {
                    return (
                        self.game_status,
                        GuessResult::DoesNotIncludeRequiredLetter(letter),
                    );
                }
            }
        }

        let guess = self.build_guess(guess_input, word_length);
        self.recalculate_played_letter_registry(&guess);

        self.guesses.push(guess);

        if guess_input.to_lowercase() == self.answer.word {
            self.game_status = GameStatus::Won;
        }

        // we need to do this _after setting the game state to 'won', but before returning
        // This way the board does not update with a duplicate row in the next 'current' row
        self.recalculate_row_states();

        if self.game_status == GameStatus::Won {
            return (self.game_status, GuessResult::Valid);
        }

        if self.guesses.len() == 6 {
            self.game_status = GameStatus::Lost;
        }

        (self.game_status, GuessResult::Valid)
    }

    /// Obtain the current states of the rows
    pub fn row_states(&self) -> Vec<RowState> {
        self.row_states.clone()
    }

    pub fn is_letter_uncovered(&self, letter: char) -> bool {
        match &self.get_letter_match_state(letter) {
            None => false,
            Some(HitAccuracy::NotInWord) => false,
            Some(_) => true,
        }
    }

    /// Build a guess from the current guess input
    ///
    /// * `guess_input` - the guess being made
    fn build_guess(&mut self, guess_input: &str, word_length: usize) -> WordGuess {
        let mut discoverable_letters = utils::build_letter_counts(&self.answer.word);
        let mut guess_letters: Vec<Option<GuessLetter>> = vec![None];
        for _i in 1..word_length {
            guess_letters.push(None);
        }

        // Weird stuff. We walk the word twice; We go over the correct guesses first, so that we
        // can subtract their letters from the count of available letters to colorize.
        for (idx, c) in guess_input.chars().enumerate() {
            if self.matches_answer_at_index(idx, c) {
                guess_letters[idx] =
                    Some(self.build_guess_letter_with_accuracy(idx, c, &mut discoverable_letters))
            }
        }

        // Then we go over the letters that are not correct.
        for (idx, c) in guess_input.chars().enumerate() {
            if guess_letters[idx].is_none() {
                guess_letters[idx] =
                    Some(self.build_guess_letter_with_accuracy(idx, c, &mut discoverable_letters))
            }
        }

        WordGuess {
            letters: guess_letters.iter().map(|o| o.unwrap()).collect(),
        }
    }

    /// Build guess including the accuracy of a letter in the guess
    ///
    /// * `letter_index` - index of letter
    /// * `raw_letter` - character value of the letter
    /// * `discoverable_letters` - list of letters that can be discovered
    fn build_guess_letter_with_accuracy(
        &mut self,
        letter_index: usize,
        raw_letter: char,
        discoverable_letters: &mut HashMap<char, usize>,
    ) -> GuessLetter {
        let accuracy = match &self.answer.word.contains(raw_letter) {
            true => {
                let in_same_place = self.matches_answer_at_index(letter_index, raw_letter);

                if in_same_place {
                    if let Some(ch) = discoverable_letters.get_mut(&raw_letter) {
                        *ch -= 1;
                    }
                    self.correct_positions.insert(letter_index);
                    HitAccuracy::InRightPlace
                } else {
                    if let Some(ch) = discoverable_letters.get_mut(&raw_letter) {
                        if *ch >= 1 {
                            *ch -= 1;
                            HitAccuracy::InWord
                        } else {
                            HitAccuracy::NotInWord
                        }
                    } else {
                        HitAccuracy::NotInWord
                    }
                }
            }
            false => HitAccuracy::NotInWord,
        };

        GuessLetter {
            letter: raw_letter,
            accuracy,
        }
    }

    /// Get the accuracy of the letter from the currently played letters
    pub fn get_letter_match_state(&self, letter: char) -> Option<HitAccuracy> {
        self.played_letters.get(&letter).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_guess() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("pasta", 5);
        assert_eq!(game.guesses.len(), 1)
    }

    #[rustfmt::skip]
    #[test]
    fn test_a_guess_is_stored_correctly() {
        let mut game = Game::new(GameOptions { answer: Some("haste".to_string()), difficulty: GameDifficulty::Easy, word_length: 5 });
        game.guess("heart", 5);

        let spell_guess = super::WordGuess {
            letters: vec![
                GuessLetter { letter: 'h', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'e', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'a', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'r', accuracy: HitAccuracy::NotInWord },
                GuessLetter { letter: 't', accuracy: HitAccuracy::InWord }
            ],
        };
        assert_eq!(game.guesses[0], spell_guess)
    }

    #[rustfmt::skip]
    #[test]
    fn test_letters_are_marked_in_word_until_the_count_of_letters_is_met() {
        let mut game = Game::new(GameOptions { answer: Some("sleep".to_string()), difficulty: GameDifficulty::Easy, word_length: 5 });
        game.guess("spell", 5);

        let spell_guess = super::WordGuess {
            letters: vec![
                GuessLetter { letter: 's', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'p', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'e', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'l', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'l', accuracy: HitAccuracy::NotInWord }
            ],
        };
        assert_eq!(game.guesses[0], spell_guess)
    }

    #[rustfmt::skip]
    #[test]
    fn test_counts_apply_to_the_in_right_place_characters_first() {
        let mut game = Game::new(GameOptions { answer: Some("ahead".to_string()), difficulty: GameDifficulty::Easy, word_length: 5 });
        game.guess("added", 5);

        let spell_guess = super::WordGuess {
            letters: vec![
                GuessLetter { letter: 'a', accuracy: HitAccuracy::InRightPlace },
                GuessLetter { letter: 'd', accuracy: HitAccuracy::NotInWord },
                GuessLetter { letter: 'd', accuracy: HitAccuracy::NotInWord },
                GuessLetter { letter: 'e', accuracy: HitAccuracy::InWord },
                GuessLetter { letter: 'd', accuracy: HitAccuracy::InRightPlace }
            ],
        };
        assert_eq!(game.guesses[0], spell_guess)
    }

    #[test]
    fn test_answer_at_index() {
        let game = Game::new(GameOptions { answer: Some("ahead".to_string()), difficulty: GameDifficulty::Easy, word_length: 5 });
        assert_eq!(game.answer_char_at_index(4), 'd');
    }

    #[test]
    fn test_answer_at_index_out_of_bounds() {
        let game = Game::new(GameOptions { answer: Some("ahead".to_string()), difficulty: GameDifficulty::Easy, word_length: 5 });
        assert_eq!(game.answer_char_at_index(6), '\0');
    }

    #[test]
    fn test_matches_answer_at_index() {
        let game = Game::new(GameOptions { answer: Some("ahead".to_string()), difficulty: GameDifficulty::Easy, word_length: 5 });
        assert!(game.matches_answer_at_index(4, 'd'));
    }

    #[test]
    fn test_matches_answer_at_index_out_of_bounds() {
        let game = Game::new(GameOptions { answer: Some("ahead".to_string()), difficulty: GameDifficulty::Easy, word_length: 5 });
        assert!(game.matches_answer_at_index(6, '\0'));
    }

    #[test]
    fn test_cannot_add_duplicate_guess() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("pasta", 5);
        let (_, duplicate_guess) = game.guess("pasta", 5);
        assert_eq!(duplicate_guess, GuessResult::DuplicateGuess);
    }

    #[test]
    fn test_a_correct_guess_wins_the_game() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (won_the_game, _) = game.guess("slump", 5);
        assert_eq!(won_the_game, GameStatus::Won);
    }

    #[test]
    fn test_a_correct_guess_with_different_case_wins() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (won_the_game, _) = game.guess("Slump", 5);
        assert_eq!(won_the_game, GameStatus::Won);
    }

    #[test]
    fn test_a_guess_cannot_be_less_than_five_characters() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (_, char_count_wrong) = game.guess("slp", 5);
        assert_eq!(char_count_wrong, GuessResult::IncorrectCharacterCount);
    }

    #[test]
    fn test_a_guess_cannot_be_more_than_five_characters() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (_, char_count_wrong) = game.guess("slumffffp", 5);
        assert_eq!(char_count_wrong, GuessResult::IncorrectCharacterCount);
    }

    #[test]
    fn test_the_game_is_lost_after_six_incorrect_guesses() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("admit", 5);
        game.guess("adorn", 5);
        game.guess("adult", 5);
        game.guess("affix", 5);
        game.guess("afire", 5);
        let (lost_the_game, _) = game.guess("after", 5);
        assert_eq!(lost_the_game, GameStatus::Lost);
    }

    #[test]
    fn test_cannot_add_guesses_after_the_game_is_won() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("slump", 5);
        let (won_the_game, game_already_over) = game.guess("adept", 5);

        assert_eq!(won_the_game, GameStatus::Won);
        assert_eq!(game_already_over, GuessResult::GameIsAlreadyOver);
    }

    #[test]
    fn test_cannot_add_guesses_after_the_game_is_lost() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("admit", 5);
        game.guess("adorn", 5);
        game.guess("adult", 5);
        game.guess("affix", 5);
        game.guess("afire", 5);
        game.guess("aging", 5);

        let (lost_the_game, game_already_over) = game.guess("agony", 5);
        assert_eq!(lost_the_game, GameStatus::Lost);
        assert_eq!(game_already_over, GuessResult::GameIsAlreadyOver);
    }

    #[test]
    fn test_cannot_add_a_word_that_does_not_exist() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        let (game_continues, invalid_word) = game.guess("djkle", 5);
        assert_eq!(game_continues, GameStatus::InProgress);
        assert_eq!(invalid_word, GuessResult::NotInDictionary);
    }

    #[test]
    fn test_can_get_the_answer_after_the_game_is_lost() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("admit", 5);
        game.guess("adorn", 5);
        game.guess("adult", 5);
        game.guess("affix", 5);
        game.guess("afire", 5);

        assert_eq!(game.get_answer(), Err(GameError::GameNotLostError));
        game.guess("aging", 5);
        assert_eq!(game.get_answer(), Ok("slump".to_string()));
    }

    #[test]
    fn test_hard_mode_requires_guessing_letters_that_have_been_found_in_place() {
        let mut game = Game::new(GameOptions {
            answer: Some("abbey".to_string()),
            difficulty: GameDifficulty::Hard,
            word_length: 5,
        });
        game.guess("sleep", 5);

        let (_, required_letter) = game.guess("hours", 5);
        assert_eq!(required_letter, GuessResult::LetterDoesNotMatch('e', 4));
    }

    #[test]
    fn test_hard_mode_requires_guessing_letters_that_have_been_found_in_the_word() {
        let mut game = Game::new(GameOptions {
            answer: Some("abbey".to_string()),
            difficulty: GameDifficulty::Hard,
            word_length: 5,
        });
        let (_, valid_word) = game.guess("slept", 5);
        assert_eq!(valid_word, GuessResult::Valid);

        let (_, required_letter) = game.guess("grift", 5);
        assert_eq!(
            required_letter,
            GuessResult::DoesNotIncludeRequiredLetter('e')
        );
    }

    #[test]
    fn test_hard_mode_can_include_guesses_with_old_and_new_letters() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            difficulty: GameDifficulty::Hard,
            word_length: 5,
        });
        game.guess("sleep", 5);

        let (game_continues, valid_word) = game.guess("sloop", 5);
        assert_eq!(game_continues, GameStatus::InProgress);
        assert_eq!(valid_word, GuessResult::Valid);
    }

    #[test]
    fn test_keeps_track_of_which_letters_matched() {
        let mut game = Game::new(GameOptions {
            answer: Some("slump".to_string()),
            ..Default::default()
        });
        game.guess("slept", 5);

        assert_eq!(
            game.get_letter_match_state('s'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('l'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('e'),
            Some(HitAccuracy::NotInWord)
        );
        assert_eq!(game.get_letter_match_state('p'), Some(HitAccuracy::InWord));
        assert_eq!(
            game.get_letter_match_state('t'),
            Some(HitAccuracy::NotInWord)
        );

        assert_eq!(game.get_letter_match_state('u'), None);
        assert_eq!(game.get_letter_match_state('m'), None);
    }

    #[test]
    fn test_letters_matches_are_not_overwritten_by_lesser_tiers() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("larva", 5);

        assert_eq!(
            game.get_letter_match_state('l'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('a'),
            Some(HitAccuracy::InRightPlace)
        );
        assert_eq!(
            game.get_letter_match_state('r'),
            Some(HitAccuracy::NotInWord)
        );
        assert_eq!(
            game.get_letter_match_state('v'),
            Some(HitAccuracy::NotInWord)
        );
        assert_eq!(
            game.get_letter_match_state('a'),
            Some(HitAccuracy::InRightPlace)
        );

        assert_eq!(game.get_letter_match_state('g'), None);
        assert_eq!(game.get_letter_match_state('h'), None);
    }

    #[test]
    fn test_letters_matches_are_not_overwritten_by_subsequent_incorrect_guesses() {
        let mut game = Game::new(GameOptions {
            answer: Some("ahead".to_string()),
            ..Default::default()
        });
        game.guess("lease", 5);
        assert_eq!(game.get_letter_match_state('e'), Some(HitAccuracy::InWord));

        game.guess("preen", 5);
        assert_eq!(
            game.get_letter_match_state('e'),
            Some(HitAccuracy::InRightPlace)
        );
    }

    #[test]
    fn test_row_states_are_tracked_at_the_start_of_the_game() {
        let game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        assert_eq!(
            game.row_states(),
            vec![
                RowState::Current,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty
            ]
        );
    }

    #[test]
    fn test_row_states_are_tracked_in_the_middle_of_the_game() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("admit", 5);

        assert_eq!(
            game.row_states(),
            vec![
                RowState::AlreadyGuessed,
                RowState::Current,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty
            ]
        );
    }

    #[test]
    fn test_row_states_are_tracked_when_you_win_before_the_end() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("admit", 5);
        game.guess("laugh", 5);

        assert_eq!(
            game.row_states(),
            vec![
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty,
                RowState::Empty
            ]
        );
    }

    #[test]
    fn test_row_states_are_tracked_at_the_end_of_the_game() {
        let mut game = Game::new(GameOptions {
            answer: Some("laugh".to_string()),
            ..Default::default()
        });
        game.guess("admit", 5);
        game.guess("adorn", 5);
        game.guess("adult", 5);
        game.guess("affix", 5);
        game.guess("afire", 5);
        game.guess("aging", 5);
        assert_eq!(
            game.row_states(),
            vec![
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed,
                RowState::AlreadyGuessed
            ]
        );
    }
}