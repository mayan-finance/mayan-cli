use anyhow::{Context, Result};
use borsh::{BorshDeserialize, BorshSerialize};
use clap::{Parser, Subcommand};
use colored::*;
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_client::rpc_config::RpcTransactionConfig;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::{
    EncodedTransaction, UiInstruction, UiMessage, UiParsedInstruction, UiTransactionEncoding,
};
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
    /// Get auction state address from order ID [alias: gasa]
    #[command(alias = "gasa")]
    GetAuctionStateAddress {
        /// The order ID to query
        order_id: String,
    },
    /// Get and parse auction state data from order ID or auction state address [alias: gas]
    #[command(alias = "gas")]
    GetAuctionState {
        /// The order ID or auction state address to query
        input: String,
        /// Solana RPC endpoint (optional, defaults to mainnet) or env var SOLANA_RPC_URL
        #[arg(long, default_value = "https://api.mainnet-beta.solana.com", env = "SOLANA_RPC_URL")]
        rpc_url: String,
    },
    /// Get bid information from auction state address or order ID [alias: gb]
    #[command(alias = "gb")]
    GetBids {
        /// The order ID or auction state address to query
        input: String,
        /// Solana RPC endpoint (optional, defaults to mainnet) or env var SOLANA_RPC_URL
        #[arg(long, default_value = "https://api.mainnet-beta.solana.com", env = "SOLANA_RPC_URL")]
        rpc_url: String,
    },
    /// Decode a base58 encoded string [alias: b58d]
    #[command(alias = "b58d")]
    Base58Decode {
        /// The base58 encoded string to decode
        input: String,
        /// Output format: hex, bytes, or utf8
        #[arg(long, default_value = "hex")]
        format: String,
    },
    /// Encode data to base58 [alias: b58e]
    #[command(alias = "b58e")]
    Base58Encode {
        /// The input data to encode
        input: String,
        /// Input format: hex, bytes, or utf8
        #[arg(long, default_value = "hex")]
        format: String,
    },
    /// Convert hex string or bytes array to exactly 32 bytes (panics if not 32 bytes) [alias: b32d]
    #[command(alias = "b32d")]
    ToBytes32 {
        /// The input hex string (with or without 0x prefix) or comma-separated bytes
        input: String,
        /// Input format: hex or bytes
        #[arg(long, default_value = "hex")]
        format: String,
    },
    /// Convert data to 32-byte array (pads if shorter, panics if longer than 32 bytes) [alias: b32e]
    #[command(alias = "b32e")]
    FromBytes32 {
        /// The input data as hex string (with or without 0x prefix) or comma-separated bytes
        input: String,
        /// Input format: hex or bytes
        #[arg(long, default_value = "hex")]
        input_format: String,
        /// Output format: hex or bytes
        #[arg(long, default_value = "hex")]
        output_format: String,
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

#[derive(Debug, Clone)]
pub struct BidEntry {
    pub signature: String,
    pub bidder: String,
    pub bid_amount: u64,
    pub slot: u64,
    pub timestamp: Option<i64>,
    pub failed: bool,
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

async fn get_bid_history(auction_state_addr: &str, rpc_url: &str) -> Result<Vec<BidEntry>> {
    let client = RpcClient::new(rpc_url.to_string());
    let pubkey = Pubkey::from_str(auction_state_addr)
        .context("Failed to parse auction state address as Pubkey")?;

    let signatures = client
        .get_signatures_for_address(&pubkey)
        .context("Failed to get signatures for auction state address")?;

    let mut bids = Vec::new();

    // Limit to 100 transactions for performance
    for sig_info in signatures.iter().take(100) {
        let signature = Signature::from_str(&sig_info.signature)?;
        let transaction = client.get_transaction_with_config(
            &signature,
            RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::JsonParsed),
                max_supported_transaction_version: Some(0),
                commitment: Some(CommitmentConfig::confirmed()),
            },
        )?;

        let meta = transaction
            .transaction
            .meta
            .as_ref()
            .ok_or(anyhow::anyhow!("Failed to get transaction meta"))?;

        let valid = meta
            .log_messages
            .as_ref()
            .map(|logs| {
                logs.iter()
                    .any(|log| log.contains("Program log: Instruction: Bid"))
            })
            .unwrap_or(false);
        if !valid {
            continue;
        }

        let failed = meta.err.is_some();

        let ui_transaction = match &transaction.transaction.transaction {
            EncodedTransaction::Json(parsed_tx) => parsed_tx,
            _ => continue, // skip unsupported encodings
        };

        // Only handle parsed messages
        let message = match &ui_transaction.message {
            UiMessage::Parsed(parsed_msg) => parsed_msg,
            _ => continue,
        };

        let instruction = message.instructions[2].clone();
        let parsed = match instruction {
            UiInstruction::Parsed(UiParsedInstruction::PartiallyDecoded(parsed)) => parsed,
            _ => {
                println!("Not a parsed instruction");
                continue;
            }
        };

        let bidder = message.account_keys[0].pubkey.clone();
        let data = bs58::decode(parsed.data).into_vec().unwrap();
        let bid_amount = u64::from_le_bytes(data[data.len() - 8..].try_into().unwrap());

        bids.push(BidEntry {
            signature: sig_info.signature.clone(),
            bidder,
            bid_amount,
            slot: sig_info.slot,
            timestamp: sig_info.block_time,
            failed,
        });
    }

    // Sort bids by slot (chronological order)
    bids.sort_by(|a, b| a.slot.cmp(&b.slot));

    Ok(bids)
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

fn format_bid_history(bids: &[BidEntry]) -> String {
    if bids.is_empty() {
        return format!("{}: No bids found", "Bid History".yellow());
    }

    let mut result = format!("{}: {} bids found\n", "Bid History".green(), bids.len());

    for (i, bid) in bids.iter().enumerate() {
        let (diff_str, status_str) = if i > 0 && bid.bid_amount > 0 && bids[i - 1].bid_amount > 0 {
            let diff = bid.bid_amount as i128 - bids[i - 1].bid_amount as i128;
            let status = format!("  {}: {}", "Status".green(), if bid.failed { "Failed".red() } else { "Success".green() });
            if diff >= 0 {
                (format!("+{}", diff).blue().to_string(), status)
            } else {
                (format!("{}", diff).red().to_string(), status)
            }
        } else {
            ("-".to_string(), "".to_string())
        };

        result.push_str(&format!(
            "\n{} {}:
  {}: {}
  {}: {}
  {}: {}
  {}: {}
  {}: {}
  {}: {}{}",
            "Bid".cyan(),
            i + 1,
            "Signature".green(),
            bid.signature,
            "Bidder".green(),
            bid.bidder,
            "Amount".green(),
            if bid.bid_amount > 0 {
                bid.bid_amount.to_string().yellow().to_string()
            } else {
                "Unknown".to_string()
            },
            "Diff".green(),
            diff_str,
            "Slot".green(),
            bid.slot,
            "Timestamp".green(),
            bid.timestamp.unwrap_or(0),
            if !status_str.is_empty() {
                format!("\n{}", status_str)
            } else {
                "".to_string()
            }
        ));
    }

    result
}

fn decode_base58(input: &str, format: &str) -> Result<()> {
    let decoded = bs58::decode(input)
        .into_vec()
        .context("Failed to decode base58 string")?;

    match format.to_lowercase().as_str() {
        "hex" => {
            println!("{}: {}", "Hex".green(), hex::encode(&decoded));
        }
        "bytes" => {
            println!("{}: {:?}", "Bytes".green(), decoded);
        }
        "utf8" => match String::from_utf8(decoded.clone()) {
            Ok(utf8_string) => {
                println!("{}: {}", "UTF-8".green(), utf8_string);
            }
            Err(_) => {
                println!("{}: Invalid UTF-8 sequence", "Error".red());
                println!("{}: {}", "Raw bytes".yellow(), hex::encode(&decoded));
            }
        },
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid format '{}'. Valid formats are: hex, bytes, utf8",
                format
            ));
        }
    }

    Ok(())
}

fn to_bytes32(input: &str, format: &str) -> Result<[u8; 32]> {
    let bytes = match format.to_lowercase().as_str() {
        "hex" => {
            // Remove 0x prefix if present
            let hex_str = input.strip_prefix("0x").unwrap_or(input);
            hex::decode(hex_str).context("Failed to decode hex string")?
        }
        "bytes" => {
            // Parse comma-separated bytes like "1,2,3,4,..."
            input
                .split(',')
                .map(|s| s.trim().parse::<u8>().context("Failed to parse byte value"))
                .collect::<Result<Vec<u8>>>()?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid format '{}'. Valid formats are: hex, bytes",
                format
            ));
        }
    };

    if bytes.len() != 32 {
        panic!(
            "Input must be exactly 32 bytes, got {} bytes. Input: {}",
            bytes.len(),
            input
        );
    }

    let mut result = [0u8; 32];
    result.copy_from_slice(&bytes);
    Ok(result)
}

fn encode_base58(input: &str, format: &str) -> Result<()> {
    let bytes = match format.to_lowercase().as_str() {
        "hex" => {
            // Remove 0x prefix if present
            let hex_str = input.strip_prefix("0x").unwrap_or(input);
            hex::decode(hex_str).context("Failed to decode hex string")?
        }
        "bytes" => {
            // Parse comma-separated bytes like "1,2,3,4,..."
            input
                .split(',')
                .map(|s| s.trim().parse::<u8>().context("Failed to parse byte value"))
                .collect::<Result<Vec<u8>>>()?
        }
        "utf8" => input.as_bytes().to_vec(),
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid format '{}'. Valid formats are: hex, bytes, utf8",
                format
            ));
        }
    };

    let encoded = bs58::encode(&bytes).into_string();
    println!("{}: {}", "Base58".green(), encoded);

    Ok(())
}

fn from_bytes32(input: &str, input_format: &str, output_format: &str) -> Result<()> {
    // First, get the input bytes
    let bytes = match input_format.to_lowercase().as_str() {
        "hex" => {
            // Remove 0x prefix if present
            let hex_str = input.strip_prefix("0x").unwrap_or(input);
            hex::decode(hex_str).context("Failed to decode hex string")?
        }
        "bytes" => {
            // Parse comma-separated bytes like "1,2,3,4,..."
            input
                .split(',')
                .map(|s| s.trim().parse::<u8>().context("Failed to parse byte value"))
                .collect::<Result<Vec<u8>>>()?
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid input format '{}'. Valid formats are: hex, bytes",
                input_format
            ));
        }
    };

    // Check if input is longer than 32 bytes
    if bytes.len() > 32 {
        panic!(
            "Input is too long: {} bytes. Maximum is 32 bytes. Input: {}",
            bytes.len(),
            input
        );
    }

    // Pad to 32 bytes (left-pad with zeros for addresses, which is standard in Solidity)
    let mut bytes32 = [0u8; 32];
    let start_index = 32 - bytes.len();
    bytes32[start_index..].copy_from_slice(&bytes);

    // Output in the requested format
    match output_format.to_lowercase().as_str() {
        "hex" => {
            println!("{}: 0x{}", "Hex".green(), hex::encode(bytes32));
        }
        "bytes" => {
            println!(
                "{}: [{}]",
                "Bytes".green(),
                bytes32
                    .iter()
                    .map(|b| b.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid output format '{}'. Valid formats are: hex, bytes",
                output_format
            ));
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::GetAuctionStateAddress { order_id } => {
            match get_auction_state_addr(&order_id).await {
                Ok(auction_state_addr) => {
                    println!(
                        "{}: {}",
                        "Auction State Address".green(),
                        auction_state_addr
                    );
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
        Commands::GetBids { input, rpc_url } => {
            // Determine if input is an order ID or auction state address
            let auction_state_addr = match Pubkey::from_str(&input) {
                Ok(_) => {
                    // Input is already a valid Pubkey (auction state address)
                    input.clone()
                }
                Err(_) => {
                    // Input is likely an order ID, fetch auction state address from API
                    match get_auction_state_addr(&input).await {
                        Ok(addr) => addr,
                        Err(e) => {
                            eprintln!("Error getting auction state address: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
            };

            match get_bid_history(&auction_state_addr, &rpc_url).await {
                Ok(bids) => {
                    println!("{}", format_bid_history(&bids));
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Base58Decode { input, format } => {
            if let Err(e) = decode_base58(&input, &format) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::Base58Encode { input, format } => {
            if let Err(e) = encode_base58(&input, &format) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
        Commands::ToBytes32 { input, format } => match to_bytes32(&input, &format) {
            Ok(bytes32) => {
                println!(
                    "{}: [{}]",
                    "Bytes32 Array".green(),
                    bytes32
                        .iter()
                        .map(|b| b.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                println!("{}: {}", "Hex".green(), hex::encode(bytes32));
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::FromBytes32 {
            input,
            input_format,
            output_format,
        } => {
            if let Err(e) = from_bytes32(&input, &input_format, &output_format) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}
