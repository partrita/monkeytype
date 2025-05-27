//! # Core Game Logic Module
//!
//! This module orchestrates the main typing game experience in MonkMinal Rust.
//! It manages the game state, processes user input, calculates performance metrics (WPM, accuracy),
//! and handles the display of the game interface and game over screen.
//!
//! ## Potential Refactor:
//! The UI rendering parts could be moved to a dedicated `ui.rs` module for better SoC.

use crate::config::{GameConfig, GameType, Difficulty};
use crate::data_loader::Quote;
use anyhow::{Result, anyhow, Context}; // Added anyhow! and Context
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers}, 
    execute,
    style::{Print}, 
    cursor,
    terminal,
};
use figlet_rs::FIGfont; 
use rand::seq::SliceRandom; 
use std::io::{stdout, Stdout, Write}; 
use std::time::{Duration, Instant}; 
use textwrap::wrap; 
use colored::Colorize; 
use log::{warn, debug, trace}; // Added log macros

/// Standard word length used for WPM calculation (average characters per word).
const STANDARD_WORD_LENGTH: f64 = 5.0;

/// Represents the current state of the typing game.
#[derive(Debug)]
pub struct GameState {
    /// The complete list of words selected for the current game session.
    pub words_to_type: Vec<String>,
    /// Index of the current word the user is expected to type from `words_to_type`.
    pub current_word_index: usize,
    /// Index of the current character within the current word.
    pub current_char_index: usize, 
    /// The characters typed by the user for the current word so far that are correct.
    pub user_input: String,      
    /// The characters typed by the user for the current word that are incorrect.
    pub errors: String, 
    /// Timestamp of when the game (typing) officially started.
    pub start_time: Option<Instant>,
    /// The configuration for the current game session.
    pub config: GameConfig,
    /// All words loaded from `allWords.json`.
    pub all_loaded_words: Vec<String>, 
    /// All quotes loaded from `quotes.json`.
    pub all_loaded_quotes: Vec<Quote>,
    /// Total number of characters correctly typed by the user across all words.
    pub correct_chars_total: usize, 
    /// Total number of characters (correct or incorrect) attempted by the user.
    pub typed_chars_total: usize,
    /// Flag indicating whether the game has ended.
    pub game_over: bool,
    /// Stores the final elapsed time in seconds when the game ends.
    pub final_elapsed_time_seconds: Option<f64>,
}

impl GameState {
    /// Creates a new `GameState` instance.
    pub fn new(
        config: GameConfig,
        all_loaded_words: Vec<String>,
        all_loaded_quotes: Vec<Quote>,
        words_for_current_game: Vec<String>,
    ) -> Self {
        GameState {
            words_to_type: words_for_current_game,
            current_word_index: 0,
            current_char_index: 0,
            user_input: String::new(),
            errors: String::new(),
            start_time: None,
            config,
            all_loaded_words,
            all_loaded_quotes,
            correct_chars_total: 0,
            typed_chars_total: 0,
            game_over: false,
            final_elapsed_time_seconds: None,
        }
    }
}

/// Calculates Words Per Minute (WPM) and accuracy.
pub fn calculate_wpm(correct_chars: usize, total_chars_typed: usize, time_seconds: f64) -> (f64, f64, f64) {
    if time_seconds < 0.01 || total_chars_typed == 0 { 
        let accuracy = if total_chars_typed == 0 { 100.0 } else { (correct_chars as f64 / total_chars_typed as f64) * 100.0 };
        return (0.0, 0.0, accuracy);
    }
    let time_in_minutes = time_seconds / 60.0;
    let gross_wpm = (total_chars_typed as f64 / STANDARD_WORD_LENGTH) / time_in_minutes;
    let errors_count = total_chars_typed.saturating_sub(correct_chars);
    let error_penalty_wpm = errors_count as f64 / time_in_minutes;
    let net_wpm = (gross_wpm - error_penalty_wpm).max(0.0); 
    let accuracy = (correct_chars as f64 / total_chars_typed as f64) * 100.0;
    (gross_wpm, net_wpm, accuracy)
}

/// Selects words or quote text for the game based on the `GameConfig`.
pub fn get_words_for_game(
    config: &GameConfig,
    all_words: &[String],
    all_quotes: &[Quote],
) -> Result<Vec<String>> {
    let mut rng = rand::thread_rng(); 
    match config.game_type {
        GameType::Quote => {
            if all_quotes.is_empty() {
                return Err(anyhow!("No quotes available for Quote mode. Please check data/quotes.json."));
            }
            let chosen_quote = all_quotes.choose(&mut rng)
                .ok_or_else(|| anyhow!("Failed to choose a quote, though list was not empty."))?;
            Ok(chosen_quote.text.split_whitespace().map(String::from).collect())
        }
        GameType::Time | GameType::Words => {
            if all_words.is_empty() {
                return Err(anyhow!("No words available for selected game mode. Please check data/allWords.json."));
            }
            let count = match config.game_type {
                GameType::Time => 300, 
                GameType::Words => config.word_count.unwrap_or(30) as usize,
                _ => unreachable!(),
            };
            
            let mut filtered_words: Vec<String> = match config.difficulty {
                Difficulty::Easy => all_words.iter().filter(|w| w.len() <= 5).cloned().collect(),
                Difficulty::Medium => all_words.iter().filter(|w| w.len() <= 8).cloned().collect(),
                Difficulty::Hard => all_words.to_vec(),
            };

            if filtered_words.is_empty() { 
                // If filtering results in an empty list (e.g. no easy words), use all available words.
                // Consider if this should be an error or a fallback. For now, fallback.
                warn!("No words found for difficulty {:?}, falling back to all available words.", config.difficulty);
                filtered_words = all_words.to_vec();
                if filtered_words.is_empty() { // Double check if all_words itself was empty after fallback attempt
                     return Err(anyhow!("No words available after difficulty filtering and fallback. Check data/allWords.json."));
                }
            }
            
            let num_to_choose = if filtered_words.len() < count { filtered_words.len() } else { count };
            if num_to_choose == 0 { // If after all filtering and selection, we have no words to choose.
                 return Err(anyhow!("No words could be selected for the game with current criteria (count: {}, available: {}).", count, filtered_words.len()));
            }
            
            Ok(filtered_words.choose_multiple(&mut rng, num_to_choose).cloned().collect())
        }
    }
}

/// Displays the main game interface (typing area, stats, timer).
fn display_game_interface(stdout: &mut Stdout, game_state: &GameState, terminal_width: u16, terminal_height: u16) -> Result<()> {
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    let elapsed_seconds = game_state.start_time.map_or(0.0, |st| st.elapsed().as_secs_f64());
    let mut header_lines: Vec<String> = Vec::new();
    let timer_display = if game_state.config.game_type == GameType::Time {
        let total_duration = game_state.config.time_seconds.unwrap_or(0) as f64;
        let remaining_time = (total_duration - elapsed_seconds).max(0.0);
        format!("Time Left: {:02}:{:02}", (remaining_time / 60.0).floor() as u32, (remaining_time % 60.0).floor() as u32)
    } else {
        format!("Time Elapsed: {:02}:{:02}", (elapsed_seconds / 60.0).floor() as u32, (elapsed_seconds % 60.0).floor() as u32)
    };
    header_lines.push(timer_display);
    if game_state.start_time.is_some() && elapsed_seconds > 0.01 {
        let (gross_wpm, net_wpm, accuracy) = calculate_wpm(
            game_state.correct_chars_total, game_state.typed_chars_total, elapsed_seconds);
        header_lines.push(format!("Gross WPM: {:.0} | Net WPM: {:.0} | Accuracy: {:.2}%", gross_wpm, net_wpm, accuracy));
    } else {
        header_lines.push("Gross WPM: - | Net WPM: - | Accuracy: -%".to_string());
    }
    for (i, line) in header_lines.iter().enumerate() {
        let padding = (terminal_width.saturating_sub(line.len() as u16)) / 2;
        execute!(stdout, cursor::MoveTo(padding, i as u16), Print(line))?;
    }
    const MAX_WORDS_TO_DISPLAY: usize = 15; 
    const APPROX_CHARS_WINDOW: usize = 60;  
    let start_idx = game_state.current_word_index.saturating_sub(MAX_WORDS_TO_DISPLAY / 3);
    let mut end_idx = start_idx;
    let mut current_len_chars = 0;
    for i in start_idx..game_state.words_to_type.len() {
        if i >= game_state.words_to_type.len() { end_idx = game_state.words_to_type.len(); break; }
        current_len_chars += game_state.words_to_type[i].len() + 1; 
        if current_len_chars > APPROX_CHARS_WINDOW && i > game_state.current_word_index { end_idx = i; break; }
        end_idx = i + 1;
    }
    if end_idx == start_idx && end_idx < game_state.words_to_type.len() { end_idx = start_idx + 1; }
    let display_words_slice = if !game_state.words_to_type.is_empty() {
        &game_state.words_to_type[start_idx..end_idx.min(game_state.words_to_type.len())]
    } else { &[] };
    let mut display_string_parts: Vec<String> = Vec::new();
    for (i_slice, word) in display_words_slice.iter().enumerate() {
        let actual_word_idx = start_idx + i_slice;
        if actual_word_idx == game_state.current_word_index {
            let target_word = &game_state.words_to_type[game_state.current_word_index];
            if !game_state.user_input.is_empty() { display_string_parts.push(format!("{}", game_state.user_input.green())); }
            if !game_state.errors.is_empty() { display_string_parts.push(format!("{}", game_state.errors.on_red())); }
            if game_state.current_char_index < target_word.len() {
                let current_char_str = target_word.chars().nth(game_state.current_char_index).unwrap().to_string();
                if game_state.errors.is_empty() { display_string_parts.push(format!("{}", current_char_str.black().on_yellow())); }
                else { display_string_parts.push(format!("{}", current_char_str.dimmed())); }
                if game_state.current_char_index + 1 < target_word.len() {
                    display_string_parts.push(format!("{}", (&target_word[(game_state.current_char_index + 1)..]).dimmed()));
                }
            }
        } else { display_string_parts.push(format!("{}", word.dimmed())); }
        display_string_parts.push(" ".to_string()); 
    }
    if !display_string_parts.is_empty() { display_string_parts.pop(); }
    let full_display_line = display_string_parts.join("");
    let wrap_width = (terminal_width.saturating_sub(4)).max(10) as usize;
    let wrapped_text_lines = wrap(&full_display_line, wrap_width);
    let header_height = header_lines.len() as u16;
    let footer_height = 1u16; 
    let available_height_for_text = terminal_height.saturating_sub(header_height).saturating_sub(footer_height);
    let text_display_start_row = header_height + available_height_for_text.saturating_sub(wrapped_text_lines.len() as u16) / 2;
    for (i, line) in wrapped_text_lines.iter().enumerate() {
        let padding = (terminal_width.saturating_sub(line.len() as u16)) / 2;
        execute!(stdout, cursor::MoveTo(padding, text_display_start_row + i as u16), Print(line))?;
    }
    let quit_msg = "Press Esc to quit";
    let quit_msg_padding = (terminal_width.saturating_sub(quit_msg.len() as u16)) / 2;
    execute!(stdout, cursor::MoveTo(quit_msg_padding, terminal_height - 1), Print(quit_msg))?;
    stdout.flush()?; 
    Ok(())
}

/// Displays the game over screen with final statistics.
fn display_game_over_screen(stdout: &mut Stdout, game_state: &GameState, terminal_width: u16, terminal_height: u16) -> Result<()> {
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    let font = FIGfont::standard().unwrap_or_else(|_| FIGfont::from_content("Game Over!").expect("Figlet fallback font failed")); 
    let game_over_banner = font.convert("Game Over!").unwrap_or_default().to_string();
    let mut lines_to_display: Vec<String> = Vec::new();
    for line in game_over_banner.lines() { lines_to_display.push(line.to_string()); }
    lines_to_display.push("".to_string()); 
    let final_time = game_state.final_elapsed_time_seconds.unwrap_or_else(|| 
        game_state.start_time.map_or(0.0, |st| st.elapsed().as_secs_f64()));
    let (gross_wpm, net_wpm, accuracy) = calculate_wpm(
        game_state.correct_chars_total, game_state.typed_chars_total, final_time);
    lines_to_display.push(format!("Gross WPM: {:.0}", gross_wpm));
    lines_to_display.push(format!("Net WPM:   {:.0}", net_wpm));
    lines_to_display.push(format!("Accuracy:  {:.2}%", accuracy));
    lines_to_display.push(format!("Time Taken: {:02}:{:02}", (final_time / 60.0).floor() as u32, (final_time % 60.0).floor() as u32));
    lines_to_display.push("".to_string()); 
    lines_to_display.push("Press any key to return to main menu.".to_string());
    let total_lines_height = lines_to_display.len() as u16;
    let start_row = terminal_height.saturating_sub(total_lines_height) / 2;
    for (i, line) in lines_to_display.iter().enumerate() {
        let padding = (terminal_width.saturating_sub(line.len() as u16)) / 2;
        execute!(stdout, cursor::MoveTo(padding, start_row + i as u16), Print(line))?;
    }
    stdout.flush()?;
    Ok(())
}

/// Runs the main game loop, handling user input, game state updates, and rendering.
pub fn run_game(config: GameConfig, all_words: Vec<String>, all_quotes: Vec<Quote>) -> Result<()> {
    let mut stdout = stdout();
    terminal::enable_raw_mode().context("Failed to enable raw mode")?;
    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::Hide).context("Failed to clear screen or hide cursor")?;

    let words_for_game = get_words_for_game(&config, &all_words, &all_quotes)
        .with_context(|| format!("Failed to get words for game with config: {:?}", config))?;
    
    // This check is now more robust as get_words_for_game returns Err if no words can be selected.
    if words_for_game.is_empty() { // Should ideally be caught by error from get_words_for_game
        warn!("get_words_for_game returned an empty list unexpectedly, though it should return Err.");
        execute!(stdout, cursor::Show).ok(); // Attempt to cleanup
        terminal::disable_raw_mode().ok();  // Attempt to cleanup
        return Err(anyhow!("No words were selected for the game, words_for_game list is empty."));
    }
    
    let mut game_state = GameState::new(config.clone(), all_words, all_quotes, words_for_game);
    let (mut term_cols, mut term_rows) = terminal::size().context("Failed to get terminal size")?;

    let initial_prompt = "Press any key to start...";
    let prompt_padding = (term_cols.saturating_sub(initial_prompt.len() as u16)) / 2;
    let prompt_row = term_rows / 2;
    execute!(stdout, cursor::MoveTo(prompt_padding, prompt_row), Print(initial_prompt))
        .context("Failed to display initial prompt")?;
    stdout.flush().context("Failed to flush stdout for initial prompt")?;
    
    loop { 
        if event::poll(Duration::from_millis(500)).context("Event polling failed")? { 
            match event::read().context("Failed to read event")? {
                Event::Key(_key_event) => { // Any key press
                    game_state.start_time = Some(Instant::now());
                    break; 
                }
                Event::Resize(new_cols, new_rows) => { // Handle resize during initial prompt
                    term_cols = new_cols;
                    term_rows = new_rows;
                    // Re-display prompt
                    execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::MoveTo((term_cols.saturating_sub(initial_prompt.len() as u16)) / 2, term_rows / 2), Print(initial_prompt))
                        .context("Failed to re-display initial prompt on resize")?;
                    stdout.flush().context("Failed to flush stdout for prompt resize")?;
                }
                _ => {} // Ignore other events like mouse during prompt
            }
        }
    }

    'game_loop: loop {
        let elapsed_seconds = game_state.start_time.map_or(0.0, |st| st.elapsed().as_secs_f64());

        if !game_state.game_over {
            let mut game_should_end = false;
            match game_state.config.game_type {
                GameType::Time => {
                    if elapsed_seconds >= game_state.config.time_seconds.unwrap_or(0) as f64 { game_should_end = true; }
                }
                GameType::Words => {
                    if game_state.current_word_index >= game_state.config.word_count.unwrap_or(0) as usize 
                       && !game_state.words_to_type.is_empty() { game_should_end = true; }
                }
                GameType::Quote => {
                    if game_state.current_word_index >= game_state.words_to_type.len() 
                       && !game_state.words_to_type.is_empty() { game_should_end = true; }
                }
            }
            if game_should_end {
                debug!("Game over condition met. Type: {:?}, Elapsed: {:.2}s, Word Index: {}/{}", 
                    game_state.config.game_type, elapsed_seconds, game_state.current_word_index, game_state.words_to_type.len());
                game_state.game_over = true;
                game_state.final_elapsed_time_seconds = Some(elapsed_seconds);
            }
        }

        if game_state.game_over {
            display_game_over_screen(&mut stdout, &game_state, term_cols, term_rows)
                .context("Failed to display game over screen")?;
            if event::poll(Duration::from_millis(100)).context("Event polling failed on game over screen")? {
                 match event::read().context("Failed to read event on game over screen")? {
                    Event::Key(_) => break 'game_loop,
                    Event::Resize(new_cols, new_rows) => {
                        term_cols = new_cols; term_rows = new_rows;
                    }
                    _ => {} 
                 }
            }
        } else {
            if event::poll(Duration::from_millis(100)).context("Event polling failed in active game")? { 
                match event::read().context("Failed to read event in active game")? {
                    Event::Key(key_event) => {
                        if key_event.kind == event::KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Esc => { 
                                    debug!("Escape key pressed. Ending game.");
                                    game_state.game_over = true; 
                                    game_state.final_elapsed_time_seconds = Some(elapsed_seconds); 
                                },
                                KeyCode::Backspace => {
                                    trace!("Backspace pressed. Errors: '{}', Input: '{}'", game_state.errors, game_state.user_input);
                                    if !game_state.errors.is_empty() { game_state.errors.pop(); } 
                                    else if !game_state.user_input.is_empty() {
                                        game_state.user_input.pop();
                                        game_state.current_char_index = game_state.current_char_index.saturating_sub(1);
                                    }
                                }
                                KeyCode::Char(c) => {
                                    trace!("Char '{}' pressed.", c);
                                    game_state.typed_chars_total += 1; 
                                    if game_state.current_word_index >= game_state.words_to_type.len() { 
                                        warn!("Character typed after all words completed. Current index: {}, Total words: {}", 
                                            game_state.current_word_index, game_state.words_to_type.len());
                                        continue; 
                                    }
                                    // Ensure target_word is valid before indexing
                                    let target_word = &game_state.words_to_type[game_state.current_word_index];
                                    if game_state.current_char_index < target_word.len() {
                                        if c == target_word.chars().nth(game_state.current_char_index).unwrap_or_default() && game_state.errors.is_empty() {
                                            game_state.user_input.push(c);
                                            game_state.current_char_index += 1;
                                            game_state.correct_chars_total += 1;
                                        } else { game_state.errors.push(c); }
                                    } else { 
                                        if c == ' ' && game_state.errors.is_empty() {
                                            game_state.current_word_index += 1;
                                            game_state.current_char_index = 0;
                                            game_state.user_input.clear();
                                            game_state.correct_chars_total += 1; 
                                        } else { game_state.errors.push(c); }
                                    }
                                }
                                _ => {} 
                            }
                        }
                    }
                    Event::Resize(new_cols, new_rows) => { 
                        term_cols = new_cols; term_rows = new_rows;
                    }
                    _ => {} 
                }
            }
            display_game_interface(&mut stdout, &game_state, term_cols, term_rows)
                .context("Failed to display game interface")?;
        }
        
        let (current_cols, current_rows) = terminal::size().context("Failed to get terminal size during loop")?;
        if current_cols != term_cols || current_rows != term_rows {
             term_cols = current_cols;
             term_rows = current_rows;
             // Screen will be redrawn at the start of the next iteration or by specific display calls.
        }
    } 

    execute!(stdout, cursor::Show).context("Failed to show cursor")?;
    terminal::disable_raw_mode().context("Failed to disable raw mode")?;
    Ok(())
}
