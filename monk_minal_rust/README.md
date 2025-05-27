# MonkMinal Rust

MonkMinal Rust is a terminal-based typing game written in Rust, inspired by Monkeytype. It aims to help users improve their typing speed and accuracy through various game modes and challenges, all within the comfort of their terminal.

## Features

*   **Multiple Game Modes**:
    *   **Time Mode**: Type as many words as you can within a fixed time limit (e.g., 15s, 30s, 60s, 120s).
    *   **Words Mode**: Type a specific number of words (e.g., 10, 20, 30, 40, 50).
    *   **Quote Mode**: Type out a randomly selected quote.
*   **Difficulty Levels**:
    *   **Easy**: Filters for shorter words (typically <= 5 characters).
    *   **Medium**: Filters for medium-length words (typically <= 8 characters).
    *   **Hard**: Uses words of any length from the dictionary.
    *   (Note: Difficulty primarily affects Time and Words modes).
*   **Real-time Feedback**:
    *   Displays Words Per Minute (WPM) - both Gross and Net.
    *   Shows typing accuracy percentage.
    *   Live timer (countdown for Time mode, elapsed for others).
*   **Interactive Terminal UI**:
    *   Text to type is displayed and styled.
    *   User input is shown with immediate feedback (correct characters, errors).
    *   Responsive design that adapts to terminal size changes.
*   **Configuration**: Interactive prompts to select game mode, duration/word count, and difficulty at the start.
*   **Cross-platform**: Built with Rust, aiming for compatibility where Rust and terminals are supported.

## Building

To build MonkMinal Rust from source, you need to have Rust and Cargo installed.

1.  **Clone the repository**:
    ```bash
    git clone <repository_url> 
    # Replace <repository_url> with the actual URL of the repository
    ```
2.  **Navigate to the project directory**:
    ```bash
    cd monk_minal_rust
    ```
3.  **Build the release binary**:
    ```bash
    cargo build --release
    ```
    The compiled binary will be located at `target/release/monk_minal_rust`.

## Running

After building, you can run the game from the project's root directory:

```bash
./target/release/monk_minal_rust
```

## CLI Options

MonkMinal Rust supports the following standard command-line options:

*   `--help`: Displays a help message with information about available commands and options.
*   `--version`: Shows the current version of the application.

These are automatically provided by the `clap` argument parser.

---

Happy Typing!
