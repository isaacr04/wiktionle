use crate::engine::{Game, GameOptions, GameStatus, GuessResult};
use crate::theme::Theme;
use crossterm::event::{KeyCode, KeyEvent};

#[derive(PartialEq)]
pub enum Disclaimer {
    MoveFeedback(GuessResult),
    GameWonMessage,
    GameOverMessage(String),
    WelcomeMessage,
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

impl App {
    pub fn new(args: AppOptions) -> Self {
        App {
            game: Game::new(args.game_config),
            input: String::from(""),
            disclaimer: Some(Disclaimer::WelcomeMessage),
            should_quit: false,
            theme: args.theme,
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) -> () {
        if self.game.game_status() != GameStatus::InProgress {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Backspace => self.on_backspace(),
            KeyCode::Enter => self.on_enter_press(),
            KeyCode::Char(letter) => self.on_letter_entered(letter),
            _ => (),
        };
    }

    pub fn on_valid_word(&mut self) -> () {
        self.disclaimer = None;
        self.input = String::from("");
    }

    pub fn on_backspace(&mut self) -> () {
        let _ = self.input.pop();
        ()
    }

    pub fn on_letter_entered(&mut self, letter: char) -> () {
        if self.input.chars().count() <= 4 {
            self.input.push(letter);
        }
    }

    pub fn on_enter_press(&mut self) -> () {
        // clear the disclaimer the first time a word is played
        if self.disclaimer == Some(Disclaimer::WelcomeMessage) {
            self.disclaimer = None;
        }

        if &self.input.chars().count() != &5 {
            return ();
        }

        match self.game.guess(&self.input) {
            (GameStatus::Lost, _) => {
                if let Ok(answer) = self.game.get_answer() {
                    self.disclaimer = Some(Disclaimer::GameOverMessage(answer.to_string()));
                }
            }
            (GameStatus::Won, _) => {
                self.disclaimer = Some(Disclaimer::GameWonMessage);
            }
            (_, word_res) => match word_res {
                GuessResult::Valid => {
                    let _ = &self.on_valid_word();
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
    use crate::engine::GameDifficulty;
    use crossterm::event::KeyModifiers;

    use super::*;

    /// Setup function preparing a test app
    /// with set settings and no answer
    fn setup_app(answer: Option<String>) -> App {
        let difficulty = GameDifficulty::Easy;
        let theme = Theme::dark_theme();
        let app = App::new(AppOptions {
            theme: theme,
            game_config: GameOptions {
                answer: answer,
                difficulty: difficulty,
            },
        });
        app
    }

    /// Helper function to make a quick key event based on keycode
    fn make_key_event(key_code: KeyCode) -> KeyEvent {
        let modifiers = KeyModifiers::empty();
        let key_event = KeyEvent::new(key_code, modifiers);
        key_event
    }

    /// Helper function to enter letters into app using on_key
    fn app_enter_letters(app: &mut App, word: &str) -> () {
        for character in word.chars() {
            app.on_key(make_key_event(KeyCode::Char(character)));
        }
    }

    #[test]
    /// Test escape quitting game
    fn test_on_key_escape() {
        let mut app = setup_app(None);
        app.on_key(make_key_event(KeyCode::Esc));
        assert!(app.should_quit)
    }

    #[test]
    /// on_valid_word is simple, it clears input, and disclaimer
    fn test_on_valid_word() {
        let mut app = setup_app(None);
        // give test input
        app.input = "test input".to_string();

        assert_eq!(app.input, "test input"); // test input is in app
        assert!(app.disclaimer == Some(Disclaimer::WelcomeMessage)); // disclaimer is welcome

        app.on_valid_word();

        assert_eq!(app.input, ""); // input cleared
        assert!(app.disclaimer == None); // disclaimer is none
    }

    #[test]
    /// Test backspace functionality
    fn test_on_backspace() {
        let mut app = setup_app(None);
        app.input = "Hello".to_string();

        assert_eq!(app.input, "Hello");

        app.on_key(make_key_event(KeyCode::Backspace)); // Hell
        assert_eq!(app.input, "Hell");

        app.on_key(make_key_event(KeyCode::Backspace)); // Hel
        assert_eq!(app.input, "Hel");

        app.on_key(make_key_event(KeyCode::Backspace)); // He
        assert_eq!(app.input, "He");

        app.on_key(make_key_event(KeyCode::Backspace)); // H
        assert_eq!(app.input, "H");

        app.on_key(make_key_event(KeyCode::Backspace)); // should be empty
        assert_eq!(app.input, "");
    }

    #[test]
    /// Check that backspace does nothing when input is already empty
    fn test_on_backspace_no_input() {
        let mut app = setup_app(None);
        app.input = "".to_string();

        // with an empty input using backspace should do nothing
        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "");
    }

    #[test]
    /// Letters can be entered into the app
    fn test_on_letter_entered() {
        let mut app = setup_app(None);

        app.on_key(make_key_event(KeyCode::Char('a')));
        app.on_key(make_key_event(KeyCode::Char('b')));
        app.on_key(make_key_event(KeyCode::Char('c')));

        assert_eq!(app.input, "abc")
    }

    #[test]
    fn test_on_invalid_letter_entered() {
        let mut app = setup_app(None);

        app.on_key(make_key_event(KeyCode::Char('a')));
        app.on_key(make_key_event(KeyCode::Char('#')));
        app.on_key(make_key_event(KeyCode::Char(' ')));

        assert_eq!(app.input, "a")
    }

    #[test]
    /// On enter input is validated
    fn test_on_enter_press() {
        let mut app = setup_app(None);

        assert!(app.disclaimer == Some(Disclaimer::WelcomeMessage)); // initialized with welcome

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // disclaimer cleared
    }

    #[test]
    /// On enter input is validated
    fn test_on_enter_press_correct_answer() {
        let mut app = setup_app(Some("train".to_string()));

        // Train entered
        app.on_key(make_key_event(KeyCode::Char('t')));
        app.on_key(make_key_event(KeyCode::Char('r')));
        app.on_key(make_key_event(KeyCode::Char('a')));
        app.on_key(make_key_event(KeyCode::Char('i')));
        app.on_key(make_key_event(KeyCode::Char('n')));

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == Some(Disclaimer::GameWonMessage)); // game won
    }

    #[test]
    /// On enter input is validated
    fn test_on_enter_press_correct_answer_different_case() {
        let mut app = setup_app(Some("train".to_string()));

        app_enter_letters(&mut app, "TRain");

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == Some(Disclaimer::GameWonMessage)); // game won
    }

    #[test]
    /// On enter input is validated
    fn test_on_enter_press_valid_word() {
        let mut app = setup_app(Some("plain".to_string()));

        app_enter_letters(&mut app, "train");

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // no response is given move to the next word
        assert_eq!(app.input, ""); // word cleared in preparation for next
    }

    #[test]
    fn test_on_enter_press_invalid_word() {
        let mut app = setup_app(Some("train".to_string()));

        app_enter_letters(&mut app, "tr@15");

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // disclaimer shouldn't be anything
    }

    #[test]
    fn test_app_lose_game() {
        let mut app = setup_app(Some("input".to_string()));

        app_enter_letters(&mut app, "train");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // no disclaimer
        assert_eq!(app.input, ""); // word cleared

        app_enter_letters(&mut app, "plain");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // no disclaimer
        assert_eq!(app.input, ""); // word cleared

        app_enter_letters(&mut app, "faint");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // no disclaimer
        assert_eq!(app.input, ""); // word cleared

        app_enter_letters(&mut app, "claim");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // no disclaimer
        assert_eq!(app.input, ""); // word cleared

        app_enter_letters(&mut app, "sword");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // no disclaimer
        assert_eq!(app.input, ""); // word cleared

        // Last word, end the game losing
        app_enter_letters(&mut app, "flail");
        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer.is_some()); // disclaimer given lost
        assert_eq!(app.game.game_status(), GameStatus::Lost)
    }

    #[test]
    /// test that valid words can be entered after invalid words
    fn test_app_enter_valid_word_after_clearing_input() {
        let mut app = setup_app(None);

        app_enter_letters(&mut app, "asdas"); // not a word
        app.on_key(make_key_event(KeyCode::Enter));

        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));
        app.on_key(make_key_event(KeyCode::Backspace));

        app_enter_letters(&mut app, "valid");
        assert_eq!(app.input, "valid");

        app.on_key(make_key_event(KeyCode::Enter));
        assert!(app.disclaimer == None); // disclaimer should be cleared
    }
}
