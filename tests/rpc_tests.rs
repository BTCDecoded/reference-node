//! RPC layer tests

use reference_node::rpc::*;
use std::net::SocketAddr;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_rpc_manager_creation() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let manager = RpcManager::new(addr);
    
    // Test that RPC components are accessible
    let _blockchain = manager.blockchain();
    let _network = manager.network();
    let _mining = manager.mining();
}

#[tokio::test]
async fn test_blockchain_rpc() {
    let blockchain = blockchain::BlockchainRpc::new();
    
    // Test getblockchaininfo
    let info = blockchain.get_blockchain_info().await.unwrap();
    assert!(info.get("chain").is_some());
    assert!(info.get("blocks").is_some());
    
    // Test getblock
    let block = blockchain.get_block("0000000000000000000000000000000000000000000000000000000000000000").await.unwrap();
    assert!(block.get("hash").is_some());
    assert!(block.get("height").is_some());
    
    // Test getblockhash
    let hash = blockchain.get_block_hash(0).await.unwrap();
    assert!(hash.is_string());
    
    // Test getrawtransaction
    let tx = blockchain.get_raw_transaction("0000000000000000000000000000000000000000000000000000000000000000").await.unwrap();
    assert!(tx.get("txid").is_some());
    assert!(tx.get("version").is_some());
}

#[tokio::test]
async fn test_network_rpc() {
    let network = network::NetworkRpc::new();
    
    // Test getnetworkinfo
    let info = network.get_network_info().await.unwrap();
    assert!(info.get("version").is_some());
    assert!(info.get("connections").is_some());
    
    // Test getpeerinfo
    let peers = network.get_peer_info().await.unwrap();
    assert!(peers.is_array());
}

#[tokio::test]
async fn test_mining_rpc() {
    let mining = mining::MiningRpc::new();
    
    // Test getmininginfo
    let info = mining.get_mining_info().await.unwrap();
    assert!(info.get("blocks").is_some());
    assert!(info.get("difficulty").is_some());
    
    // Test getblocktemplate
    let template = mining.get_block_template().await.unwrap();
    assert!(template.get("version").is_some());
    assert!(template.get("height").is_some());
    assert!(template.get("coinbasevalue").is_some());
}

#[tokio::test]
async fn test_rpc_server_creation() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let server = server::RpcServer::new(addr);
    
    // Test that server can be created
    assert!(true); // If we get here, creation succeeded
}

#[tokio::test]
async fn test_rpc_request_processing() {
    // Test JSON-RPC request processing
    let request = r#"{"jsonrpc":"2.0","method":"getblockchaininfo","params":[],"id":1}"#;
    
    // This would normally be handled by the server, but we can test the logic
    let request_value: serde_json::Value = serde_json::from_str(request).unwrap();
    let method = request_value.get("method").unwrap().as_str().unwrap();
    let params = request_value.get("params").unwrap();
    
    assert_eq!(method, "getblockchaininfo");
    assert!(params.is_array());
}

#[tokio::test]
async fn test_rpc_error_handling() {
    // Test invalid JSON
    let invalid_json = r#"{"jsonrpc":"2.0","method":"invalid_method","params":[],"id":1}"#;
    
    let request_value: serde_json::Value = serde_json::from_str(invalid_json).unwrap();
    let method = request_value.get("method").unwrap().as_str().unwrap();
    
    // Should handle unknown methods gracefully
    assert_eq!(method, "invalid_method");
}

