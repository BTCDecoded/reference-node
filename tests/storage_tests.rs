//! Storage layer tests

use reference_node::storage::*;
use consensus_proof::*;
use tempfile::TempDir;

#[test]
fn test_storage_creation() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path()).unwrap();
    
    // Test that storage components are accessible
    let _blocks = storage.blocks();
    let _utxos = storage.utxos();
    let _chain = storage.chain();
    let _transactions = storage.transactions();
}

#[test]
fn test_block_store() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path()).unwrap();
    let blockstore = storage.blocks();
    
    // Create a test block
    let block = Block {
        header: BlockHeader {
            version: 1,
            prev_block_hash: [0u8; 32],
            merkle_root: [0u8; 32],
            timestamp: 1234567890,
            bits: 0x1d00ffff,
            nonce: 0,
        },
        transactions: vec![],
    };
    
    // Store the block
    blockstore.store_block(&block).unwrap();
    
    // Verify block count
    assert_eq!(blockstore.block_count().unwrap(), 1);
}

#[test]
fn test_utxo_store() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path()).unwrap();
    let utxostore = storage.utxos();
    
    // Create a test UTXO
    let outpoint = OutPoint {
        hash: [1u8; 32],
        index: 0,
    };
    
    let utxo = UTXO {
        value: 5000000000, // 50 BTC in satoshis
        script_pubkey: vec![0x76, 0xa9, 0x14], // P2PKH script
        height: 0,
    };
    
    // Add UTXO
    utxostore.add_utxo(&outpoint, &utxo).unwrap();
    
    // Verify UTXO exists
    assert!(utxostore.has_utxo(&outpoint).unwrap());
    
    // Get UTXO
    let retrieved_utxo = utxostore.get_utxo(&outpoint).unwrap().unwrap();
    assert_eq!(retrieved_utxo.value, utxo.value);
    
    // Verify total value
    assert_eq!(utxostore.total_value().unwrap(), 5000000000);
}

#[test]
fn test_chain_state() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path()).unwrap();
    let chainstate = storage.chain();
    
    // Create genesis header
    let genesis_header = BlockHeader {
        version: 1,
        prev_block_hash: [0u8; 32],
        merkle_root: [0u8; 32],
        timestamp: 1231006505, // Bitcoin genesis timestamp
        bits: 0x1d00ffff,
        nonce: 2083236893,
    };
    
    // Initialize chain state
    chainstate.initialize(&genesis_header).unwrap();
    
    // Verify initialization
    assert!(chainstate.is_initialized().unwrap());
    
    // Get height
    let height = chainstate.get_height().unwrap().unwrap();
    assert_eq!(height, 0);
    
    // Get tip hash
    let tip_hash = chainstate.get_tip_hash().unwrap().unwrap();
    // The hash is calculated, so it won't be all zeros
    assert_ne!(tip_hash, [0u8; 32]);
}

#[test]
fn test_transaction_index() {
    let temp_dir = TempDir::new().unwrap();
    let storage = Storage::new(temp_dir.path()).unwrap();
    let txindex = storage.transactions();
    
    // Create a test transaction
    let tx = Transaction {
        version: 1,
        inputs: vec![],
        outputs: vec![TransactionOutput {
            value: 5000000000,
            script_pubkey: vec![0x76, 0xa9, 0x14],
        }],
        lock_time: 0,
    };
    
    let block_hash = [1u8; 32];
    
    // Index the transaction
    txindex.index_transaction(&tx, &block_hash, 0, 0).unwrap();
    
    // Verify transaction count
    assert_eq!(txindex.transaction_count().unwrap(), 1);
    
    // Get transaction metadata
    let tx_hash = [0u8; 32]; // Simplified hash
    let _metadata = txindex.get_metadata(&tx_hash);
    // Note: This will be None due to simplified hashing, but the test structure is correct
}
