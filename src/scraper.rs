//! # wiktionle-scraper
//!
//! Scrapes Wiktionary's Word of the Day chronological index and saves each
//! entry to a JSON file managed by [`WordListManager`].
//!
//! ## Usage
//! ```
//! cargo run --bin scraper -- [OPTIONS]
//!
//! Options:
//!   -s, --start-date <YYYY-MM>   Earliest date to include (inclusive)
//!   -e, --end-date   <YYYY-MM>   Latest  date to include (inclusive)
//!   -o, --output     <FILE>         Output JSON path  [default: wotd_words.json]
//! ```
//!
//! Existing records in the JSON file are always skipped, so the scraper is
//! safe to re-run incrementally.

#[path = "./word_list_manager.rs"]
mod word_list_manager;
use word_list_manager::{WordEntry, WordListManager};

use chrono::{Datelike, NaiveDate, Utc};
use clap::Parser;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

use std::thread;
use std::time::Duration;

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

const ARCHIVE_BASE: &str =
    "https://en.wiktionary.org/wiki/Wiktionary:Word_of_the_day/Archive";

const DEFAULT_OUTPUT: &str = "wotd_words.json";

const REQUEST_DELAY_MS: u64 = 600;
const BATCH_SIZE: usize = 50;

const MONTHS: [&str; 12] = [
    "January","February","March","April","May","June",
    "July","August","September","October","November","December"
];

// ─────────────────────────────────────────────────────────────────────────────
// CLI arguments
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Parser, Debug)]
#[clap(
    name = "wiktionle-scraper",
    about = "Scrapes Wiktionary Word-of-the-Day entries into a JSON word list."
)]
struct Args {

    #[clap(short, long, value_name = "YYYY-MM")]
    start_date: Option<String>,

    #[clap(short, long, value_name = "YYYY-MM")]
    end_date: Option<String>,

    #[clap(short, long, default_value = DEFAULT_OUTPUT)]
    output: String,
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args = Args::parse();

    let start_date = args
        .start_date
        .as_deref()
        .map(parse_year_month)
        .transpose()?;

    let end_date = args
        .end_date
        .as_deref()
        .map(parse_year_month)
        .transpose()?;

    let mut manager = WordListManager::new(&args.output)?;

    if let Some((first,last)) = manager.get_date_range() {
        println!(
            "Loaded existing file: {} entries ({} – {})",
            manager.count(),
            first,
            last
        );
    } else {
        println!("No existing word list found.");
    }

    let client = Client::builder()
        .user_agent(
            "wiktionle-scraper/2.0 (educational project; \
             https://github.com/isaacr04/wiktionle)"
        )
        .timeout(Duration::from_secs(30))
        .build()?;

    println!("\nScanning monthly Wiktionary archives…");

    let current_year = Utc::now().year();
    let mut entries_raw: Vec<(NaiveDate,String)> = Vec::new();

    for year in 2006..=current_year {

        for month_name in MONTHS {

            let month_num = month_name_to_number(month_name).unwrap();

            if !month_in_range(year, month_num, start_date, end_date) {
                continue;
            }

            let url = format!("{ARCHIVE_BASE}/{year}/{month_name}");
            println!("Fetching {url}");

            thread::sleep(Duration::from_millis(REQUEST_DELAY_MS));

            let html = match client.get(&url).send() {
                Ok(resp) => resp.text()?,
                Err(_) => continue
            };

            let mut parsed = parse_month_archive(&html, year, month_name);
            entries_raw.append(&mut parsed);
        }
    }

    let to_process: Vec<(NaiveDate,String)> = entries_raw
        .into_iter()
        .filter(|(date,_)| !manager.has_date(*date))
        .collect();

    println!(
        "\n{} entries to process (new, within date range).\n",
        to_process.len()
    );

    let total = to_process.len();
    let mut buffer: Vec<WordEntry> = Vec::with_capacity(BATCH_SIZE);

    for (idx,(date,html_block)) in to_process.into_iter().enumerate() {

        println!("[{:>4}/{total}] {}", idx+1, date);

        match parse_wotd_block(&html_block, date) {
            Ok(entry) => buffer.push(entry),
            Err(e) => eprintln!("Failed: {e}")
        }

        if buffer.len() >= BATCH_SIZE {
            let batch = std::mem::take(&mut buffer);
            let n = manager.add_entries(batch)?;
            println!("  — Saved batch of {n} entries.");
        }
    }

    if !buffer.is_empty() {
        let n = manager.add_entries(buffer)?;
        println!("  — Saved final batch of {n} entries.");
    }

    println!("\nComplete. {} entries in '{}'.", manager.count(), args.output);

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Archive parsing
// ─────────────────────────────────────────────────────────────────────────────

fn parse_month_archive(
    html: &str,
    year: i32,
    month_name: &str
) -> Vec<(NaiveDate,String)> {

    let doc = Html::parse_document(html);

    let entry_sel = Selector::parse(".mf-wotd").unwrap();
    let date_sel = Selector::parse("#WOTD-rss-date").unwrap();

    let mut entries = Vec::new();
    let month = month_name_to_number(month_name).unwrap();

    for block in doc.select(&entry_sel) {

        let date_el = match block.select(&date_sel).next() {
            Some(d) => d,
            None => continue
        };

        let date_text = date_el.text().collect::<String>();

        let day: u32 = match date_text.split_whitespace().last() {
            Some(d) => d.parse().unwrap_or(0),
            None => continue
        };

        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            entries.push((date, block.html()));
        }
    }

    entries
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry parsing
// ─────────────────────────────────────────────────────────────────────────────

fn parse_wotd_block(
    html: &str,
    date: NaiveDate
) -> Result<WordEntry,Box<dyn std::error::Error>> {

    let doc = Html::parse_fragment(html);

    let word_sel = Selector::parse("#WOTD-rss-title").unwrap();
    let pos_sel = Selector::parse("i").unwrap();
    let def_sel = Selector::parse("#WOTD-rss-description li").unwrap();

    let word = doc
        .select(&word_sel)
        .next()
        .ok_or("missing word")?
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    let word_class = doc
        .select(&pos_sel)
        .next()
        .map(|x| x.text().collect::<String>())
        .unwrap_or_else(|| "unknown".to_string());

    let definition = doc
        .select(&def_sel)
        .next()
        .map(|x| x.text().collect::<String>())
        .unwrap_or_default();

    Ok(WordEntry::new(
        word,
        date,
        word_class.trim().to_string(),
        clean_definition_text(&definition),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn parse_year_month(s: &str) -> Result<(i32,u32),Box<dyn std::error::Error>> {

    let parts: Vec<&str> = s.split('-').collect();

    if parts.len() != 2 {
        return Err("Expected YYYY-MM".into());
    }

    let year: i32 = parts[0].parse()?;
    let month: u32 = parts[1].parse()?;

    Ok((year,month))
}

fn month_in_range(
    year: i32,
    month: u32,
    start: Option<(i32,u32)>,
    end: Option<(i32,u32)>
) -> bool {

    if let Some((sy,sm)) = start {
        if (year,month) < (sy,sm) {
            return false;
        }
    }

    if let Some((ey,em)) = end {
        if (year,month) > (ey,em) {
            return false;
        }
    }

    true
}

fn month_name_to_number(name: &str) -> Option<u32> {
    match name {
        "January" => Some(1),
        "February" => Some(2),
        "March" => Some(3),
        "April" => Some(4),
        "May" => Some(5),
        "June" => Some(6),
        "July" => Some(7),
        "August" => Some(8),
        "September" => Some(9),
        "October" => Some(10),
        "November" => Some(11),
        "December" => Some(12),
        _ => None
    }
}

fn clean_definition_text(raw: &str) -> String {

    let mut out = String::with_capacity(raw.len());
    let mut in_bracket = 0;

    for ch in raw.chars() {
        match ch {
            '[' => in_bracket += 1,
            ']' if in_bracket > 0 => in_bracket -= 1,
            _ if in_bracket == 0 => out.push(ch),
            _ => {}
        }
    }

    out.split_whitespace().collect::<Vec<_>>().join(" ")
}