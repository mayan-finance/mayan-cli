# Mayan Utils CLI

A command-line utility for Mayan Finance operations that provides easy access to auction state data from order IDs or auction state addresses.

## Features

- **Get Auction State Address**: Retrieve the auction state address from a Mayan order ID
- **Get Auction State**: Fetch and parse complete auction state data from order ID or auction state address
- **Colored Output**: Enhanced readability with green-colored field names
- **Flexible Input**: Works with both order IDs and direct auction state addresses

## Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `get-auction-state-address` | `gasa` | Get auction state address from order ID |
| `get-auction-state` | `gas` | Get and parse auction state data from order ID or auction state address |

## Installation

### Option 1: Automated Install Script (Recommended)

We provide an automated install script that detects your system and builds the appropriate binary:

```bash
# Clone the repository
git clone <your-repo-url>
cd mayan-utils

# Run the install script
./install.sh
```

**Supported Systems:**
- **macOS**: Intel (x86_64) and Apple Silicon (M1/M2/M3)
- **Linux**: x86_64, ARM64, and ARM v7 architectures
- **Auto-detection**: Automatically detects your system and builds the correct target

The script will:
1. ✅ Check for Rust installation
2. 🔍 Detect your system architecture
3. 🏗️ Build the optimized binary for your platform
4. 📦 Install to `/usr/local/bin` (may require sudo)
5. ✅ Verify the installation

### Option 2: Manual Build

### Prerequisites

- Rust (install from [rustup.rs](https://rustup.rs/))

### Build from Source

```bash
git clone <your-repo-url>
cd mayan-utils
cargo build --release
```

The compiled binary will be available at `target/release/mayan-utils`.

### Manual Installation

```bash
# Copy to system bin directory (requires sudo on most systems)
sudo cp target/release/mayan-utils /usr/local/bin/

# Or copy to user bin directory
mkdir -p ~/.local/bin
cp target/release/mayan-utils ~/.local/bin/
export PATH="$HOME/.local/bin:$PATH"  # Add to your shell profile
```

## Usage

### Get Auction State Address

Retrieve the auction state address for a given order ID:

```bash
# Using full command name
cargo run -- get-auction-state-address <ORDER_ID>

# Using short alias
cargo run -- gasa <ORDER_ID>

# Using the compiled binary
./target/release/mayan-utils gasa <ORDER_ID>
```

#### Example

```bash
cargo run -- gasa "SWIFT_0xcd96bb4c31aa86d29a39117206055d2b17b65156c66886050c10abd48ee6691a"
```

Output:
```
Auction State Address: 6p7fUeppNLatf5TkmMA4ybpSJBejMnGSRknhPfEBNSF3
```

### Get Auction State

Fetch and parse the complete auction state data. This command accepts either:
- **Order ID**: Automatically fetches auction state address from Mayan API
- **Auction State Address**: Directly queries Solana blockchain

```bash
# Using full command name with order ID
cargo run -- get-auction-state <ORDER_ID>

# Using short alias with order ID
cargo run -- gas <ORDER_ID>

# Using auction state address directly
cargo run -- gas <AUCTION_STATE_ADDRESS>

# Using a custom RPC endpoint
cargo run -- gas <ORDER_ID_OR_ADDRESS> --rpc-url <RPC_URL>

# Using the compiled binary
./target/release/mayan-utils gas <ORDER_ID_OR_ADDRESS>
```

#### Example with Order ID

```bash
cargo run -- gas "SWIFT_0xcd96bb4c31aa86d29a39117206055d2b17b65156c66886050c10abd48ee6691a"
```

#### Example with Auction State Address

```bash
cargo run -- gas "6p7fUeppNLatf5TkmMA4ybpSJBejMnGSRknhPfEBNSF3"
```

Output:
```
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
cargo run -- get-auction-state-address --help
cargo run -- get-auction-state --help
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
- `clap`: Command-line argument parsing with aliases
- `tokio`: Async runtime
- `anyhow`: Error handling
- `solana-client`: Solana RPC client
- `solana-sdk`: Solana SDK for public key handling
- `borsh`: Binary serialization format for Solana account data
- `hex`: Hexadecimal encoding for hash display
- `colored`: Terminal color output for better readability

## Development

### Running Tests

```bash
cargo test
```

### Running with Debug Output

```bash
RUST_LOG=debug cargo run -- <command> <args>
```
