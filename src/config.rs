use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub wallet_path: PathBuf,
    pub network: NetworkConfig,
    pub zcash: ZcashConfig,
    pub namada: NamadaConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub zcash_rpc_url: String,
    pub namada_rpc_url: String,
    pub zcash_network: String, // "mainnet" or "testnet"
    pub namada_chain_id: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZcashConfig {
    pub data_dir: PathBuf,
    pub rpc_user: Option<String>,
    pub rpc_password: Option<String>,
    pub rpc_port: u16,
    pub confirmations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamadaConfig {
    pub rpc_url: String,
    pub chain_id: String,
    pub gas_price: u64,
    pub gas_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<PathBuf>,
}

impl Config {
    pub fn default() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        let wallet_path = config_dir.join("wallet.db");
        
        Ok(Config {
            wallet_path,
            network: NetworkConfig {
                zcash_rpc_url: "http://localhost:8232".to_string(),
                namada_rpc_url: "http://localhost:26657".to_string(),
                zcash_network: "testnet".to_string(),
                namada_chain_id: "shielded-airdrop-test".to_string(),
                timeout_seconds: 30,
            },
            zcash: ZcashConfig {
                data_dir: config_dir.join("zcash"),
                rpc_user: None,
                rpc_password: None,
                rpc_port: 8232,
                confirmations: 10,
            },
            namada: NamadaConfig {
                rpc_url: "http://localhost:26657".to_string(),
                chain_id: "shielded-airdrop-test".to_string(),
                gas_price: 1000,
                gas_limit: 1000000,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: Some(config_dir.join("wallet.log")),
            },
        })
    }
    
    pub fn from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;
        
        let config: Config = toml::from_str(&content)
            .with_context(|| "Failed to parse config file")?;
        
        Ok(config)
    }
    
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .with_context(|| "Failed to serialize config")?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file: {:?}", path))?;
        
        Ok(())
    }
    
    pub fn get_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
            .join("zec-nam");
        
        fs::create_dir_all(&config_dir)
            .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
        
        Ok(config_dir)
    }
    
    pub fn get_data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?
            .join("zec-nam");
        
        fs::create_dir_all(&data_dir)
            .with_context(|| format!("Failed to create data directory: {:?}", data_dir))?;
        
        Ok(data_dir)
    }
    
    pub fn is_mainnet(&self) -> bool {
        self.network.zcash_network == "mainnet"
    }
    
    pub fn is_testnet(&self) -> bool {
        self.network.zcash_network == "testnet"
    }
} 