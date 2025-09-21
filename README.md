# Polygon Arbitrage Opportunity Detector

This Rust application detects potential arbitrage opportunities on the Polygon network. It periodically checks the prices of token pairs (e.g., WETH/USDC, WBTC/USDC) across multiple Decentralized Exchanges (DEXes) like QuickSwap, SushiSwap, and Uniswap V3.

If a profitable opportunity is found (exceeding a configurable threshold and accounting for simulated gas fees), it is logged to the console and a local SQLite database.

## System Architecture

The bot is built with a modular and asynchronous architecture:

-   **Main Loop (`main.rs`):** The entry point that orchestrates the application. It runs on a `tokio` timer to periodically trigger the arbitrage check.
-   **Configuration (`config.rs`):** Manages all settings via a `.env` file, including RPC endpoints, trade amounts, and profit thresholds.
-   **DEX Interaction (`dex.rs`):** Contains the logic for fetching prices from different DEX protocols (Uniswap V2 & V3). It uses `ethers-rs` and contract ABIs to communicate with the Polygon blockchain.
-   **Arbitrage Logic (`arbitrage.rs`):** The core engine that compares prices between all DEX permutations for configured token pairs. It calculates the simulated profit and decides if an opportunity is worth logging.
-   **Database (`db.rs`):** Uses `sqlx` and a local SQLite database to persist all found opportunities for later analysis. Migrations are handled automatically.

## Technology Stack

-   **Language:** Rust
-   **Blockchain:** Polygon Network
-   **Async Runtime:** Tokio
-   **Ethereum Interaction:** `ethers-rs`
-   **Database:** SQLite (via `sqlx`)
-   **Configuration:** `dotenvy`
-   **Logging:** `tracing`

## Setup and Installation

### Prerequisites

1.  **Rust:** Install the Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2.  **Polygon RPC URL:** Get an RPC URL from a provider like Alchemy, Infura, or Ankr.
3.  **`sqlx-cli`:** Install the SQLx command-line tool to manage database migrations.
    ```sh
    cargo install sqlx-cli --no-default-features --features rustls,sqlite
    ```

### Installation Steps

1.  **Clone the repository:**
    ```sh
    git clone <your-repo-url>
    cd polygon-arbitrage-bot
    ```

2.  **Create a `.env` file:**
    Copy the example file and fill in your details.
    ```sh
    cp .env.example .env
    ```
    Now, edit `.env` and add your Polygon RPC URL.

3.  **Prepare the Database:**
    The application uses a local SQLite database file. `sqlx-cli` will create it based on your `.env` file and run the necessary migrations.
    ```sh
    sqlx database create
    sqlx migrate run
    ```

4.  **Build the project:**
    ```sh
    cargo build --release
    ```

## How to Run the Bot

Once built, you can run the bot with the following command:

```sh
cargo run --release
