//! Bitcoin protocol message handling
//! 
//! Implements Bitcoin P2P protocol message serialization and deserialization.

use anyhow::Result;
use consensus_proof::{Block, Transaction, BlockHeader, Hash};
use serde::{Deserialize, Serialize};

/// Bitcoin protocol constants
pub const BITCOIN_MAGIC_MAINNET: [u8; 4] = [0xf9, 0xbe, 0xb4, 0xd9];
pub const BITCOIN_MAGIC_TESTNET: [u8; 4] = [0x0b, 0x11, 0x09, 0x07];
pub const BITCOIN_MAGIC_REGTEST: [u8; 4] = [0xfa, 0xbf, 0xb5, 0xda];

/// Maximum protocol message size (32MB)
pub const MAX_PROTOCOL_MESSAGE_LENGTH: usize = 32 * 1024 * 1024;

/// Allowed Bitcoin protocol commands
pub const ALLOWED_COMMANDS: &[&str] = &[
    "version", "verack", "ping", "pong", "getheaders", "headers",
    "getblocks", "block", "getdata", "inv", "tx", "notfound",
    "getaddr", "addr", "mempool", "reject", "feefilter", "sendcmpct",
    "cmpctblock", "getblocktxn", "blocktxn", "getblocktxn"
];

/// Bitcoin protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    Version(VersionMessage),
    Verack,
    Ping(PingMessage),
    Pong(PongMessage),
    GetHeaders(GetHeadersMessage),
    Headers(HeadersMessage),
    GetBlocks(GetBlocksMessage),
    Block(BlockMessage),
    GetData(GetDataMessage),
    Inv(InvMessage),
    Tx(TxMessage),
}

/// Version message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMessage {
    pub version: i32,
    pub services: u64,
    pub timestamp: i64,
    pub addr_recv: NetworkAddress,
    pub addr_from: NetworkAddress,
    pub nonce: u64,
    pub user_agent: String,
    pub start_height: i32,
    pub relay: bool,
}

/// Network address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAddress {
    pub services: u64,
    pub ip: [u8; 16],
    pub port: u16,
}

/// Ping message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMessage {
    pub nonce: u64,
}

/// Pong message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    pub nonce: u64,
}

/// Get headers message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetHeadersMessage {
    pub version: i32,
    pub block_locator_hashes: Vec<Hash>,
    pub hash_stop: Hash,
}

/// Headers message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadersMessage {
    pub headers: Vec<BlockHeader>,
}

/// Get blocks message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetBlocksMessage {
    pub version: i32,
    pub block_locator_hashes: Vec<Hash>,
    pub hash_stop: Hash,
}

/// Block message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMessage {
    pub block: Block,
}

/// Get data message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetDataMessage {
    pub inventory: Vec<InventoryItem>,
}

/// Inventory item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub inv_type: u32,
    pub hash: Hash,
}

/// Inventory message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvMessage {
    pub inventory: Vec<InventoryItem>,
}

/// Transaction message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxMessage {
    pub transaction: Transaction,
}

/// Bitcoin protocol message parser
pub struct ProtocolParser;

impl ProtocolParser {
    /// Parse a raw message into a protocol message
    pub fn parse_message(data: &[u8]) -> Result<ProtocolMessage> {
        // Validate message size
        if data.len() < 24 {
            return Err(anyhow::anyhow!("Message too short"));
        }
        
        if data.len() > MAX_PROTOCOL_MESSAGE_LENGTH {
            return Err(anyhow::anyhow!("Message too large"));
        }
        
        // Parse message header
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if magic != 0xd9b4bef9 {
            return Err(anyhow::anyhow!("Invalid magic number"));
        }
        
        let command = String::from_utf8_lossy(&data[4..12]).trim_end_matches('\0').to_string();
        
        // Validate command string
        if !ALLOWED_COMMANDS.contains(&command.as_str()) {
            return Err(anyhow::anyhow!("Unknown command: {}", command));
        }
        
        let payload_length = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let checksum = &data[20..24];
        
        // Validate payload length
        if payload_length as usize > MAX_PROTOCOL_MESSAGE_LENGTH - 24 {
            return Err(anyhow::anyhow!("Payload too large"));
        }
        
        if data.len() < 24 + payload_length as usize {
            return Err(anyhow::anyhow!("Incomplete message"));
        }
        
        let payload = &data[24..24 + payload_length as usize];
        
        // Verify checksum using Bitcoin double SHA256
        let calculated_checksum = Self::calculate_checksum(payload);
        if calculated_checksum != checksum {
            return Err(anyhow::anyhow!("Invalid checksum"));
        }
        
        // Parse payload based on command
        match command.as_str() {
            "version" => Ok(ProtocolMessage::Version(bincode::deserialize(payload)?)),
            "verack" => Ok(ProtocolMessage::Verack),
            "ping" => Ok(ProtocolMessage::Ping(bincode::deserialize(payload)?)),
            "pong" => Ok(ProtocolMessage::Pong(bincode::deserialize(payload)?)),
            "getheaders" => Ok(ProtocolMessage::GetHeaders(bincode::deserialize(payload)?)),
            "headers" => Ok(ProtocolMessage::Headers(bincode::deserialize(payload)?)),
            "getblocks" => Ok(ProtocolMessage::GetBlocks(bincode::deserialize(payload)?)),
            "block" => Ok(ProtocolMessage::Block(bincode::deserialize(payload)?)),
            "getdata" => Ok(ProtocolMessage::GetData(bincode::deserialize(payload)?)),
            "inv" => Ok(ProtocolMessage::Inv(bincode::deserialize(payload)?)),
            "tx" => Ok(ProtocolMessage::Tx(bincode::deserialize(payload)?)),
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }
    
    /// Serialize a protocol message to bytes
    pub fn serialize_message(message: &ProtocolMessage) -> Result<Vec<u8>> {
        let (command, payload) = match message {
            ProtocolMessage::Version(msg) => ("version", bincode::serialize(msg)?),
            ProtocolMessage::Verack => ("verack", vec![]),
            ProtocolMessage::Ping(msg) => ("ping", bincode::serialize(msg)?),
            ProtocolMessage::Pong(msg) => ("pong", bincode::serialize(msg)?),
            ProtocolMessage::GetHeaders(msg) => ("getheaders", bincode::serialize(msg)?),
            ProtocolMessage::Headers(msg) => ("headers", bincode::serialize(msg)?),
            ProtocolMessage::GetBlocks(msg) => ("getblocks", bincode::serialize(msg)?),
            ProtocolMessage::Block(msg) => ("block", bincode::serialize(msg)?),
            ProtocolMessage::GetData(msg) => ("getdata", bincode::serialize(msg)?),
            ProtocolMessage::Inv(msg) => ("inv", bincode::serialize(msg)?),
            ProtocolMessage::Tx(msg) => ("tx", bincode::serialize(msg)?),
        };
        
        let mut message = Vec::new();
        
        // Magic number
        message.extend_from_slice(&0xd9b4bef9u32.to_le_bytes());
        
        // Command (12 bytes, null-padded)
        let mut command_bytes = [0u8; 12];
        command_bytes[..command.len()].copy_from_slice(command.as_bytes());
        message.extend_from_slice(&command_bytes);
        
        // Payload length
        message.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        
        // Checksum
        let checksum = Self::calculate_checksum(&payload);
        message.extend_from_slice(&checksum);
        
        // Payload
        message.extend_from_slice(&payload);
        
        Ok(message)
    }
    
    /// Calculate message checksum
    fn calculate_checksum(payload: &[u8]) -> [u8; 4] {
        use sha2::{Sha256, Digest};
        
        let hash1 = Sha256::digest(payload);
        let hash2 = Sha256::digest(hash1);
        
        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&hash2[..4]);
        checksum
    }
}
