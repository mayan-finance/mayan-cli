# Mayan Utils CLI

A command-line utility for Mayan Finance operations.

## Features

- **Get Auction State Address**: Retrieve the auction state address from a Mayan order ID
- **Parse Auction State**: Fetch and parse the complete auction state data from Solana blockchain

## Installation

### Prerequisites

- Rust (install from [rustup.rs](https://rustup.rs/))

### Build from Source

```bash
git clone <your-repo-url>
cd mayan-utils
cargo build --release
```

The compiled binary will be available at `target/release/mayan-utils`.

## Usage

### Get Auction State Address

Retrieve the auction state address for a given order ID:

```bash
# Using cargo run (for development)
cargo run -- get-auction-state <ORDER_ID>

# Using the compiled binary
./target/release/mayan-utils get-auction-state <ORDER_ID>
```

#### Example

```bash
cargo run -- get-auction-state "SWIFT_0xcd96bb4c31aa86d29a39117206055d2b17b65156c66886050c10abd48ee6691a"
```

Output:
```
Auction State Address: 6p7fUeppNLatf5TkmMA4ybpSJBejMnGSRknhPfEBNSF3
```

### Parse Auction State

Fetch the auction state address and then retrieve and parse the complete auction state data from the Solana blockchain:

```bash
# Using cargo run (for development)
cargo run -- parse-auction-state <ORDER_ID>

# Using a custom RPC endpoint
cargo run -- parse-auction-state <ORDER_ID> --rpc-url <RPC_URL>

# Using the compiled binary
./target/release/mayan-utils parse-auction-state <ORDER_ID>
```

#### Example

```bash
cargo run -- parse-auction-state "SWIFT_0xcd96bb4c31aa86d29a39117206055d2b17b65156c66886050c10abd48ee6691a"
```

Output:
```
Fetching and parsing auction state for order ID: SWIFT_0xcd96bb4c31aa86d29a39117206055d2b17b65156c66886050c10abd48ee6691a
Using RPC URL: https://api.mainnet-beta.solana.com
Auction State Details:
  Bump: 255
  Hash: cd96bb4c31aa86d29a39117206055d2b17b65156c66886050c10abd48ee6691a
  Initializer: B88xH3Jmhq4WEaiRno2mYmsxV35MmgSY45ZmQnbL8yft
  Close Epoch: 797
  Amount Out Min: 641865924
  Winner: FzZ77TM8Ekcb6gyWPmcT9upWkAZKZc5xrYfuFu7pifPn
  Amount Promised: 644921303
  Valid From: 1748670506
  Sequence Message: 0
```

### Help

To see all available commands:

```bash
cargo run -- --help
```

To get help for a specific command:

```bash
cargo run -- get-auction-state --help
cargo run -- parse-auction-state --help
```

## Auction State Data Structure

The parsed auction state contains the following fields:

- **Bump**: Program derived address bump seed
- **Hash**: 32-byte hash identifier
- **Initializer**: Public key of the auction initializer
- **Close Epoch**: Epoch when the auction closes
- **Amount Out Min**: Minimum amount out (in token units)
- **Winner**: Public key of the auction winner
- **Amount Promised**: Amount promised by the winner
- **Valid From**: Unix timestamp when the auction becomes valid
- **Sequence Message**: Sequence number for the message

## API Reference

This tool uses the following APIs:
- **Mayan Explorer API**: `https://explorer-api.mayan.finance/v3/swap/order-id/<order-id>`
  - Method: GET
  - Response: JSON object containing order details including `auctionStateAddr`
- **Solana RPC API**: Configurable RPC endpoint (defaults to mainnet)
  - Used to fetch account data from the blockchain
  - Data is deserialized using Borsh format

## Dependencies

- `reqwest`: HTTP client for API requests
- `serde`: JSON serialization/deserialization
- `clap`: Command-line argument parsing
- `tokio`: Async runtime
- `anyhow`: Error handling
- `solana-client`: Solana RPC client
- `solana-sdk`: Solana SDK for public key handling
- `borsh`: Binary serialization format for Solana account data
- `hex`: Hexadecimal encoding for hash display

## Development

### Running Tests

```bash
cargo test
```

### Running with Debug Output

```bash
RUST_LOG=debug cargo run -- <command> <args>
```
