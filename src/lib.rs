use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use serde::{Serialize, Deserialize};
use serde_bytes::{Bytes, ByteBuf};

// Remove conflicting glob imports and use specific imports
use rs_merkle::{MerkleTree};
use rs_merkle::algorithms::Sha256;

// Real cryptographic types for Zcash implementation
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct FieldElement(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct GroupElement(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct Scalar(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct ValueCommitment(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct NoteCommitment(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct Nullifier(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct PublicKey(pub [u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature(#[serde(with = "serde_bytes")] pub [u8; 64]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct MerkleRoot(pub [u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MerkleProof(pub Vec<[u8; 32]>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProofBytes(pub Vec<u8>);

// Protocol Constants
const MERKLE_DEPTH_SAPLING: usize = 32;
const MERKLE_DEPTH_ORCHARD: usize = 32;
const MERKLE_DEPTH_EXCLUSION: usize = 32;
const MAX_MONEY: u64 = 21_000_000 * 100_000_000; // Max ZEC in zatoshis

#[derive(Debug, Clone)]
pub struct ProtocolError(pub String);

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Protocol Error: {}", self.0)
    }
}

impl Error for ProtocolError {}

// ==================== CORE TYPES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaplingNote {
    pub diversifier: [u8; 11],
    pub value: u64,
    pub note_commitment: NoteCommitment,
    pub nullifier_key: Scalar,
    pub randomness: Scalar,
    pub position: u64,
}

impl SaplingNote {
    pub fn value_commitment(&self) -> ValueCommitment {
        // Mock implementation - in real code this would compute the value commitment
        let mut commitment = [0u8; 32];
        commitment[..8].copy_from_slice(&self.value.to_le_bytes());
        ValueCommitment(commitment)
    }
    
    pub fn commitment(&self) -> NoteCommitment {
        self.note_commitment
    }
    
    pub fn nullifier(&self) -> Nullifier {
        // Mock implementation - in real code this would compute the nullifier
        let mut nullifier = [0u8; 32];
        nullifier[..8].copy_from_slice(&self.position.to_le_bytes());
        Nullifier(nullifier)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchardNote {
    pub diversifier: [u8; 11],
    pub value: u64,
    pub note_commitment: NoteCommitment,
    pub nullifier_key: Scalar,
    pub randomness: Scalar,
    pub position: u64,
    pub rho: FieldElement,
    pub psi: FieldElement,
}

impl OrchardNote {
    pub fn value_commitment(&self) -> ValueCommitment {
        // Mock implementation - in real code this would compute the value commitment
        let mut commitment = [0u8; 32];
        commitment[..8].copy_from_slice(&self.value.to_le_bytes());
        ValueCommitment(commitment)
    }
    
    pub fn commitment(&self) -> NoteCommitment {
        self.note_commitment
    }
    
    pub fn nullifier(&self) -> Nullifier {
        // Mock implementation - in real code this would compute the nullifier
        let mut nullifier = [0u8; 32];
        nullifier[..8].copy_from_slice(&self.position.to_le_bytes());
        Nullifier(nullifier)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullifierSet {
    pub nullifiers: HashSet<Nullifier>,
}

impl NullifierSet {
    pub fn new() -> Self {
        Self {
            nullifiers: HashSet::new(),
        }
    }

    pub fn contains(&self, nullifier: &Nullifier) -> bool {
        self.nullifiers.contains(nullifier)
    }

    pub fn insert(&mut self, nullifier: Nullifier) {
        self.nullifiers.insert(nullifier);
    }
}

// ==================== AIRDROP NULLIFIER DERIVATION ====================

pub struct AirdropNullifierDerivation;

impl AirdropNullifierDerivation {
    /// Derive airdrop nullifier for Sapling notes
    pub fn derive_sapling_airdrop_nullifier(
        nullifier_key: &Scalar,
        rho: &Scalar,
    ) -> Result<Nullifier, ProtocolError> {
        // In real implementation: BLAKE2s-256("MASP_alt", LESBSP256(nk*) || LESBSP256(rho*))
        let mut hasher = blake2s_simd::Params::new()
            .hash_length(32)
            .personal(b"MASP_alt")
            .to_state();
        
        hasher.update(&nullifier_key.0);
        hasher.update(&rho.0);
        
        let hash = hasher.finalize();
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(hash.as_bytes());
        
        Ok(Nullifier(nullifier))
    }

    /// Derive airdrop nullifier for Orchard notes  
    pub fn derive_orchard_airdrop_nullifier(
        nullifier_key: &Scalar,
        rho: &FieldElement,
        psi: &FieldElement,
        note_commitment: &NoteCommitment,
    ) -> Result<Nullifier, ProtocolError> {
        // In real implementation: ExtractP(((PoseidonHash(nk, rho) + psi) mod q * P || K_Airdrop + cm))
        // This is a simplified mock implementation
        let mut data = Vec::new();
        data.extend_from_slice(&nullifier_key.0);
        data.extend_from_slice(&rho.0);
        data.extend_from_slice(&psi.0);
        data.extend_from_slice(&note_commitment.0);
        
        let hash = blake2s_simd::Params::new().hash_length(32).hash(&data);
        let mut nullifier = [0u8; 32];
        nullifier.copy_from_slice(&hash.as_bytes()[..32]);
        
        Ok(Nullifier(nullifier))
    }
}

// ==================== NON-MEMBERSHIP PROOFS ====================

#[derive(Debug, Clone)]
pub enum NonMembershipApproach {
    ComplementSet,
    NotBlacklisted,
}

#[derive(Debug, Clone)]
pub struct ComplementSetProof {
    pub exclusion_root: MerkleRoot,
    pub exclusion_path: MerkleProof,
    pub position: u64,
    pub start: FieldElement,
    pub end: FieldElement,
}

#[derive(Debug, Clone)]
pub struct NotBlacklistedProof {
    pub polynomial_evaluation: FieldElement,
    pub inverse: FieldElement,
}

pub struct NonMembershipProver;

impl NonMembershipProver {
    /// Generate complement set proof
    pub fn prove_complement_set(
        nullifier: &Nullifier,
        nullifier_set: &NullifierSet,
    ) -> Result<ComplementSetProof, ProtocolError> {
        // Build a Merkle tree from the nullifier set
        let leaves: Vec<[u8; 32]> = nullifier_set.nullifiers.iter().map(|n| n.0).collect();
        let tree = MerkleTree::<Sha256>::from_leaves(&leaves);
        let root = tree.root().unwrap_or([0u8; 32]);
        // Find the position of the nullifier in the leaves (if present)
        let position = leaves.iter().position(|n| n == &nullifier.0).unwrap_or(0) as u64;
        // Generate a Merkle proof for the nullifier
        let _proof = tree.proof(&[position as usize]);
        let start = nullifier.0;
        let mut end = nullifier.0;
        end[31] = end[31].wrapping_add(1);
        Ok(ComplementSetProof {
            exclusion_root: MerkleRoot(root),
            exclusion_path: MerkleProof(vec![[0u8; 32]; MERKLE_DEPTH_EXCLUSION]), // Mock path
            position,
            start: FieldElement(start),
            end: FieldElement(end),
        })
    }

    /// Generate not-blacklisted proof
    pub fn prove_not_blacklisted(
        nullifier: &Nullifier,
        nullifier_set: &NullifierSet,
    ) -> Result<NotBlacklistedProof, ProtocolError> {
        // In real implementation, this would:
        // 1. Construct polynomial P(X) = ∏(X - nf_i) for all nf_i in set
        // 2. Evaluate P(nullifier)
        // 3. Compute inverse if non-zero
        
        if nullifier_set.contains(nullifier) {
            return Err(ProtocolError("Nullifier is in blacklist".to_string()));
        }
        
        // Mock polynomial evaluation (should be non-zero)
        let polynomial_evaluation = FieldElement([1u8; 32]); // Non-zero value
        let inverse = FieldElement([1u8; 32]); // Mock inverse
        
        Ok(NotBlacklistedProof {
            polynomial_evaluation,
            inverse,
        })
    }
    
    /// Verify non-membership proof
    pub fn verify_non_membership(
        _nullifier: &Nullifier,
        _proof_type: NonMembershipApproach,
        _proof_data: &[u8],
    ) -> Result<bool, ProtocolError> {
        match _proof_type {
            NonMembershipApproach::ComplementSet => {
                // Verify Merkle path and range inclusion
                // Mock verification
                Ok(true)
            }
            NonMembershipApproach::NotBlacklisted => {
                // Verify polynomial evaluation and inverse
                // Mock verification
                Ok(true)
            }
        }
    }
}

// ==================== STATEMENTS AND CIRCUITS ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimStatementSapling {
    // Public inputs
    pub sapling_root: MerkleRoot,
    pub value_commitment: ValueCommitment,
    pub airdrop_nullifier: Nullifier,
    pub randomized_key: PublicKey,
    pub nullifier_set: Vec<Nullifier>,
    
    // Proof
    pub proof: ProofBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimStatementOrchard {
    // Public inputs
    pub orchard_root: MerkleRoot,
    pub value_commitment: ValueCommitment,
    pub airdrop_nullifier: Nullifier,
    pub randomized_key: PublicKey,
    pub nullifier_set: Vec<Nullifier>,
    
    // Proof
    pub proof: ProofBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceStatement {
    // Public inputs
    pub sapling_value_commitment: ValueCommitment,
    pub orchard_value_commitment: ValueCommitment,
    
    // Proof
    pub proof: ProofBytes,
}

pub struct CircuitProver;

impl CircuitProver {
    /// Generate Sapling claim proof
    pub fn prove_sapling_claim(
        note: &SaplingNote,
        _merkle_path: &MerkleProof,
        nullifier_set: &NullifierSet,
        _alpha: &Scalar,
    ) -> Result<ClaimStatementSapling, ProtocolError> {
        // Mock proof generation
        let airdrop_nullifier = AirdropNullifierDerivation::derive_sapling_airdrop_nullifier(
            &note.nullifier_key,
            &note.randomness,
        )?;
        
        let nullifier_list: Vec<Nullifier> = nullifier_set.nullifiers.iter().cloned().collect();
        
        Ok(ClaimStatementSapling {
            sapling_root: MerkleRoot([0u8; 32]),
            value_commitment: note.value_commitment(),
            airdrop_nullifier,
            randomized_key: PublicKey([0u8; 32]),
            nullifier_set: nullifier_list,
            proof: ProofBytes(vec![0u8; 192]),
        })
    }

    pub fn prove_orchard_claim(
        note: &OrchardNote,
        _merkle_path: &MerkleProof,
        nullifier_set: &NullifierSet,
        _alpha: &Scalar,
    ) -> Result<ClaimStatementOrchard, ProtocolError> {
        // Mock proof generation
        let airdrop_nullifier = AirdropNullifierDerivation::derive_orchard_airdrop_nullifier(
            &note.nullifier_key,
            &note.rho,
            &note.psi,
            &note.note_commitment,
        )?;
        
        let nullifier_list: Vec<Nullifier> = nullifier_set.nullifiers.iter().cloned().collect();
        
        Ok(ClaimStatementOrchard {
            orchard_root: MerkleRoot([0u8; 32]),
            value_commitment: note.value_commitment(),
            airdrop_nullifier,
            randomized_key: PublicKey([0u8; 32]),
            nullifier_set: nullifier_list,
            proof: ProofBytes(vec![0u8; 1024]), // Halo2 proof
        })
    }
    
    /// Generate equivalence proof between Sapling and Orchard value commitments
    pub fn prove_equivalence(
        _value: u64,
        _sapling_randomness: &Scalar,
        _orchard_randomness: &Scalar,
    ) -> Result<EquivalenceStatement, ProtocolError> {
        // Mock equivalence proof generation
        let proof = vec![0u8; 192]; // Mock proof
        
        Ok(EquivalenceStatement {
            sapling_value_commitment: ValueCommitment([0u8; 32]), // Would compute actual commitment
            orchard_value_commitment: ValueCommitment([0u8; 32]), // Would compute actual commitment
            proof: ProofBytes(proof),
        })
    }
    
    /// Verify claim statement
    pub fn verify_claim_sapling(claim: &ClaimStatementSapling) -> Result<bool, ProtocolError> {
        // In real implementation, verify Groth16 proof
        // Mock verification
        Ok(claim.proof.0.len() == 192)
    }
    
    pub fn verify_claim_orchard(claim: &ClaimStatementOrchard) -> Result<bool, ProtocolError> {
        // In real implementation, verify Halo2 proof
        // Mock verification
        Ok(claim.proof.0.len() == 1024)
    }
    
    pub fn verify_equivalence(equiv: &EquivalenceStatement) -> Result<bool, ProtocolError> {
        // In real implementation, verify equivalence proof
        // Mock verification
        Ok(equiv.proof.0.len() == 192)
    }
}

// ==================== TRANSACTION STRUCTURES ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDescription {
    pub value_commitment: ValueCommitment,
    pub note_commitment: NoteCommitment,
    pub ephemeral_key: PublicKey,
    pub encrypted_note: Vec<u8>,
    pub encrypted_outgoing: Vec<u8>,
    pub proof: ProofBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertDescription {
    pub convert_root: MerkleRoot,
    pub value_commitment_mint: ValueCommitment,
    pub proof: ProofBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClaimDescription {
    Sapling(ClaimStatementSapling),
    Orchard(ClaimStatementOrchard),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaspMintDescription {
    pub masp_root: MerkleRoot,
    pub value_commitment: ValueCommitment,
    pub recipient: PublicKey,
    pub proof: ProofBytes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldedAirdropTransaction {
    pub claim_description: ClaimDescription,
    pub masp_mint_description: MaspMintDescription,
    pub equivalence_description: Option<EquivalenceStatement>,
    pub binding_signature: Signature,
}

impl ShieldedAirdropTransaction {
    /// Create a new Sapling->MASP airdrop transaction
    pub fn create_sapling_to_masp_airdrop(
        claiming_note: &SaplingNote,
        _merkle_path: &MerkleProof,
        nullifier_set: &NullifierSet,
        _airdrop_amount: u64,
        masp_recipient: &PublicKey,
    ) -> Result<Self, ProtocolError> {
        // Create claim description
        let claim_description = ClaimDescription::Sapling(ClaimStatementSapling {
            sapling_root: MerkleRoot([0u8; 32]),
            value_commitment: claiming_note.value_commitment(),
            airdrop_nullifier: claiming_note.nullifier(),
            randomized_key: PublicKey([0u8; 32]),
            nullifier_set: nullifier_set.nullifiers.iter().cloned().collect(),
            proof: ProofBytes(vec![0u8; 192]),
        });

        // Create MASP mint description
        let masp_mint_description = MaspMintDescription {
            masp_root: MerkleRoot([0u8; 32]),
            value_commitment: ValueCommitment([0u8; 32]),
            recipient: masp_recipient.clone(),
            proof: ProofBytes(vec![0u8; 192]),
        };

        // Create equivalence statement (optional)
        let equivalence_description = Some(EquivalenceStatement {
            sapling_value_commitment: claiming_note.value_commitment(),
            orchard_value_commitment: ValueCommitment([0u8; 32]),
            proof: ProofBytes(vec![0u8; 192]),
        });

        // Create binding signature
        let binding_signature = Signature([0u8; 64]);

        Ok(ShieldedAirdropTransaction {
            claim_description,
            masp_mint_description,
            equivalence_description,
            binding_signature,
        })
    }

    /// Create a new Orchard->MASP airdrop transaction
    pub fn create_orchard_to_masp_airdrop(
        claiming_note: &OrchardNote,
        _merkle_path: &MerkleProof,
        nullifier_set: &NullifierSet,
        _airdrop_amount: u64,
        masp_recipient: &PublicKey,
    ) -> Result<Self, ProtocolError> {
        // Create claim description
        let claim_description = ClaimDescription::Orchard(ClaimStatementOrchard {
            orchard_root: MerkleRoot([0u8; 32]),
            value_commitment: claiming_note.value_commitment(),
            airdrop_nullifier: claiming_note.nullifier(),
            randomized_key: PublicKey([0u8; 32]),
            nullifier_set: nullifier_set.nullifiers.iter().cloned().collect(),
            proof: ProofBytes(vec![0u8; 192]),
        });

        // Create MASP mint description
        let masp_mint_description = MaspMintDescription {
            masp_root: MerkleRoot([0u8; 32]),
            value_commitment: ValueCommitment([0u8; 32]),
            recipient: masp_recipient.clone(),
            proof: ProofBytes(vec![0u8; 192]),
        };

        // Create equivalence statement (optional)
        let equivalence_description = Some(EquivalenceStatement {
            sapling_value_commitment: ValueCommitment([0u8; 32]),
            orchard_value_commitment: claiming_note.value_commitment(),
            proof: ProofBytes(vec![0u8; 192]),
        });

        // Create binding signature
        let binding_signature = Signature([0u8; 64]);

        Ok(ShieldedAirdropTransaction {
            claim_description,
            masp_mint_description,
            equivalence_description,
            binding_signature,
        })
    }
}

impl ShieldedAirdropTransaction {

    
    /// Validate the transaction
    pub fn validate(&self, airdrop_nullifier_set: &NullifierSet) -> Result<bool, ProtocolError> {
        // Verify all proofs
        match &self.claim_description {
            ClaimDescription::Sapling(claim) => {
                if !CircuitProver::verify_claim_sapling(claim)? {
                    return Ok(false);
                }
                
                // Check airdrop nullifier not already used
                if airdrop_nullifier_set.contains(&claim.airdrop_nullifier) {
                    return Ok(false);
                }
            }
            ClaimDescription::Orchard(claim) => {
                if !CircuitProver::verify_claim_orchard(claim)? {
                    return Ok(false);
                }
                
                // Check airdrop nullifier not already used
                if airdrop_nullifier_set.contains(&claim.airdrop_nullifier) {
                    return Ok(false);
                }
                
                // Verify equivalence proof if present
                if let Some(equiv) = &self.equivalence_description {
                    if !CircuitProver::verify_equivalence(equiv)? {
                        return Ok(false);
                    }
                    
                    // Check value commitments match
                    if equiv.sapling_value_commitment != claim.value_commitment {
                        return Ok(false);
                    }
                }
            }
        }
        
        // Additional validations would include:
        // - MASP mint proof verification
        // - Binding signature verification
        // - Balance equation verification
        
        Ok(true)
    }
    
    /// Extract the airdrop nullifier from this transaction
    pub fn get_airdrop_nullifier(&self) -> Nullifier {
        match &self.claim_description {
            ClaimDescription::Sapling(claim) => claim.airdrop_nullifier,
            ClaimDescription::Orchard(claim) => claim.airdrop_nullifier,
        }
    }
    
    /// Serialize transaction for network transmission
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        
        // Serialize claim description
        match &self.claim_description {
            ClaimDescription::Sapling(claim) => {
                data.push(0); // Sapling type
                data.extend_from_slice(&claim.value_commitment.0);
                data.extend_from_slice(&claim.sapling_root.0);
                data.extend_from_slice(&claim.airdrop_nullifier.0);
            }
            ClaimDescription::Orchard(claim) => {
                data.push(1); // Orchard type
                data.extend_from_slice(&claim.value_commitment.0);
                data.extend_from_slice(&claim.orchard_root.0);
                data.extend_from_slice(&claim.airdrop_nullifier.0);
            }
        }
        
        // Serialize MASP mint description
        data.extend_from_slice(&self.masp_mint_description.value_commitment.0);
        data.extend_from_slice(&self.masp_mint_description.recipient.0);
        
        // Serialize equivalence description if present
        if let Some(equiv) = &self.equivalence_description {
            data.push(1); // Present
            data.extend_from_slice(&equiv.sapling_value_commitment.0);
            data.extend_from_slice(&equiv.orchard_value_commitment.0);
        } else {
            data.push(0); // Not present
        }
        
        // Serialize binding signature
        data.extend_from_slice(&self.binding_signature.0);
        
        data
    }
}

// ==================== WALLET INTEGRATION ====================

#[derive(Debug)]
pub struct AirdropWallet {
    pub sapling_notes: Vec<SaplingNote>,
    pub orchard_notes: Vec<OrchardNote>,
    pub nullifier_set: NullifierSet,
    pub airdrop_nullifier_set: NullifierSet,
}

impl AirdropWallet {
    pub fn new() -> Self {
        Self {
            sapling_notes: Vec::new(),
            orchard_notes: Vec::new(),
            nullifier_set: NullifierSet::new(),
            airdrop_nullifier_set: NullifierSet::new(),
        }
    }
    
    /// Add a Sapling note to the wallet
    pub fn add_sapling_note(&mut self, note: SaplingNote) {
        self.sapling_notes.push(note);
    }
    
    /// Add an Orchard note to the wallet
    pub fn add_orchard_note(&mut self, note: OrchardNote) {
        self.orchard_notes.push(note);
    }
    
    /// Find eligible notes for airdrop claiming
    pub fn find_eligible_notes(&self, min_value: u64) -> (Vec<&SaplingNote>, Vec<&OrchardNote>) {
        let sapling_eligible: Vec<&SaplingNote> = self
            .sapling_notes
            .iter()
            .filter(|note| note.value >= min_value)
            .collect();
            
        let orchard_eligible: Vec<&OrchardNote> = self
            .orchard_notes
            .iter()
            .filter(|note| note.value >= min_value)
            .collect();
            
        (sapling_eligible, orchard_eligible)
    }
    
    /// Create an airdrop transaction using a Sapling note
    pub fn create_sapling_airdrop_tx(
        &self,
        note_index: usize,
        airdrop_amount: u64,
        recipient_address: &[u8],
    ) -> Result<ShieldedAirdropTransaction, ProtocolError> {
        if note_index >= self.sapling_notes.len() {
            return Err(ProtocolError("Invalid note index".to_string()));
        }
        
        let note = &self.sapling_notes[note_index];
        let merkle_path = MerkleProof(vec![[0u8; 32]; MERKLE_DEPTH_SAPLING]); // Mock path
        
        // Convert recipient_address to PublicKey
        let mut masp_recipient = [0u8; 32];
        if recipient_address.len() >= 32 {
            masp_recipient.copy_from_slice(&recipient_address[..32]);
        } else {
            masp_recipient[..recipient_address.len()].copy_from_slice(recipient_address);
        }
        
        ShieldedAirdropTransaction::create_sapling_to_masp_airdrop(
            note,
            &merkle_path,
            &self.nullifier_set,
            airdrop_amount,
            &PublicKey(masp_recipient),
        )
    }
    
    /// Create an airdrop transaction using an Orchard note
    pub fn create_orchard_airdrop_tx(
        &self,
        note_index: usize,
        airdrop_amount: u64,
        recipient_address: &[u8],
    ) -> Result<ShieldedAirdropTransaction, ProtocolError> {
        if note_index >= self.orchard_notes.len() {
            return Err(ProtocolError("Invalid note index".to_string()));
        }
        
        let note = &self.orchard_notes[note_index];
        let merkle_path = MerkleProof(vec![[0u8; 32]; MERKLE_DEPTH_ORCHARD]); // Mock path
        
        // Convert recipient_address to PublicKey
        let mut masp_recipient = [0u8; 32];
        if recipient_address.len() >= 32 {
            masp_recipient.copy_from_slice(&recipient_address[..32]);
        } else {
            masp_recipient[..recipient_address.len()].copy_from_slice(recipient_address);
        }
        
        ShieldedAirdropTransaction::create_orchard_to_masp_airdrop(
            note,
            &merkle_path,
            &self.nullifier_set,
            airdrop_amount,
            &PublicKey(masp_recipient),
        )
    }
    
    /// Process an incoming airdrop transaction (for validation)
    pub fn process_airdrop_transaction(
        &mut self,
        tx: &ShieldedAirdropTransaction,
    ) -> Result<bool, ProtocolError> {
        // Validate the transaction
        if !tx.validate(&self.airdrop_nullifier_set)? {
            return Ok(false);
        }
        
        // Add airdrop nullifier to prevent double-spending
        let airdrop_nullifier = tx.get_airdrop_nullifier();
        self.airdrop_nullifier_set.insert(airdrop_nullifier);
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_airdrop_nullifier_derivation() {
        let nk = [1u8; 32];
        let rho = [2u8; 32];
        
        let sapling_airdrop_nullifier = 
            AirdropNullifierDerivation::derive_sapling_airdrop_nullifier(&Scalar(nk), &Scalar(rho))
                .unwrap();
        
        // Should be deterministic
        let sapling_airdrop_nullifier_2 = 
            AirdropNullifierDerivation::derive_sapling_airdrop_nullifier(&Scalar(nk), &Scalar(rho))
                .unwrap();
        
        assert_eq!(sapling_airdrop_nullifier, sapling_airdrop_nullifier_2);
    }
    
    #[test]
    fn test_non_membership_proof() {
        let nullifier = [1u8; 32];
        let mut nullifier_set = NullifierSet::new();
        nullifier_set.insert(Nullifier([2u8; 32])); // Different nullifier
        
        // Should succeed for non-blacklisted approach
        let proof = NonMembershipProver::prove_not_blacklisted(&Nullifier(nullifier), &nullifier_set)
            .unwrap();
        assert_eq!(proof.polynomial_evaluation, FieldElement([1u8; 32]));
        
        // Should fail if nullifier is in set
        nullifier_set.insert(Nullifier(nullifier));
        let result = NonMembershipProver::prove_not_blacklisted(&Nullifier(nullifier), &nullifier_set);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sapling_airdrop_transaction() {
        let mut wallet = AirdropWallet::new();
        
        let note = SaplingNote {
            diversifier: [0u8; 11],
            value: 1000000, // 0.01 ZEC
            note_commitment: NoteCommitment([1u8; 32]),
            nullifier_key: Scalar([2u8; 32]),
            randomness: Scalar([3u8; 32]),
            position: 0,
        };
        
        wallet.add_sapling_note(note);
        
        let recipient = [4u8; 32];
        let tx = wallet.create_sapling_airdrop_tx(0, 500000, &recipient).unwrap();
        
        // Transaction should be valid
        assert!(wallet.process_airdrop_transaction(&tx).unwrap());
        
        // Double-spend should fail
        let tx2 = wallet.create_sapling_airdrop_tx(0, 500000, &recipient).unwrap();
        assert!(!wallet.process_airdrop_transaction(&tx2).unwrap());
    }
}

// ==================== CLI INTERFACE ====================

