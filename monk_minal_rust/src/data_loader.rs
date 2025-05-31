//! # Data Loading Module
//!
//! This module is responsible for loading external data required by the MonkMinal Rust application.
//! Currently, it handles loading lists of words for typing tests and quotes for the quote typing mode.
//! Data is loaded from JSON files embedded in the binary at compile time using `include_str!`.

use anyhow::Result;
use serde::Deserialize;

/// Represents the structure of `allWords.json`.
///
/// Contains a single field `words` which is a vector of strings.
#[derive(Deserialize, Debug)]
pub struct AllWords {
    /// A list of words to be used in typing games.
    words: Vec<String>,
}

/// Represents the structure of a single quote in `quotes.json`.
///
/// Each quote has a text content and a source.
#[derive(Deserialize, Debug, Clone)]
pub struct Quote {
    /// The text content of the quote.
    pub text: String,
    /// The source or author of the quote.
    pub source: String,
}

// Note on `include_str!`:
// The paths used in `include_str!` are relative to the current source file (`src/data_loader.rs`).
// - `../../data/allWords.json` translates to `<project_root>/data/allWords.json`.
// - `../../data/quotes.json` translates to `<project_root>/data/quotes.json`.
// Cargo is configured to rebuild the crate if these external files change, ensuring
// the binary always includes the latest version of the data.

/// Loads all words from the embedded `allWords.json` file.
///
/// The JSON file is expected to have a single key "words" containing a list of strings.
///
/// # Returns
///
/// Returns a `Result<Vec<String>>` which is `Ok` with a vector of words if loading and
/// parsing are successful, or an `Err` if the file cannot be read or parsed.
pub fn load_all_words() -> Result<Vec<String>> {
    // Embed the content of allWords.json directly into the binary at compile time.
    // If allWords.json changes, Cargo will rebuild the crate.
    let words_json = include_str!("../../data/allWords.json"); 
    let all_words_data: AllWords = serde_json::from_str(words_json)?; // Parse the JSON string.
    Ok(all_words_data.words) // Return the list of words.
}

/// Loads all quotes from the embedded `quotes.json` file.
///
/// The JSON file is expected to be an array of objects, each with "text" and "source" fields.
///
/// # Returns
///
/// Returns a `Result<Vec<Quote>>` which is `Ok` with a vector of quotes if loading and
/// parsing are successful, or an `Err` if the file cannot be read or parsed.
pub fn load_quotes() -> Result<Vec<Quote>> {
    // Embed the content of quotes.json directly into the binary at compile time.
    // If quotes.json changes, Cargo will rebuild the crate.
    let quotes_json = include_str!("../../data/quotes.json");
    let quotes_data: Vec<Quote> = serde_json::from_str(quotes_json)?; // Parse the JSON string.
    Ok(quotes_data) // Return the list of quotes.
}
