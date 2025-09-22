//! Network layer tests

use reference_node::network::*;
use reference_node::network::peer::Peer;
use reference_node::network::inventory::InventoryManager;
use reference_node::network::relay::RelayManager;
use reference_node::network::protocol::*;
use std::net::SocketAddr;
use tokio::sync::mpsc;

#[tokio::test]
async fn test_network_manager_creation() {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let manager = NetworkManager::new(addr);
    
    assert_eq!(manager.peer_count(), 0);
    assert!(manager.peer_addresses().is_empty());
}

#[tokio::test]
async fn test_peer_creation() {
    let (tx, _rx) = mpsc::unbounded_channel();
    let addr: SocketAddr = "127.0.0.1:8333".parse().unwrap();
    
    // Create a mock stream by binding to a local address
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let local_addr = listener.local_addr().unwrap();
    
    // Connect to the listener
    let stream = tokio::net::TcpStream::connect(local_addr).await.unwrap();
    let peer = Peer::new(stream, addr, tx);
    
    assert_eq!(peer.address(), addr);
    assert!(peer.is_connected());
}

#[tokio::test]
async fn test_inventory_manager() {
    let mut inventory = InventoryManager::new();
    
    // Test initial state
    assert_eq!(inventory.inventory_count(), 0);
    assert_eq!(inventory.pending_request_count(), 0);
    
    // Test adding inventory
    let hash = [1u8; 32];
    let items = vec![
        InventoryItem {
            inv_type: 1,
            hash,
        }
    ];
    
    inventory.add_inventory("peer1", &items).unwrap();
    assert_eq!(inventory.inventory_count(), 1);
    assert!(inventory.has_inventory(&hash));
}

#[tokio::test]
async fn test_relay_manager() {
    let mut relay = RelayManager::new();
    let hash = [1u8; 32];
    
    // Test initial state
    let stats = relay.get_stats();
    assert_eq!(stats.relayed_blocks, 0);
    assert_eq!(stats.relayed_transactions, 0);
    
    // Test relay policies
    assert!(relay.should_relay_block(&hash));
    assert!(relay.should_relay_transaction(&hash));
    
    // Test marking as relayed
    relay.mark_block_relayed(hash);
    relay.mark_transaction_relayed(hash);
    
    let stats = relay.get_stats();
    assert_eq!(stats.relayed_blocks, 1);
    assert_eq!(stats.relayed_transactions, 1);
    
    // Test that items are not relayed again
    assert!(!relay.should_relay_block(&hash));
    assert!(!relay.should_relay_transaction(&hash));
}

#[tokio::test]
async fn test_protocol_parser() {
    use reference_node::network::protocol::*;
    
    // Test version message
    let version_msg = VersionMessage {
        version: 70015,
        services: 1,
        timestamp: 1234567890,
        addr_recv: NetworkAddress {
            services: 1,
            ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            port: 8333,
        },
        addr_from: NetworkAddress {
            services: 1,
            ip: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            port: 8333,
        },
        nonce: 12345,
        user_agent: "reference-node/0.1.0".to_string(),
        start_height: 0,
        relay: true,
    };
    
    let message = ProtocolMessage::Version(version_msg);
    let serialized = ProtocolParser::serialize_message(&message).unwrap();
    
    // The serialized message should have the correct structure
    assert!(serialized.len() >= 24); // Header size
    assert_eq!(&serialized[0..4], &0xd9b4bef9u32.to_le_bytes()); // Magic number
}
