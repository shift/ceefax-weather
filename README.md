# CEEFAX Weather Map

A Rust-based console application that fetches weather data from `wttr.in` and displays it as a retro CEEFAX/TELETEXT-style weather map.

Fully created via promting Gemini 2.5 Pro.

## Features

-   **Retro CEEFAX/TELETEXT Interface**: Faithfully recreates the classic teletext aesthetic using a custom RGB color palette and layout.
-   **Multiple Countries**: Supports weather maps for both the United Kingdom and Germany.
-   **Live Weather Data**: Fetches real-time temperature and weather summaries from the `wttr.in` JSON API.
-   **Loading Animation**: Displays a themed "page searching" animation while fetching data in the background.
-   **Self-Contained & Reproducible**: Packaged with a Nix flake to ensure it runs reliably with all its dependencies.

## Requirements

-   [Nix](https://nixos.org/) (with Flakes enabled)

## How to Run

1.  **Clone the repository or save the project files** into a single directory:
    -   `flake.nix`
    -   `Cargo.toml`
    -   `src/main.rs`

2.  **Fetch Rust dependencies**: Before the first run, you need to generate the `Cargo.lock` file. Navigate to the project directory in your terminal and run:
    ```bash
    nix develop -c cargo fetch
    ```

3.  **Run the application**:
    -   **For the UK map (default):**
        ```bash
        nix run .#
        ```
    -   **For the German map:**
        ```bash
        nix run .# -- --country germany
        ```

4.  **Exit**: Press any key or `Esc` to close the application.

## Project Structure

-   **`flake.nix`**: The Nix flake file that defines the development environment and packages the application. It ensures that the correct Rust toolchain and any system dependencies (like `openssl`) are available.
-   **`Cargo.toml`**: The Rust package manager's manifest file. It lists the project's Rust dependencies (crates) like `ratatui`, `reqwest`, and `clap`.
-   **`src/main.rs`**: The main application source code. It contains all the logic for fetching data, defining the layouts, and rendering the terminal user interface with `ratatui`.
