//! # Game Configuration Module
//!
//! This module handles the interactive configuration process for the MonkMinal Rust game.
//! It defines the available game types, difficulty levels, and the structure for storing
//! the chosen game configuration. The primary function `get_game_config` uses `dialoguer`
//! to prompt the user for their desired settings.

use anyhow::Result;
use dialoguer::{theme::ColorfulTheme, Select, Input}; // Input is not used but was considered.
use serde::{Serialize, Deserialize}; // For potential future config saving/loading.

/// Defines the different types of games available.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum GameType {
    /// Game mode where the user types for a fixed duration.
    Time,
    /// Game mode where the user types a fixed number of words.
    Words,
    /// Game mode where the user types a specific quote.
    Quote,
}

/// Defines the difficulty levels for the game.
/// Difficulty can affect word length or other game parameters.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Difficulty {
    /// Easy difficulty: typically shorter words or simpler text.
    Easy,
    /// Medium difficulty: standard word lengths or text complexity.
    Medium,
    /// Hard difficulty: typically longer words or more complex text.
    Hard,
}

/// Stores the user's chosen game configuration.
///
/// This struct is populated by `get_game_config` based on user input.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameConfig {
    /// The type of game selected by the user (e.g., Time, Words, Quote).
    pub game_type: GameType,
    /// Optional duration in seconds for `GameType::Time`. `None` for other types.
    pub time_seconds: Option<u32>,
    /// Optional number of words for `GameType::Words`. `None` for other types.
    pub word_count: Option<u32>,
    /// The difficulty level selected by the user.
    pub difficulty: Difficulty,
}

impl GameConfig {
    /// Creates a new `GameConfig` with default values.
    /// These defaults are typically overwritten by user selections.
    pub fn new() -> Self {
        // Default values are set here, but `get_game_config` will guide the user
        // to set their preferred options.
        GameConfig {
            game_type: GameType::Time, // Default game type
            time_seconds: Some(30),    // Default time for Time mode
            word_count: None,          // No default word count for Words mode (user must choose)
            difficulty: Difficulty::Medium, // Default difficulty
        }
    }
}

/// Prompts the user to select game configuration options interactively.
///
/// Uses `dialoguer` to present menus for game type, time/word count (if applicable),
/// and difficulty.
///
/// # Returns
///
/// Returns a `Result<GameConfig>` which is `Ok` if the user successfully completes
/// the configuration, or an `Err` if an error occurs during the interaction (e.g., user cancels).
pub fn get_game_config() -> Result<GameConfig> {
    let theme = ColorfulTheme::default(); // Use dialoguer's colorful theme for prompts.
    let mut config = GameConfig::new(); // Initialize with default config.

    // 1. Pick game type
    let game_types = ["Time", "Words", "Quote"];
    let selection_idx = Select::with_theme(&theme)
        .with_prompt("Pick a game type:")
        .items(&game_types)
        .default(0) // Default to "Time"
        .interact()?; // This can return an error if the user cancels (e.g., Esc)

    match game_types[selection_idx] {
        "Time" => {
            config.game_type = GameType::Time;
            let time_options = ["15s", "30s", "60s", "120s"];
            let time_selection_idx = Select::with_theme(&theme)
                .with_prompt("Pick a time limit:")
                .items(&time_options)
                .default(1) // Default to "30s"
                .interact()?;
            
            // Parse the selected time string (e.g., "30s") into u32.
            let time_str = time_options[time_selection_idx].trim_end_matches('s');
            config.time_seconds = Some(time_str.parse::<u32>()?); // This can fail if parse is invalid.
            config.word_count = None; // Ensure word_count is None for Time mode.
        }
        "Words" => {
            config.game_type = GameType::Words;
            let word_count_options = ["10", "20", "30", "40", "50"];
            let count_selection_idx = Select::with_theme(&theme)
                .with_prompt("Pick a number of words:")
                .items(&word_count_options)
                .default(1) // Default to "20" words
                .interact()?;

            // Parse the selected word count string into u32.
            config.word_count = Some(word_count_options[count_selection_idx].parse::<u32>()?);
            config.time_seconds = None; // Ensure time_seconds is None for Words mode.
        }
        "Quote" => {
            config.game_type = GameType::Quote;
            // For Quote mode, specific options like choosing a quote source or length
            // could be added here in the future.
            config.time_seconds = None;
            config.word_count = None;
            // Inform user that quote selection is not yet implemented if desired.
            // println!("{}", "Quote mode selected. Specific quote selection will be added later.".italic());
        }
        _ => unreachable!(), // This case should not be reached due to `Select` behavior.
    }

    // 2. Pick difficulty
    let difficulties = ["Easy", "Medium", "Hard"];
    let difficulty_selection_idx = Select::with_theme(&theme)
        .with_prompt("Pick a difficulty:")
        .items(&difficulties)
        .default(1) // Default to "Medium"
        .interact()?;

    config.difficulty = match difficulties[difficulty_selection_idx] {
        "Easy" => Difficulty::Easy,
        "Medium" => Difficulty::Medium,
        "Hard" => Difficulty::Hard,
        _ => unreachable!(), // Should not be reached.
    };
    
    Ok(config) // Return the populated GameConfig.
}
