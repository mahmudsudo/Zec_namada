use std::path::Path;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use sled;
use tracing::{info, warn, error};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    AirdropWallet as CoreWallet, SaplingNote, OrchardNote, 
    ShieldedAirdropTransaction, NullifierSet, ProtocolError, PublicKey, ClaimDescription
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetadata {
    pub name: String,
    pub created_at: u64,
    pub last_sync: u64,
    pub network: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteMetadata {
    pub note_type: String, // "sapling" or "orchard"
    pub value: u64,
    pub position: u64,
    pub is_spent: bool,
    pub created_at: u64,
    pub last_used: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub tx_hash: String,
    pub airdrop_nullifier: Vec<u8>,
    pub amount: u64,
    pub recipient: String,
    pub status: String, // "pending", "confirmed", "failed"
    pub created_at: u64,
    pub confirmed_at: Option<u64>,
    pub block_height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaplingNoteRecord {
    pub note: SaplingNote,
    pub created_at: u64,
    pub is_spent: bool,
    pub last_used: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardNoteRecord {
    pub note: OrchardNote,
    pub created_at: u64,
    pub is_spent: bool,
    pub last_used: Option<u64>,
}

pub struct AirdropWallet {
    db: sled::Db,
    core_wallet: CoreWallet,
    metadata: WalletMetadata,
}

impl AirdropWallet {
    pub fn new(path: &Path, name: &str, network: &str) -> Result<Self> {
        let db = sled::open(path)
            .with_context(|| format!("Failed to open wallet database: {:?}", path))?;
        
        let core_wallet = CoreWallet::new();
        
        let metadata = WalletMetadata {
            name: name.to_string(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_sync: 0,
            network: network.to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        let mut wallet = Self {
            db,
            core_wallet,
            metadata,
        };
        
        // Initialize database
        wallet.init_database()?;
        
        Ok(wallet)
    }
    
    pub fn load(path: &Path) -> Result<Self> {
        let db = sled::open(path)
            .with_context(|| format!("Failed to open wallet database: {:?}", path))?;
        
        let core_wallet = CoreWallet::new();
        
        // Load metadata
        let metadata_bytes = db.get("metadata")?
            .ok_or_else(|| anyhow::anyhow!("Wallet metadata not found"))?;
        let metadata: WalletMetadata = bincode::deserialize(&metadata_bytes)
            .with_context(|| "Failed to deserialize wallet metadata")?;
        
        let mut wallet = Self {
            db,
            core_wallet,
            metadata,
        };
        
        // Load notes
        wallet.load_notes()?;
        
        Ok(wallet)
    }
    
    fn init_database(&mut self) -> Result<()> {
        // Store metadata
        let metadata_bytes = bincode::serialize(&self.metadata)
            .with_context(|| "Failed to serialize wallet metadata")?;
        self.db.insert("metadata", metadata_bytes)?;
        
        // Initialize empty collections
        self.db.insert("sapling_notes", b"")?;
        self.db.insert("orchard_notes", b"")?;
        self.db.insert("transactions", b"")?;
        self.db.insert("nullifier_set", b"")?;
        self.db.insert("airdrop_nullifier_set", b"")?;
        
        self.db.flush()?;
        info!("Initialized wallet database");
         
        Ok(())
    }
    
    fn load_notes(&mut self) -> Result<()> {
        // Load Sapling notes
        let sapling_tree = self.db.open_tree("sapling_notes")?;
        for result in sapling_tree.iter() {
            let (key, value) = result?;
            if let Ok(note_record) = bincode::deserialize::<SaplingNoteRecord>(&value) {
                self.core_wallet.add_sapling_note(note_record.note);
            }
        }
        
        // Load Orchard notes
        let orchard_tree = self.db.open_tree("orchard_notes")?;
        for result in orchard_tree.iter() {
            let (key, value) = result?;
            if let Ok(note_record) = bincode::deserialize::<OrchardNoteRecord>(&value) {
                self.core_wallet.add_orchard_note(note_record.note);
            }
        }
        
        // Load nullifier sets
        let nullifier_tree = self.db.open_tree("nullifier_set")?;
        for result in nullifier_tree.iter() {
            let (key, _) = result?;
            let nullifier: Vec<u8> = key.to_vec();
            if nullifier.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&nullifier);
                self.core_wallet.nullifier_set.insert(arr);
            }
        }
        
        let airdrop_tree = self.db.open_tree("airdrop_nullifier_set")?;
        for result in airdrop_tree.iter() {
            let (key, _) = result?;
            let nullifier: Vec<u8> = key.to_vec();
            if nullifier.len() == 32 {
                let mut arr = [0u8; 32];
                arr.copy_from_slice(&nullifier);
                self.core_wallet.airdrop_nullifier_set.insert(arr);
            }
        }
        
        info!("Loaded {} Sapling notes, {} Orchard notes", 
              self.core_wallet.sapling_notes.len(),
              self.core_wallet.orchard_notes.len());
        
        Ok(())
    }
    
    pub fn add_sapling_note(&mut self, note: SaplingNote) -> Result<()> {
        let note_id = format!("sapling_{}", note.position);
        let created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Create note record with metadata
        let note_record = SaplingNoteRecord {
            note: note.clone(),
            created_at,
            is_spent: false,
            last_used: None,
        };
        
        // Store in database
        let note_bytes = bincode::serialize(&note_record)
            .with_context(|| "Failed to serialize Sapling note record")?;
        
        let tree = self.db.open_tree("sapling_notes")?;
        tree.insert(note_id.as_bytes(), note_bytes)?;
        tree.flush()?;
        
        // Add to core wallet
        self.core_wallet.add_sapling_note(note);
        
        info!("Added Sapling note with value {} at position {}", note.value, note.position);
        
        Ok(())
    }
    
    pub fn add_orchard_note(&mut self, note: OrchardNote) -> Result<()> {
        let note_id = format!("orchard_{}", note.position);
        let created_at = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Create note record with metadata
        let note_record = OrchardNoteRecord {
            note: note.clone(),
            created_at,
            is_spent: false,
            last_used: None,
        };
        
        // Store in database
        let note_bytes = bincode::serialize(&note_record)
            .with_context(|| "Failed to serialize Orchard note record")?;
        
        let tree = self.db.open_tree("orchard_notes")?;
        tree.insert(note_id.as_bytes(), note_bytes)?;
        tree.flush()?;
        
        // Add to core wallet
        self.core_wallet.add_orchard_note(note);
        
        info!("Added Orchard note with value {} at position {}", note.value, note.position);
        
        Ok(())
    }
    
    pub fn get_balance(&self) -> (u64, u64) {
        let mut sapling_balance: u64 = 0;
        let mut orchard_balance: u64 = 0;
        
        // Count unspent Sapling notes
        if let Some(tree) = self.db.open_tree("sapling_notes").ok() {
            for result in tree.iter() {
                if let Ok((_, value)) = result {
                    if let Ok(note_record) = bincode::deserialize::<SaplingNoteRecord>(&value) {
                        if !note_record.is_spent {
                            sapling_balance += note_record.note.value;
                        }
                    }
                }
            }
        }
        
        // Count unspent Orchard notes
        if let Some(tree) = self.db.open_tree("orchard_notes").ok() {
            for result in tree.iter() {
                if let Ok((_, value)) = result {
                    if let Ok(note_record) = bincode::deserialize::<OrchardNoteRecord>(&value) {
                        if !note_record.is_spent {
                            orchard_balance += note_record.note.value;
                        }
                    }
                }
            }
        }
        
        (sapling_balance, orchard_balance)
    }
    
    pub fn list_notes(&self, min_value: Option<u64>, note_type: Option<&str>) -> Vec<NoteMetadata> {
        let mut notes = Vec::new();
        
        if note_type.is_none() || note_type == Some("sapling") {
            if let Some(tree) = self.db.open_tree("sapling_notes").ok() {
                for result in tree.iter() {
                    if let Ok((_, value)) = result {
                        if let Ok(note_record) = bincode::deserialize::<SaplingNoteRecord>(&value) {
                            if let Some(min_val) = min_value {
                                if note_record.note.value < min_val {
                                    continue;
                                }
                            }
                            
                            notes.push(NoteMetadata {
                                note_type: "sapling".to_string(),
                                value: note_record.note.value,
                                position: note_record.note.position,
                                is_spent: note_record.is_spent,
                                created_at: note_record.created_at,
                                last_used: note_record.last_used,
                            });
                        }
                    }
                }
            }
        }
        
        if note_type.is_none() || note_type == Some("orchard") {
            if let Some(tree) = self.db.open_tree("orchard_notes").ok() {
                for result in tree.iter() {
                    if let Ok((_, value)) = result {
                        if let Ok(note_record) = bincode::deserialize::<OrchardNoteRecord>(&value) {
                            if let Some(min_val) = min_value {
                                if note_record.note.value < min_val {
                                    continue;
                                }
                            }
                            
                            notes.push(NoteMetadata {
                                note_type: "orchard".to_string(),
                                value: note_record.note.value,
                                position: note_record.note.position,
                                is_spent: note_record.is_spent,
                                created_at: note_record.created_at,
                                last_used: note_record.last_used,
                            });
                        }
                    }
                }
            }
        }
        
        notes
    }
    
    pub fn create_sapling_airdrop_tx(
        &self,
        note_index: usize,
        airdrop_amount: u64,
        recipient: &str,
    ) -> Result<ShieldedAirdropTransaction> {
        let tx = self.core_wallet.create_sapling_airdrop_tx(
            note_index,
            airdrop_amount,
            recipient.as_bytes(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create Sapling airdrop transaction: {}", e))?;
        
        // Mark the note as spent
        if note_index < self.core_wallet.sapling_notes.len() {
            let note = &self.core_wallet.sapling_notes[note_index];
            // Note: We need to make this function mutable to mark as spent
            // For now, we'll just return the transaction
        }
        
        Ok(tx)
    }
    
    pub fn create_orchard_airdrop_tx(
        &self,
        note_index: usize,
        airdrop_amount: u64,
        recipient: &str,
    ) -> Result<ShieldedAirdropTransaction> {
        let tx = self.core_wallet.create_orchard_airdrop_tx(
            note_index,
            airdrop_amount,
            recipient.as_bytes(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create Orchard airdrop transaction: {}", e))?;
        
        // Mark the note as spent
        if note_index < self.core_wallet.orchard_notes.len() {
            let note = &self.core_wallet.orchard_notes[note_index];
            // Note: We need to make this function mutable to mark as spent
            // For now, we'll just return the transaction
        }
        
        Ok(tx)
    }
    
    pub fn create_sapling_airdrop_tx_mut(
        &mut self,
        note_index: usize,
        airdrop_amount: u64,
        recipient: &str,
    ) -> Result<ShieldedAirdropTransaction> {
        let tx = self.core_wallet.create_sapling_airdrop_tx(
            note_index,
            airdrop_amount,
            recipient.as_bytes(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create Sapling airdrop transaction: {}", e))?;
        
        // Mark the note as spent
        if note_index < self.core_wallet.sapling_notes.len() {
            let note = &self.core_wallet.sapling_notes[note_index];
            self.mark_note_as_spent("sapling", note.position)?;
        }
        
        Ok(tx)
    }
    
    pub fn create_orchard_airdrop_tx_mut(
        &mut self,
        note_index: usize,
        airdrop_amount: u64,
        recipient: &str,
    ) -> Result<ShieldedAirdropTransaction> {
        let tx = self.core_wallet.create_orchard_airdrop_tx(
            note_index,
            airdrop_amount,
            recipient.as_bytes(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create Orchard airdrop transaction: {}", e))?;
        
        // Mark the note as spent
        if note_index < self.core_wallet.orchard_notes.len() {
            let note = &self.core_wallet.orchard_notes[note_index];
            self.mark_note_as_spent("orchard", note.position)?;
        }
        
        Ok(tx)
    }
    
    pub fn record_transaction(&mut self, tx: &ShieldedAirdropTransaction, tx_hash: &str) -> Result<()> {
        let airdrop_nullifier = tx.get_airdrop_nullifier();
        let amount = match &tx.claim_description {
            ClaimDescription::Sapling(claim) => claim.value_commitment[0] as u64,
            ClaimDescription::Orchard(claim) => claim.value_commitment[0] as u64,
        };
        
        let record = TransactionRecord {
            tx_hash: tx_hash.to_string(),
            airdrop_nullifier: airdrop_nullifier.to_vec(),
            amount,
            recipient: "masp_recipient".to_string(), // Would be extracted from MASP description
            status: "pending".to_string(),
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            confirmed_at: None,
            block_height: None,
        };
        
        let record_bytes = bincode::serialize(&record)
            .with_context(|| "Failed to serialize transaction record")?;
        let tree = self.db.open_tree("transactions")?;
        tree.insert(tx_hash.as_bytes(), record_bytes)?;
        Ok(())
    }
    
    pub fn get_metadata(&self) -> &WalletMetadata {
        &self.metadata
    }
    
    pub fn update_last_sync(&mut self) -> Result<()> {
        self.metadata.last_sync = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let metadata_bytes = bincode::serialize(&self.metadata)
            .with_context(|| "Failed to serialize wallet metadata")?;
        self.db.insert("metadata", metadata_bytes)?;
        
        Ok(())
    }
    
    pub fn export_data(&self) -> Result<Vec<u8>> {
        let export_data = ExportData {
            metadata: self.metadata.clone(),
            sapling_notes: self.core_wallet.sapling_notes.clone(),
            orchard_notes: self.core_wallet.orchard_notes.clone(),
            nullifier_set: self.core_wallet.nullifier_set.nullifiers.iter().cloned().collect(),
            airdrop_nullifier_set: self.core_wallet.airdrop_nullifier_set.nullifiers.iter().cloned().collect(),
        };
        
        bincode::serialize(&export_data)
            .with_context(|| "Failed to serialize export data")
    }
    
    pub fn import_data(&mut self, data: &[u8]) -> Result<()> {
        let export_data: ExportData = bincode::deserialize(data)
            .with_context(|| "Failed to deserialize import data")?;
        
        // Clear existing data
        self.core_wallet.sapling_notes.clear();
        self.core_wallet.orchard_notes.clear();
        self.core_wallet.nullifier_set.nullifiers.clear();
        self.core_wallet.airdrop_nullifier_set.nullifiers.clear();
        
        // Import new data
        for note in export_data.sapling_notes {
            self.add_sapling_note(note)?;
        }
        
        for note in export_data.orchard_notes {
            self.add_orchard_note(note)?;
        }
        
        for nullifier in export_data.nullifier_set {
            self.core_wallet.nullifier_set.insert(nullifier.try_into().unwrap());
        }
        
        for nullifier in export_data.airdrop_nullifier_set {
            self.core_wallet.airdrop_nullifier_set.insert(nullifier.try_into().unwrap());
        }
        
        info!("Imported wallet data successfully");
        Ok(())
    }
    
    pub fn mark_note_as_spent(&mut self, note_type: &str, position: u64) -> Result<()> {
        let note_id = format!("{}_{}", note_type, position);
        let tree_name = format!("{}_notes", note_type);
        
        let tree = self.db.open_tree(&tree_name)?;
        if let Some(value) = tree.get(&note_id.as_bytes())? {
            match note_type {
                "sapling" => {
                    let mut note_record: SaplingNoteRecord = bincode::deserialize(&value)
                        .with_context(|| "Failed to deserialize Sapling note record")?;
                    note_record.is_spent = true;
                    note_record.last_used = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                    
                    let updated_bytes = bincode::serialize(&note_record)
                        .with_context(|| "Failed to serialize updated Sapling note record")?;
                    tree.insert(note_id.as_bytes(), updated_bytes)?;
                    
                    info!("Marked Sapling note at position {} as spent", position);
                }
                "orchard" => {
                    let mut note_record: OrchardNoteRecord = bincode::deserialize(&value)
                        .with_context(|| "Failed to deserialize Orchard note record")?;
                    note_record.is_spent = true;
                    note_record.last_used = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                    
                    let updated_bytes = bincode::serialize(&note_record)
                        .with_context(|| "Failed to serialize updated Orchard note record")?;
                    tree.insert(note_id.as_bytes(), updated_bytes)?;
                    
                    info!("Marked Orchard note at position {} as spent", position);
                }
                _ => {
                    return Err(anyhow::anyhow!("Invalid note type: {}", note_type));
                }
            }
        } else {
            return Err(anyhow::anyhow!("Note not found: {}", note_id));
        }
        
        Ok(())
    }
    
    pub fn mark_note_as_spent_by_index(&mut self, note_type: &str, note_index: usize) -> Result<()> {
        match note_type {
            "sapling" => {
                if note_index < self.core_wallet.sapling_notes.len() {
                    let note = &self.core_wallet.sapling_notes[note_index];
                    self.mark_note_as_spent("sapling", note.position)?;
                }
            }
            "orchard" => {
                if note_index < self.core_wallet.orchard_notes.len() {
                    let note = &self.core_wallet.orchard_notes[note_index];
                    self.mark_note_as_spent("orchard", note.position)?;
                }
            }
            _ => return Err(anyhow::anyhow!("Invalid note type: {}", note_type)),
        }
        
        Ok(())
    }

    /// Create a Sapling->MASP airdrop transaction
    pub fn create_sapling_to_masp_airdrop_tx(
        &self,
        note_index: usize,
        airdrop_amount: u64,
        masp_recipient: &PublicKey,
    ) -> Result<ShieldedAirdropTransaction, ProtocolError> {
        if note_index >= self.core_wallet.sapling_notes.len() {
            return Err(ProtocolError("Invalid note index".to_string()));
        }
        let note = &self.core_wallet.sapling_notes[note_index];
        let merkle_path = vec![[0u8; 32]; 32];
        ShieldedAirdropTransaction::create_sapling_to_masp_airdrop(
            note,
            &merkle_path,
            &self.core_wallet.nullifier_set,
            airdrop_amount,
            masp_recipient,
        )
    }
    /// Create an Orchard->MASP airdrop transaction
    pub fn create_orchard_to_masp_airdrop_tx(
        &self,
        note_index: usize,
        airdrop_amount: u64,
        masp_recipient: &PublicKey,
    ) -> Result<ShieldedAirdropTransaction, ProtocolError> {
        if note_index >= self.core_wallet.orchard_notes.len() {
            return Err(ProtocolError("Invalid note index".to_string()));
        }
        let note = &self.core_wallet.orchard_notes[note_index];
        let merkle_path = vec![[0u8; 32]; 32];
        ShieldedAirdropTransaction::create_orchard_to_masp_airdrop(
            note,
            &merkle_path,
            &self.core_wallet.nullifier_set,
            airdrop_amount,
            masp_recipient,
        )
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    metadata: WalletMetadata,
    sapling_notes: Vec<SaplingNote>,
    orchard_notes: Vec<OrchardNote>,
    nullifier_set: Vec<Vec<u8>>,
    airdrop_nullifier_set: Vec<Vec<u8>>,
} 