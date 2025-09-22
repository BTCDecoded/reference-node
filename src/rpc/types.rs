//! RPC types and utilities
//! 
//! Common types and utilities used across RPC methods.

use serde::{Deserialize, Serialize};
// use consensus_proof::{Block, Transaction, Hash};

/// RPC error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// RPC response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcResponse<T> {
    pub jsonrpc: String,
    pub result: Option<T>,
    pub error: Option<RpcError>,
    pub id: Option<serde_json::Value>,
}

impl<T> RpcResponse<T> {
    /// Create a successful response
    pub fn success(result: T, id: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }
    
    /// Create an error response
    pub fn error(error: RpcError, id: Option<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id,
        }
    }
}

/// Blockchain info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    pub chain: String,
    pub blocks: u64,
    pub headers: u64,
    pub bestblockhash: String,
    pub difficulty: f64,
    pub mediantime: u64,
    pub verificationprogress: f64,
    pub initialblockdownload: bool,
    pub chainwork: String,
    pub size_on_disk: u64,
    pub pruned: bool,
    pub softforks: Vec<serde_json::Value>,
    pub warnings: String,
}

/// Block info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInfo {
    pub hash: String,
    pub confirmations: u64,
    pub strippedsize: u64,
    pub size: u64,
    pub weight: u64,
    pub height: u64,
    pub version: i32,
    pub version_hex: String,
    pub merkleroot: String,
    pub tx: Vec<String>,
    pub time: u64,
    pub mediantime: u64,
    pub nonce: u64,
    pub bits: String,
    pub difficulty: f64,
    pub chainwork: String,
    pub n_tx: u64,
    pub previousblockhash: Option<String>,
    pub nextblockhash: Option<String>,
}

/// Transaction info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub txid: String,
    pub hash: String,
    pub version: i32,
    pub size: u64,
    pub vsize: u64,
    pub weight: u64,
    pub locktime: u64,
    pub vin: Vec<serde_json::Value>,
    pub vout: Vec<serde_json::Value>,
    pub hex: String,
}

/// Network info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub version: u32,
    pub subversion: String,
    pub protocolversion: u32,
    pub localservices: String,
    pub localrelay: bool,
    pub timeoffset: i64,
    pub networkactive: bool,
    pub connections: u32,
    pub networks: Vec<serde_json::Value>,
    pub relayfee: f64,
    pub incrementalfee: f64,
    pub localaddresses: Vec<serde_json::Value>,
    pub warnings: String,
}

/// Mining info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningInfo {
    pub blocks: u64,
    pub currentblocksize: u64,
    pub currentblocktx: u64,
    pub difficulty: f64,
    pub networkhashps: f64,
    pub pooledtx: u64,
    pub chain: String,
    pub warnings: String,
}

/// Block template response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTemplate {
    pub capabilities: Vec<String>,
    pub version: u32,
    pub rules: Vec<String>,
    pub vbavailable: serde_json::Value,
    pub vbrequired: u32,
    pub previousblockhash: String,
    pub transactions: Vec<serde_json::Value>,
    pub coinbaseaux: serde_json::Value,
    pub coinbasevalue: u64,
    pub longpollid: String,
    pub target: String,
    pub mintime: u64,
    pub mutable: Vec<String>,
    pub noncerange: String,
    pub sigoplimit: u64,
    pub sizelimit: u64,
    pub weightlimit: u64,
    pub curtime: u64,
    pub bits: String,
    pub height: u64,
}
