# Wiktionle 

A educational Wordle clone that uses Wiktionary's historical "Word of the Day" entries to help you discover and learn new vocabulary.

![gameplay](https://user-images.githubusercontent.com/3178471/150548930-9dab1e11-2997-48da-af33-6e3386017a50.gif)

## Installation

### Requirements
+ [Rust](https://rust-lang.org/tools/install/)

### Steps
1. Clone Wiktionle Repository 
2. Open terminal inside of repository directory.
3. Run Wiktionle with the following command:
```
cargo run --bin wiktionle
```
4. Update wordlist with the following command:
```
cargo run --bin scraper
```

### Configuration
Additional options may be passed to `cargo run --bin wikitionle` to customize your playing experience using the following format
```
cargo run --bin wiktionle -- [OPTIONS]
```

```
OPTIONS
The following options can be applied to Wiktionle to customize your experience.

   --theme <THEME>                     THEME: dark, light

   --difficulty <DIFFICULTY>           DIFFICULTY: easy, hard

   --word-length <WORD_LENGTH>         WORD_LENGTH: numeric value

   --word-selection <WORD_SELECTION>   WORD_SELECTION: most-recent, random-by-length,
```

```
EXAMPLE
Starting the wiktionle with a light them and a word length of 3.

   $ cargo run --bin wiktionle -- --theme light --word-length 3
```

### How to Run tests
Run all tests using the command:
```
cargo test --no-fail-fast
```

To report test coverage run the following commands:
```
cargo install cargo-llvm-cov
cargo llvm-cov --no-fail-fast --fail-under-lines 80 --no-report -- --format=terse 
cargo llvm-cov report --ignore-filename-regex="words.rs|main.rs|events.rs|ui.rs"
```

#### Reasons for excluding files from coverage report
+ **words.rs**: it is a file simply filled with thousands of lines of hardcoded strings which are no longer needed by this program as we no longer hard code the word list.
+ **main.rs**, **events.rs**, **ui.rs**: These files handle handle input and graphical output from the terminal which we do not have availability to test to through github actions or automatically running our tests.


## Features

- **Educational Hints**: Get helpful clues with the word's part of speech and primary definition
- **Learn as You Play**: View the full Wiktionary entry and the original Word of the Day date after solving
- **Multiple Difficulty Levels**: Choose custom word lengths to randomly pick from matching past Words of the Day
- **Daily Challenge**: Play today's actual Wiktionary Word of the Day
- **Vocabulary Expansion**: Discover words curated by Wiktionary editors as particularly noteworthy

## How to Play

1. Guess the 5-letter word in 6 tries (or customize word length)
2. After each guess, colors show how close you are:
   - 🟩 **Green**: Correct letter in the correct position
   - 🟨 **Yellow**: Correct letter in the wrong position
   - ⬜ **Gray**: Letter not in the word
3. Use hints (word class and definition) if you get stuck
4. Learn the full definition and see when it was Word of the Day after solving!

## Credits

Project extend from [sammylupt's 'wordlet' project](https://github.com/sammylupt/wordlet)

### Usage of Third Party Materials
ui.rs includes code from:
   - [minesweep-rs](https://github.com/cpcloud/minesweep-rs/blob/main/src/ui.rs)

events.rs includes code from 
   - [battleship-rs](https://github.com/deepu105/battleship-rs/blob/main/src/event.rs)
   - [rust-commandline-example](https://github.com/zupzup/rust-commandline-example/blob/main/src/main.rs)

Please see licenses in docs/third_party_licenses for each repository. Notices are included in each file.