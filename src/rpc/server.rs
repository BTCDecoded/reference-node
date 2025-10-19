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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_rpc_server_creation() {
        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let server = RpcServer::new(addr);
        assert_eq!(server.addr, addr);
    }

    #[tokio::test]
    async fn test_process_request_valid_json() {
        let request = r#"{"jsonrpc":"2.0","method":"getblockchaininfo","params":[],"id":1}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["id"], 1);
    }

    #[tokio::test]
    async fn test_process_request_invalid_json() {
        let request = "invalid json";
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["error"]["code"], -32700);
        assert_eq!(response["error"]["message"], "Parse error");
    }

    #[tokio::test]
    async fn test_process_request_unknown_method() {
        let request = r#"{"jsonrpc":"2.0","method":"unknown_method","params":[],"id":1}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["error"]["code"], -32601);
        assert_eq!(response["error"]["message"], "Method not found");
        assert_eq!(response["id"], 1);
    }

    #[tokio::test]
    async fn test_process_request_without_id() {
        let request = r#"{"jsonrpc":"2.0","method":"getblockchaininfo","params":[]}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["id"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_process_request_with_params() {
        let request = r#"{"jsonrpc":"2.0","method":"getblock","params":["000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"],"id":1}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["id"], 1);
    }

    #[tokio::test]
    async fn test_call_method_getblockchaininfo() {
        let result = RpcServer::call_method("getblockchaininfo", json!([])).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.get("chain").is_some());
    }

    #[tokio::test]
    async fn test_call_method_getblock() {
        let params = json!(["000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"]);
        let result = RpcServer::call_method("getblock", params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.get("hash").is_some());
    }

    #[tokio::test]
    async fn test_call_method_getblockhash() {
        let params = json!([0]);
        let result = RpcServer::call_method("getblockhash", params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_string());
    }

    #[tokio::test]
    async fn test_call_method_getrawtransaction() {
        let params = json!(["0000000000000000000000000000000000000000000000000000000000000000"]);
        let result = RpcServer::call_method("getrawtransaction", params).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.get("txid").is_some());
    }

    #[tokio::test]
    async fn test_call_method_getnetworkinfo() {
        let result = RpcServer::call_method("getnetworkinfo", json!([])).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.get("version").is_some());
    }

    #[tokio::test]
    async fn test_call_method_getpeerinfo() {
        let result = RpcServer::call_method("getpeerinfo", json!([])).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.is_array());
    }

    #[tokio::test]
    async fn test_call_method_getmininginfo() {
        let result = RpcServer::call_method("getmininginfo", json!([])).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.get("blocks").is_some());
    }

    #[tokio::test]
    async fn test_call_method_getblocktemplate() {
        let result = RpcServer::call_method("getblocktemplate", json!([])).await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.get("version").is_some());
    }

    #[tokio::test]
    async fn test_call_method_unknown_method() {
        let result = RpcServer::call_method("unknown_method", json!([])).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown method"));
    }

    #[tokio::test]
    async fn test_json_rpc_2_0_compliance() {
        let request = r#"{"jsonrpc":"2.0","method":"getblockchaininfo","params":[],"id":1}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["id"], 1);
    }

    #[tokio::test]
    async fn test_error_response_format() {
        let request = r#"{"jsonrpc":"2.0","method":"unknown_method","params":[],"id":1}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["error"].is_object());
        assert!(response["error"]["code"].is_number());
        assert!(response["error"]["message"].is_string());
        assert_eq!(response["id"], 1);
    }

    #[tokio::test]
    async fn test_parse_error_response() {
        let request = "invalid json";
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["error"]["code"], -32700);
        assert_eq!(response["error"]["message"], "Parse error");
        assert!(response["error"]["data"].is_string());
    }

    #[tokio::test]
    async fn test_method_not_found_response() {
        let request = r#"{"jsonrpc":"2.0","method":"nonexistent","params":[],"id":42}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["error"]["code"], -32601);
        assert_eq!(response["error"]["message"], "Method not found");
        assert!(response["error"]["data"].is_string());
        assert_eq!(response["id"], 42);
    }

    #[tokio::test]
    async fn test_empty_params_handling() {
        let request = r#"{"jsonrpc":"2.0","method":"getblockchaininfo","id":1}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        assert_eq!(response["id"], 1);
    }

    #[tokio::test]
    async fn test_missing_method_handling() {
        let request = r#"{"jsonrpc":"2.0","params":[],"id":1}"#;
        let response = RpcServer::process_request(request).await;
        
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["error"]["code"], -32601);
        assert_eq!(response["error"]["message"], "Method not found");
        assert_eq!(response["id"], 1);
    }

    #[tokio::test]
    async fn test_blockchain_methods_integration() {
        // Test all blockchain methods
        let methods = vec![
            "getblockchaininfo",
            "getblock",
            "getblockhash", 
            "getrawtransaction"
        ];
        
        for method in methods {
            let request = format!(r#"{{"jsonrpc":"2.0","method":"{}","params":[],"id":1}}"#, method);
            let response = RpcServer::process_request(&request).await;
            
            assert_eq!(response["jsonrpc"], "2.0");
            assert!(response["result"].is_object() || response["result"].is_string());
            assert_eq!(response["id"], 1);
        }
    }

    #[tokio::test]
    async fn test_network_methods_integration() {
        // Test all network methods
        let methods = vec![
            "getnetworkinfo",
            "getpeerinfo"
        ];
        
        for method in methods {
            let request = format!(r#"{{"jsonrpc":"2.0","method":"{}","params":[],"id":1}}"#, method);
            let response = RpcServer::process_request(&request).await;
            
            assert_eq!(response["jsonrpc"], "2.0");
            assert!(response["result"].is_object() || response["result"].is_array());
            assert_eq!(response["id"], 1);
        }
    }

    #[tokio::test]
    async fn test_mining_methods_integration() {
        // Test all mining methods
        let methods = vec![
            "getmininginfo",
            "getblocktemplate"
        ];
        
        for method in methods {
            let request = format!(r#"{{"jsonrpc":"2.0","method":"{}","params":[],"id":1}}"#, method);
            let response = RpcServer::process_request(&request).await;
            
            assert_eq!(response["jsonrpc"], "2.0");
            assert!(response["result"].is_object());
            assert_eq!(response["id"], 1);
        }
    }
}
