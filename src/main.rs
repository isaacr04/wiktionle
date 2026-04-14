mod app;
mod engine;
mod events;
mod theme;
mod ui;
#[path = "./word_list_manager.rs"]
mod word_list_manager;

use crate::app::{App, AppOptions};
use crate::engine::{GameDifficulty, GameOptions, WordSelectionMode};
use crate::events::{AppEvent, Events};
use crate::theme::Theme;

use clap::Parser;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::io;
use std::time::Duration;
use tui::{Terminal, backend::CrosstermBackend};

#[derive(Parser, Debug)]
#[clap(about = "Wordlet is a command line Wordle clone.", version, author)]
struct Args {
    #[clap(
        short,
        long,
        default_value = "easy",
        help = "Change the game's difficulty. Valid values are easy and hard"
    )]
    difficulty: String,

    #[clap(
        short,
        long,
        default_value = "dark",
        help = "Change the display colors. Valid values are light and dark"
    )]
    theme: String,

    #[clap(
        short,
        long,
        default_value_t = 5,
        help = "Change the display colors. Valid values are any integer greater than 0"
    )]
    word_length: usize,

    #[clap(
        long,
        default_value = "random",
        help = "Word selection mode. Valid values are random and most-recent"
    )]
    word_selection: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;

    let args = Args::parse();
    let difficulty = match args.difficulty.as_ref() {
        "hard" => GameDifficulty::Hard,
        _ => GameDifficulty::Easy,
    };

    let theme = match args.theme.as_ref() {
        "light" => Theme::light_theme(),
        _ => Theme::dark_theme(),
    };

    let word_selection = match args.word_selection.as_ref() {
        "most-recent" => WordSelectionMode::MostRecent,
        _ => WordSelectionMode::RandomByLength,
    };

    let word_length = args.word_length;

    let mut app = App::new(AppOptions {
        theme,
        game_config: GameOptions {
            answer: None,
            difficulty,
            word_length,
            word_selection,
        },
    });

    let tick_rate = Duration::from_millis(100);
    let events = Events::new(tick_rate);

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    loop {
        let current_word_length = app.game.word_length();

        terminal.draw(|frame| {
            let _r = ui::draw(frame, &mut app, current_word_length);
        })?;

        match events.next()? {
            AppEvent::Input(event) => app.on_key(event),
            AppEvent::Tick => {}
        }

        if app.should_quit {
            disable_raw_mode()?;
            terminal.clear()?;
            terminal.show_cursor()?;
            break;
        }
    }

    Ok(())
}
