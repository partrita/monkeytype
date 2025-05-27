//! # MonkMinal Rust
//!
//! MonkMinal Rust is a terminal-based typing tutor application inspired by MonkeyType.
//! It provides various game modes to help users improve their typing speed and accuracy.
//! This is the main entry point of the application.

use anyhow::Result;
use clap::Parser;
use colored::*;
use figlet_rs::FIGfont;
// log crate for logging errors
use log::{error, info, warn, debug, trace};


// Modules defining different parts of the application
pub mod config; 
pub mod data_loader;
pub mod game;

/// Command Line Interface arguments for MonkMinal Rust.
///
/// Uses `clap` for parsing and automatically provides `--version` and `--help`.
#[derive(Parser, Debug)]
#[clap(author = "shikhar13012001", version = "0.1.0", about = "A terminal-based typing tutor written in Rust.", long_about = None)]
struct CliArgs {
    // No explicit arguments are defined here for now, as clap handles --version and --help.
    // Future arguments like specific game modes or configurations could be added.
}

/// Main entry point for the MonkMinal Rust application.
///
/// This function performs the following steps:
/// 1. Parses command line arguments (currently only handles `--version` and `--help` via `clap`).
/// 2. Displays a welcome banner.
/// 3. Prompts the user for game configuration using `dialoguer`.
/// 4. Loads necessary game data (words, quotes) from JSON files.
/// 5. Starts and runs the main game loop.
/// 6. Handles errors that occur during gameplay and ensures the terminal is reset.
fn main() -> Result<()> {
    // For this simplified logging, we are not using an external logger facade like env_logger.
    // Log messages will go to stderr by default if not captured by a more sophisticated logger.
    // To actually see log::info etc. you would typically need a logger initialized.
    // However, log::error! will print to stderr regardless of RUST_LOG.
    // For now, we focus on log::error! for critical failures.

    // Parse command-line arguments. Clap handles --version and --help automatically.
    let _args = CliArgs::parse(); 

    // Display the application welcome banner using Figlet.
    let standard_font = FIGfont::standard().unwrap_or_else(|_| FIGfont::from_content("MonkMinal").unwrap_or_default());
    let figure = standard_font.convert("MonkMinal");
    println!("{}", figure.unwrap_or_default().to_string().cyan());
    println!(); 

    // Print application title, version, author, and description.
    println!(
        "{} {}",
        "monk-minal".green().bold(),
        format!("v{}", env!("CARGO_PKG_VERSION")).dimmed()
    );
    println!("{}{}", "by ".dimmed(), env!("CARGO_PKG_AUTHORS").italic());
    println!("{}", env!("CARGO_PKG_DESCRIPTION").italic().dimmed());
    println!(); 

    // Get game configuration from the user.
    let game_config = match config::get_game_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            error!("Failed to get game configuration: {}", e);
            // Attempt to reset terminal if dialoguer left it in a weird state (though it usually handles this)
            use crossterm::{execute, terminal, cursor};
            let mut stderr_temp = std::io::stderr(); 
            execute!(stderr_temp, cursor::Show).ok(); 
            terminal::disable_raw_mode().ok();
            return Err(e.context("Configuration failed")); // Propagate error
        }
    };
    println!(); // Add spacing after dialoguer prompts.

    // Load game data (words and quotes).
    let all_words = match data_loader::load_all_words() {
        Ok(words) => words,
        Err(e) => {
            error!("Failed to load words data: {}", e);
            return Err(e.context("Loading words failed"));
        }
    };
    let all_quotes = match data_loader::load_quotes() {
        Ok(quotes) => quotes,
        Err(e) => {
            error!("Failed to load quotes data: {}", e);
            return Err(e.context("Loading quotes failed"));
        }
    };

    // Run the game with the chosen configuration and loaded data.
    if let Err(e) = game::run_game(game_config, all_words, all_quotes) {
        // Log the error using the log crate.
        // The error `e` from run_game should be an anyhow::Error, which includes context.
        error!("Game error: {:?}", e); // {:?} for full context from anyhow
        
        // `run_game` should ideally handle its own terminal teardown on error.
        // This is a fallback.
        use crossterm::{execute, terminal, cursor};
        let mut stderr_temp = std::io::stderr(); 
        execute!(stderr_temp, cursor::Show).ok(); 
        terminal::disable_raw_mode().ok(); 
        std::process::exit(1); // Exit with an error code
    }
    
    Ok(())
}
