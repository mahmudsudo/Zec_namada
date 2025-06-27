use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, error, warn};
use anyhow::{Result, Context};

mod wallet;
mod config;


use wallet::AirdropWallet;
use config::Config;

#[derive(Parser)]
#[command(name = "zec-nam")]
#[command(about = "Shielded Airdrop Protocol Wallet for Zcash-Namada Integration")]
#[command(version)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new wallet
    Init {
        #[arg(short, long)]
        name: Option<String>,
        
        #[arg(short, long)]
        network: Option<String>,
    },
    
    /// Show wallet status and balance
    Status,
    
    /// Import notes from Zcash wallet
    Import {
        #[arg(short, long)]
        file: PathBuf,
        
        #[arg(short, long)]
        format: Option<String>,
    },
    
    /// List all notes in the wallet
    ListNotes {
        #[arg(short, long)]
        min_value: Option<u64>,
        
        #[arg(short, long)]
        note_type: Option<String>,
    },
    
    /// Create an airdrop transaction
    CreateAirdrop {
        #[arg(short, long)]
        note_index: usize,
        
        #[arg(short, long)]
        amount: u64,
        
        #[arg(short, long)]
        recipient: String,
        
        #[arg(short, long)]
        note_type: Option<String>,
    },
    
    /// Submit an airdrop transaction to the network
    SubmitAirdrop {
        #[arg(short, long)]
        tx_file: PathBuf,
    },
    
    /// Verify an airdrop transaction
    VerifyAirdrop {
        #[arg(short, long)]
        tx_file: PathBuf,
    },
    
    /// Show transaction details
    ShowTx {
        #[arg(short, long)]
        tx_file: PathBuf,
    },
    
    /// Connect to Zcash network and sync
    Sync,
    
    /// Show network status
    NetworkStatus,
    
    /// Export wallet data
    Export {
        #[arg(short, long)]
        file: PathBuf,
        
        #[arg(short, long)]
        format: Option<String>,
    },
    
    /// Generate test data for development
    GenerateTestData {
        #[arg(short, long)]
        count: Option<usize>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    // Load configuration
    let config = match cli.config {
        Some(path) => Config::from_file(&path)?,
        None => Config::default()?,
    };
    
    info!("Starting ZEC-NAM wallet with config: {:?}", config.wallet_path);
    
    match cli.command {
        Commands::Init { name, network } => {
            commands::init_wallet(&config, name, network).await?;
        }
        
        Commands::Status => {
            commands::show_status(&config).await?;
        }
        
        Commands::Import { file, format } => {
            commands::import_notes(&config, &file, format).await?;
        }
        
        Commands::ListNotes { min_value, note_type } => {
            commands::list_notes(&config, min_value, note_type).await?;
        }
        
        Commands::CreateAirdrop { note_index, amount, recipient, note_type } => {
            commands::create_airdrop(&config, note_index, amount, &recipient, note_type).await?;
        }
        
        Commands::SubmitAirdrop { tx_file } => {
            commands::submit_airdrop(&config, &tx_file).await?;
        }
        
        Commands::VerifyAirdrop { tx_file } => {
            commands::verify_airdrop(&config, &tx_file).await?;
        }
        
        Commands::ShowTx { tx_file } => {
            commands::show_transaction(&config, &tx_file).await?;
        }
        
        Commands::Sync => {
            commands::sync_wallet(&config).await?;
        }
        
        Commands::NetworkStatus => {
            commands::network_status(&config).await?;
        }
        
        Commands::Export { file, format } => {
            commands::export_wallet(&config, &file, format).await?;
        }
        
        Commands::GenerateTestData { count } => {
            commands::generate_test_data(&config, count.unwrap_or(10)).await?;
        }
    }
    
    Ok(())
}

