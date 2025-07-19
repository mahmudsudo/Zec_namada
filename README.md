# Zec_namada

## Shielded Airdrop Protocol CLI (Zcash <-> MASP/Namada)

### Features
- Create shielded airdrop transactions from Zcash Sapling or Orchard pools to MASP (Namada)
- Serialize, verify, and inspect airdrop transactions
- CLI wallet for Zcash users
- Integration-ready transaction format for Namada chain verification

### Usage

#### Create a Sapling->MASP airdrop transaction
```
zec-nam create-masp-airdrop \
    --note-index 0 \
    --amount 1000000 \
    --masp-recipient <hex_pubkey> \
    --note-type sapling \
    --out-file tx.bin
```

#### Create an Orchard->MASP airdrop transaction
```
zec-nam create-masp-airdrop \
    --note-index 0 \
    --amount 1000000 \
    --masp-recipient <hex_pubkey> \
    --note-type orchard \
    --out-file tx.bin
```

#### Verify a MASP airdrop transaction
```
zec-nam verify-masp-airdrop --tx-file tx.bin
```

#### Show MASP airdrop transaction details
```
zec-nam show-masp-airdrop-tx --tx-file tx.bin
```

### Protocol Flow
1. User imports Zcash notes (Sapling/Orchard) into the wallet.
2. User creates a MASP airdrop transaction, specifying the note and MASP recipient.
3. Transaction is serialized and can be submitted to Namada for verification/minting.
4. Namada chain or relayer verifies the Zcash claim and MASP mint proof.

### For Namada Integrators
- The transaction format is designed for easy deserialization and verification on-chain or in relayer code.
- Extend the Rust verification logic to plug in real MASP/Namada proof verification as needed.
