use anyhow::{Context, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Parser)]
#[command(name = "mayan-cli")]
#[command(about = "A CLI utility for Mayan Finance operations")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Get auction state address from order ID
    #[command(alias = "gasa")]
    GetAuctionStateAddress {
        /// The order ID to query
        order_id: String,
    },
    /// Get and parse auction state data from order ID or auction state address
    #[command(alias = "gas")]
    GetAuctionState {
        /// The order ID or auction state address to query
        input: String,
        /// Solana RPC endpoint (optional, defaults to mainnet)
        #[arg(long, default_value = "https://api.mainnet-beta.solana.com")]
        rpc_url: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
struct MayanOrderResponse {
    #[serde(rename = "auctionStateAddr")]
    auction_state_addr: String,
    id: String,
    status: String,
    // Add other fields as needed - keeping minimal for now
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct AuctionState {
    pub bump: u8,
    pub hash: [u8; 32],
    pub initializer: Pubkey,
    pub close_epoch: u64,
    pub amount_out_min: u64,
    pub winner: Pubkey,
    pub amount_promised: u64,
    pub valid_from: u64,
    pub seq_msg: u64,
}

async fn get_auction_state_addr(order_id: &str) -> Result<String> {
    let url = format!(
        "https://explorer-api.mayan.finance/v3/swap/order-id/{}",
        order_id
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to send request to Mayan API")?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "API request failed with status: {}",
            response.status()
        ));
    }

    let order_data: MayanOrderResponse = response
        .json()
        .await
        .context("Failed to parse JSON response")?;

    Ok(order_data.auction_state_addr)
}

async fn get_and_parse_auction_state(input: &str, rpc_url: &str) -> Result<AuctionState> {
    // Determine if input is an order ID or auction state address
    // Solana addresses are base58 encoded and typically 32-44 characters
    // Try to parse as Pubkey first to see if it's a valid address
    let auction_state_addr = match Pubkey::from_str(input) {
        Ok(_) => {
            // Input is already a valid Pubkey (auction state address)
            input.to_string()
        }
        Err(_) => {
            // Input is likely an order ID, fetch auction state address from API
            get_auction_state_addr(input).await?
        }
    };

    // Connect to Solana RPC
    let client = RpcClient::new(rpc_url.to_string());

    // Parse the auction state address as a Pubkey
    let pubkey = Pubkey::from_str(&auction_state_addr)
        .context("Failed to parse auction state address as Pubkey")?;

    // Fetch the account data
    let account_data = client
        .get_account_data(&pubkey)
        .context("Failed to fetch account data from Solana")?;

    // Try to deserialize the account data using Borsh
    // Note: Some accounts may have a discriminator prefix, let's try with and without
    let auction_state = if account_data.len() >= 8 {
        // Try skipping potential 8-byte discriminator
        match AuctionState::try_from_slice(&account_data[8..]) {
            Ok(state) => state,
            Err(_) => {
                // Fall back to deserializing from the beginning
                AuctionState::try_from_slice(&account_data)
                    .context("Failed to deserialize auction state data (tried both with and without discriminator)")?
            }
        }
    } else {
        AuctionState::try_from_slice(&account_data)
            .context("Failed to deserialize auction state data")?
    };

    Ok(auction_state)
}

fn format_auction_state(auction_state: &AuctionState) -> String {
    format!(
        "Auction State Details:
  {}: {}
  {}: {}
  {}: {}
  {}: {}
  {}: {}
  {}: {}
  {}: {}
  {}: {}
  {}: {}",
        "Bump".green(),
        auction_state.bump,
        "Hash".green(),
        hex::encode(auction_state.hash),
        "Initializer".green(),
        auction_state.initializer,
        "Close Epoch".green(),
        auction_state.close_epoch,
        "Amount Out Min".green(),
        auction_state.amount_out_min,
        "Winner".green(),
        auction_state.winner,
        "Amount Promised".green(),
        auction_state.amount_promised,
        "Valid From".green(),
        auction_state.valid_from,
        "Sequence Message".green(),
        auction_state.seq_msg
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GetAuctionStateAddress { order_id } => {
            match get_auction_state_addr(&order_id).await {
                Ok(auction_state_addr) => {
                    println!("{}: {}", "Auction State Address".green(), auction_state_addr);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::GetAuctionState { input, rpc_url } => {
            match get_and_parse_auction_state(&input, &rpc_url).await {
                Ok(auction_state) => {
                    println!("{}", format_auction_state(&auction_state));
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
