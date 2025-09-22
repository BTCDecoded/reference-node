//! JSON-RPC server implementation
//! 
//! Handles HTTP/WebSocket connections and routes JSON-RPC requests.

use anyhow::Result;
use serde_json::{Value, json};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, info, warn, error};

use super::{blockchain, network, mining};

/// JSON-RPC server
pub struct RpcServer {
    addr: SocketAddr,
}

impl RpcServer {
    /// Create a new RPC server
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr }
    }
    
    /// Start the RPC server
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        info!("RPC server listening on {}", self.addr);
        
        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("New RPC connection from {}", addr);
                    tokio::spawn(Self::handle_connection(stream, addr));
                }
                Err(e) => {
                    error!("Failed to accept RPC connection: {}", e);
                }
            }
        }
    }
    
    /// Handle a client connection
    async fn handle_connection(mut stream: TcpStream, addr: std::net::SocketAddr) {
        let mut buffer = [0u8; 4096];
        
        loop {
            match stream.read(&mut buffer).await {
                Ok(0) => {
                    debug!("RPC client {} disconnected", addr);
                    break;
                }
                Ok(n) => {
                    let request = String::from_utf8_lossy(&buffer[..n]);
                    debug!("RPC request from {}: {}", addr, request);
                    
                    let response = Self::process_request(&request).await;
                    let response_json = serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
                    
                    if let Err(e) = stream.write_all(response_json.as_bytes()).await {
                        warn!("Failed to send RPC response to {}: {}", addr, e);
                        break;
                    }
                }
                Err(e) => {
                    error!("Error reading from RPC client {}: {}", addr, e);
                    break;
                }
            }
        }
    }
    
    /// Process a JSON-RPC request
    async fn process_request(request: &str) -> Value {
        let request: Value = match serde_json::from_str(request) {
            Ok(req) => req,
            Err(e) => {
                return json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": "Parse error",
                        "data": e.to_string()
                    },
                    "id": null
                });
            }
        };
        
        let method = request.get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        
        let params = request.get("params").cloned().unwrap_or(json!([]));
        let id = request.get("id").cloned();
        
        let result = Self::call_method(method, params).await;
        
        match result {
            Ok(response) => {
                json!({
                    "jsonrpc": "2.0",
                    "result": response,
                    "id": id
                })
            }
            Err(e) => {
                json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": "Method not found",
                        "data": e.to_string()
                    },
                    "id": id
                })
            }
        }
    }
    
    /// Call a specific RPC method
    async fn call_method(method: &str, params: Value) -> Result<Value> {
        match method {
            // Blockchain methods
            "getblockchaininfo" => {
                let blockchain = blockchain::BlockchainRpc::new();
                Ok(blockchain.get_blockchain_info().await?)
            }
            "getblock" => {
                let blockchain = blockchain::BlockchainRpc::new();
                let hash = params.get(0).and_then(|p| p.as_str()).unwrap_or("");
                Ok(blockchain.get_block(hash).await?)
            }
            "getblockhash" => {
                let blockchain = blockchain::BlockchainRpc::new();
                let height = params.get(0).and_then(|p| p.as_u64()).unwrap_or(0);
                Ok(blockchain.get_block_hash(height).await?)
            }
            "getrawtransaction" => {
                let blockchain = blockchain::BlockchainRpc::new();
                let txid = params.get(0).and_then(|p| p.as_str()).unwrap_or("");
                Ok(blockchain.get_raw_transaction(txid).await?)
            }
            
            // Network methods
            "getnetworkinfo" => {
                let network = network::NetworkRpc::new();
                Ok(network.get_network_info().await?)
            }
            "getpeerinfo" => {
                let network = network::NetworkRpc::new();
                Ok(network.get_peer_info().await?)
            }
            
            // Mining methods
            "getmininginfo" => {
                let mining = mining::MiningRpc::new();
                Ok(mining.get_mining_info().await?)
            }
            "getblocktemplate" => {
                let mining = mining::MiningRpc::new();
                Ok(mining.get_block_template().await?)
            }
            
            _ => Err(anyhow::anyhow!("Unknown method: {}", method))
        }
    }
}
