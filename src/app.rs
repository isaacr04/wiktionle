use crate::engine::{Game, GameOptions, GameStatus, GuessResult};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(PartialEq)]
pub enum Disclaimer {
    MoveFeedback(GuessResult),
    GameWonMessage(String, String),
    GameOverMessage(String),
    WelcomeMessage,
    ClassHint(String),
    DefinitonHint(String),
}

pub struct App {
    pub game: Game,
    pub input: String,
    pub disclaimer: Option<Disclaimer>,
    pub should_quit: bool,
    pub theme: Theme,
}

pub struct AppOptions {
    pub theme: Theme,
    pub game_config: GameOptions,
}

/// Encapsulates together, the game, the theme, input handling, and primary app logic
impl App {
    /// Constructor for an app given a set of arguments
    ///
    /// * `args` - defines theme and game options of game instance used by the app
    pub fn new(args: AppOptions) -> Self {
        App {
            game: Game::new(args.game_config),
            input: String::from(""),
            disclaimer: Some(Disclaimer::WelcomeMessage),
            should_quit: false,
            theme: args.theme,
        }
    }

    /// Handle key events, determining which action should be taken depending on input
    ///
    /// * `key` - The key event resulting from a user's input
    pub fn on_key(&mut self, key: KeyEvent) {
        if self.game.game_status() != GameStatus::InProgress {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Backspace => self.on_backspace(),
            KeyCode::Enter => self.on_enter_press(self.game.guesses().len()),
            KeyCode::Char(letter) => {
                if letter.is_alphabetic() {
                    self.on_letter_entered(letter.to_lowercase().next().unwrap())
                }
            },
            _ => (),
        };
    }

    /// On valid word given, clear disclaimer, and input in preparation for next word
    pub fn on_valid_word(&mut self) -> () {
        self.disclaimer = None;
        self.input = String::from("");
    }

    /// On backspace remove last letter from input
    pub fn on_backspace(&mut self) -> () {
        let _ = self.input.pop();
        ()
    }

    /// On letter input it is appended to the current input limited to 5 characters
    pub fn on_letter_entered(&mut self, letter: char) {
        if self.input.chars().count() < self.game.word_length() {
            self.input.push(letter);
        }
    }

    /// Handle user input of enter
    /// * display disclaimer for invalid input, or other messages
    /// * checks if input was correct or wrong and display correct disclaimer
    /// * checks if game was lost
    pub fn on_enter_press(&mut self, guesses: usize) {
        if self.disclaimer == Some(Disclaimer::WelcomeMessage) {
            self.disclaimer = None;
        }

        if self.input.chars().count() != self.game.word_length() {
            return;
        }

        match self.game.guess(&self.input) {
            (GameStatus::Lost, _) => {
                if let Ok(answer) = self.game.get_answer() {
                    self.disclaimer = Some(Disclaimer::GameOverMessage(answer.to_string()));
                }
            }
            (GameStatus::Won, _) => {
                self.disclaimer = Some(Disclaimer::GameWonMessage(self.game.get_definition(), self.game.get_date()));
            }
            (_, word_res) => match word_res {
                GuessResult::Valid => {
                    let _ = &self.on_valid_word();
                    if guesses >= 3 {
                        self.disclaimer = Some(Disclaimer::DefinitonHint(self.game.get_definition()))
                    }
                    else if guesses >= 1 {
                        self.disclaimer = Some(Disclaimer::ClassHint(self.game.get_part_of_speech()))
                    }
                }
                result => {
                    self.disclaimer = Some(Disclaimer::MoveFeedback(result));
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::{GameDifficulty, WordSelectionMode};
    use chrono::NaiveDate;
    use crossterm::event::KeyModifiers;

    use super::*;
    use rstest::*;

    fn setup_app(answer: Option<String>) -> App {
        let difficulty = GameDifficulty::Easy;
        let theme = Theme::dark_theme();
        let app = App::new(AppOptions {
            theme,
            game_config: GameOptions {
                answer,
                difficulty,
                word_length: 5,
                word_selection: WordSelectionMode::RandomByLength,
            },
        });
        app
    }

    fn make_key_event(key_code: KeyCode) -> KeyEvent {
        let modifiers = KeyModifiers::empty();
        KeyEvent::new(key_code, modifiers)
    }

    fn app_enter_letters(app: &mut App, word: &str) {
        for character in word.chars() {
            app.on_key(make_key_event(KeyCode::Char(character)));
        }
    }

    #[rstest]
    fn test_on_key_escape() {
        let mut app = setup_app(None);
        app.on_key(make_key_event(KeyCode::Esc));
        assert!(app.should_quit)
    }

    #[rstest]
    fn test_on_valid_word() {
        let mut app = setup_app(None);
        app.input = "test input".to_string();

        assert_eq!(app.input, "test input");
        assert!(app.disclaimer == Some(Disclaimer::WelcomeMessage));

        app.on_valid_word();

        assert_eq!(app.input, "");
        assert!(app.disclaimer == None);
    }

    #[rstest]
    fn test_on_backspace() {
        let mut app = setup_app(None);
        app.input = "Hello".to_string();

        assert_eq!(app.input, "Hello");

        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "Hell");

        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "Hel");

        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "He");

        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "H");

        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "");
    }

    #[rstest]
    fn test_on_backspace_no_input() {
        let mut app = setup_app(None);
        app.input = "".to_string();

        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "");
    }

    #[rstest]
    fn test_on_letter_entered() {
        let mut app = setup_app(None);

        app.on_key(make_key_event(KeyCode::Char('a')));
        app.on_key(make_key_event(KeyCode::Char('b')));
        app.on_key(make_key_event(KeyCode::Char('c')));

        assert_eq!(app.input, "abc")
    }

    #[rstest]
    fn test_on_invalid_letter_entered() {
        let mut app = setup_app(None);

        app.on_key(make_key_event(KeyCode::Char('a')));
        app.on_key(make_key_event(KeyCode::Char('#')));
        app.on_key(make_key_event(KeyCode::Char(' ')));

        assert_eq!(app.input, "a")
    }

    #[rstest]
    fn test_on_enter_press() {
        let mut app = setup_app(None);

        assert!(app.disclaimer == Some(Disclaimer::WelcomeMessage));

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
    }

    #[rstest]
    fn test_on_enter_press_correct_answer() {
        let mut app = setup_app(Some("train".to_string()));

        app.on_key(make_key_event(KeyCode::Char('t')));
        app.on_key(make_key_event(KeyCode::Char('r')));
        app.on_key(make_key_event(KeyCode::Char('a')));
        app.on_key(make_key_event(KeyCode::Char('i')));
        app.on_key(make_key_event(KeyCode::Char('n')));

        app.on_key(make_key_event(KeyCode::Enter));
        let test = String::new();
        let test_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().to_string();
        assert!(app.disclaimer == Some(Disclaimer::GameWonMessage(test, test_date)));
    }

    #[rstest]
    fn test_on_enter_press_correct_answer_different_case() {
        let mut app = setup_app(Some("train".to_string()));

        app_enter_letters(&mut app, "TRain");

        app.on_key(make_key_event(KeyCode::Enter));
        let test = String::new();
        let test_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().to_string();
        assert!(app.disclaimer == Some(Disclaimer::GameWonMessage(test, test_date)));
    }

    #[rstest]
    fn test_on_enter_press_valid_word() {
        let mut app = setup_app(Some("plain".to_string()));

        app_enter_letters(&mut app, "train");

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
        assert_eq!(app.input, "");
    }

    #[rstest]
    fn test_on_enter_press_invalid_word() {
        let mut app = setup_app(Some("train".to_string()));

        app_enter_letters(&mut app, "tr@15");

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
    }

    #[rstest]
    fn test_app_lose_game() {
        let mut app = setup_app(Some("input".to_string()));

        app_enter_letters(&mut app, "train");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
        assert_eq!(app.input, "");

        app_enter_letters(&mut app, "plain");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
        assert_eq!(app.input, "");

        app_enter_letters(&mut app, "faint");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
        assert_eq!(app.input, "");

        app_enter_letters(&mut app, "claim");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
        assert_eq!(app.input, "");

        app_enter_letters(&mut app, "sword");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
        assert_eq!(app.input, "");

        app_enter_letters(&mut app, "flail");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer.is_some());
        assert_eq!(app.game.game_status(), GameStatus::Lost)
    }

    #[rstest]
    fn test_app_enter_valid_word_after_clearing_input() {
        let mut app = setup_app(None);

        app_enter_letters(&mut app, "asdas");
        app.on_key(make_key_event(KeyCode::Enter));

        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));

        app_enter_letters(&mut app, "valid");
        assert_eq!(app.input, "valid");

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None);
    }
}
