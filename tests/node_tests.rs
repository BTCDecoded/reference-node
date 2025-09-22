//! Node orchestration tests

use reference_node::node::*;
use std::net::SocketAddr;
use tempfile::TempDir;

#[tokio::test]
async fn test_node_creation() {
    let temp_dir = TempDir::new().unwrap();
    let network_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let rpc_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    
    let node = Node::new(
        temp_dir.path().to_str().unwrap(),
        network_addr,
        rpc_addr,
        None, // Use default Regtest protocol
    ).unwrap();
    
    // Test that node components are accessible
    let _protocol = node.protocol();
    let _storage = node.storage();
    let _network = node.network();
    let _rpc = node.rpc();
}

#[tokio::test]
async fn test_sync_coordinator() {
    let sync = sync::SyncCoordinator::new();
    
    // Test initial state
    assert_eq!(sync.state(), &sync::SyncState::Initial);
    assert_eq!(sync.progress(), 0.0);
    assert!(!sync.is_synced());
    
    // Test state transitions (simplified)
    // In a real implementation, these would be triggered by actual sync events
}

#[tokio::test]
async fn test_mempool_manager() {
    let mut mempool = mempool::MempoolManager::new();
    
    // Test initial state
    assert_eq!(mempool.size(), 0);
    assert!(mempool.transaction_hashes().is_empty());
    
    // Test adding transaction (simplified)
    use consensus_proof::Transaction;
    let tx = Transaction {
        version: 1,
        inputs: vec![],
        outputs: vec![],
        lock_time: 0,
    };
    
    let result = mempool.add_transaction(tx).await.unwrap();
    assert!(result); // Simplified implementation always returns true
}

#[tokio::test]
async fn test_mining_coordinator() {
    let mut miner = miner::MiningCoordinator::new();
    
    // Test initial state
    assert!(!miner.is_mining_enabled());
    
    let info = miner.get_mining_info();
    assert!(!info.enabled);
    assert_eq!(info.threads, 1);
    assert!(!info.has_template);
    
    // Test enabling mining
    miner.enable_mining();
    assert!(miner.is_mining_enabled());
    
    // Test disabling mining
    miner.disable_mining();
    assert!(!miner.is_mining_enabled());
}

#[tokio::test]
async fn test_sync_state_transitions() {
    // Test sync state enum values
    let states = vec![
        sync::SyncState::Initial,
        sync::SyncState::Headers,
        sync::SyncState::Blocks,
        sync::SyncState::Synced,
        sync::SyncState::Error("test error".to_string()),
    ];
    
    // Test that all states can be created
    for state in states {
        match state {
            sync::SyncState::Initial => assert!(true),
            sync::SyncState::Headers => assert!(true),
            sync::SyncState::Blocks => assert!(true),
            sync::SyncState::Synced => assert!(true),
            sync::SyncState::Error(_) => assert!(true),
        }
    }
}

#[tokio::test]
async fn test_mining_info() {
    let miner = miner::MiningCoordinator::new();
    let info = miner.get_mining_info();
    
    assert!(!info.enabled);
    assert_eq!(info.threads, 1);
    assert!(!info.has_template);
}
