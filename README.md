# Hyperliquid Perpetuals Trading (Perp Trading) Bot

An experimental, multi-market trading system for the Hyperliquid exchange, implemented in Rust and powered by [`hyperliquid_rust_sdk`](https://github.com/Novus-Tech-LLC/hyperliquid-rust-sdk). It manages multiple markets concurrently and places orders based on a configurable indicator-driven strategy.

## Key Features

- Connect to Hyperliquid mainnet, testnet, or a local node
- Manage multiple markets with configurable margin allocation per market
- Pluggable strategy with adjustable risk, style, and market stance
- Indicator engine with per-timeframe bindings
- Fully asynchronous design built on `tokio` with `flume` channels

## Architecture Overview

- Core trading logic lives in Rust (`src/`) with dedicated modules for markets, signals, execution, and strategy
- Optional frontend lives under `src/frontend/` (React + TypeScript)

## Prerequisites

- Rust toolchain (stable), installed via `rustup`
- Hyperliquid API credentials

## Quick Start

1) Clone the repository.

2) Create a `.env` file in the project root:

   ```env
   PRIVATE_KEY=<your API private key>  # obtain at https://app.hyperliquid.xyz/API
   AGENT_KEY=<optional agent API public key>
   WALLET=<public wallet address>
   ```

3) Run the application.

   On Unix-like systems:
   ```bash
   ./run.sh
   ```

   On Windows (PowerShell):
   ```powershell
   cargo run --release
   ```

## Configuration

- See `config.toml` for example strategy and runtime configuration
- Supported trading pairs are defined in `src/assets.rs` (`MARKETS`)

## Strategy

The default strategy is implemented in `src/strategy.rs` as `CustomStrategy`. It combines multiple indicators (e.g., RSI, StochRSI, EMA crosses, ADX, ATR). You can tune:

- Risk level: `Low`, `Normal`, `High`
- Trading style: `Scalp`, `Swing`
- Market stance: `Bull`, `Bear`, `Neutral`

Signals are emitted when multiple indicator conditions align (e.g., oversold RSI with a bullish StochRSI crossover) and then translated into orders by the executor.

## Indicators

Indicators are configured as `(IndicatorKind, TimeFrame)` pairs. Available kinds include:

- `Rsi(u32)`
- `SmaOnRsi { periods, smoothing_length }`
- `StochRsi { periods, k_smoothing, d_smoothing }`
- `Adx { periods, di_length }`
- `Atr(u32)`
- `Ema(u32)`
- `EmaCross { short, long }`
- `Sma(u32)`

Each pair is wrapped in an `Entry` with an `EditType` (`Add`, `Remove`, `Toggle`). See the usage in the codebase for constructing market configurations with custom indicator sets.

## Project Structure

- `src/bot.rs` – Orchestrates markets and keeps margin in sync
- `src/market.rs` – Single-market pipeline: data feed, signal engine, order execution
- `src/signal/` – Indicator trackers and strategy logic
- `src/executor.rs` – Order placement via the Hyperliquid API
- `src/strategy.rs` – Strategies consumed by the signal engine
- `src/trade_setup.rs` – Trading parameters and metadata
- `src/assets.rs` – Supported markets (`MARKETS`)
- `src/frontend/` – Optional React/TypeScript UI
- `config.toml` – Example runtime/strategy configuration

## Development

- Build: `cargo build --release`
- Run: `cargo run --release`

## Contact

| Platform | Link |
|----------|------|
| 📱 Telegram | [t.me/novustch](https://t.me/novustch) |
| 📲 WhatsApp | [wa.me/14105015750](https://wa.me/14105015750) |
| 💬 Discord | [discordapp.com/users/985432160498491473](https://discordapp.com/users/985432160498491473)

<div align="center">
    <a href="https://t.me/novustch" target="_blank"><img alt="Telegram"
        src="https://img.shields.io/badge/Telegram-26A5E4?style=for-the-badge&logo=telegram&logoColor=white"/></a>
    <a href="https://wa.me/14105015750" target="_blank"><img alt="WhatsApp"
        src="https://img.shields.io/badge/WhatsApp-25D366?style=for-the-badge&logo=whatsapp&logoColor=white"/></a>
    <a href="https://discordapp.com/users/985432160498491473" target="_blank"><img alt="Discord"
        src="https://img.shields.io/badge/Discord-7289DA?style=for-the-badge&logo=discord&logoColor=white"/></a>
</div>

Feel free to reach out for implementation assistance or integration support.

## Disclaimer

This software is experimental and unaudited. Trading involves risk. Use at your own discretion and responsibility when interacting with live markets.
