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
    /// with set settings
    fn setup_app() -> App {
        let difficulty = GameDifficulty::Easy;
        let theme = Theme::dark_theme();
        let app = App::new(AppOptions {
            theme: theme,
            game_config: GameOptions {
                answer: None,
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

    #[test]
    /// Test escape quitting game
    fn test_on_key_escape() {
        let mut app = setup_app();
        app.on_key(make_key_event(KeyCode::Esc));
        assert!(app.should_quit)
    }

    #[test]
    /// Test backspace input
    fn test_on_key_backspace() {
        let mut app = setup_app();
        app.input = "123".to_string();

        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "12");
        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "1");
        app.on_key(make_key_event(KeyCode::Backspace));
        assert_eq!(app.input, "");
    }

    #[test]
    /// Test character entry inputs
    fn test_on_characters_entered() {
        let mut app = setup_app();
        app.on_key(make_key_event(KeyCode::Char('a')));
        app.on_key(make_key_event(KeyCode::Char('B')));
        app.on_key(make_key_event(KeyCode::Char(' ')));
        app.on_key(make_key_event(KeyCode::Char('#')));
        app.on_key(make_key_event(KeyCode::Char('5')));

        // Should only accept valid inputs
        assert_eq!(app.input, "aB")
    }

    #[test]
    /// on_valid_word is simple, it clears input, and disclaimer
    fn test_on_valid_word() {
        let mut app = setup_app();
        // give test input
        app.input = "test input".to_string();
        // setup disclaimer
        app.disclaimer = Some(Disclaimer::WelcomeMessage);

        app.on_valid_word();

        assert_eq!(app.input, ""); // input cleared
        assert!(app.disclaimer == None); // disclaimer is none
    }

    #[test]
    /// Test backspace functionaliry
    fn test_on_backspace() {
        let mut app = setup_app();
        app.input = "Hello".to_string();

        assert_eq!(app.input, "Hello");

        app.on_backspace(); // Hell
        assert_eq!(app.input, "Hell");

        app.on_backspace(); // Hel
        assert_eq!(app.input, "Hel");

        app.on_backspace(); // He
        assert_eq!(app.input, "He");

        app.on_backspace(); // H
        assert_eq!(app.input, "H");

        app.on_backspace(); //
        app.on_backspace(); //
        app.on_backspace(); // should still be empty
        assert_eq!(app.input, "");
    }

    #[test]
    fn test_on_letter_entered() {
        let mut app = setup_app();

        app.on_letter_entered('a');
        app.on_letter_entered('b');
        app.on_letter_entered('c');

        assert_eq!(app.input, "abc")
    }

    #[test]
    fn test_on_invalid_letter_entered() {
        let mut app = setup_app();

        app.on_letter_entered('a');
        app.on_letter_entered('#');
        app.on_letter_entered(' ');

        assert_eq!(app.input, "a")
    }

    #[test]
    fn test_on_enter_press() {
        
    }

    #[test]
    fn test_on_enter_press_short_word() {

    }

    #[test]
    fn test_on_enter_press_invalid_word() {

    }
}
