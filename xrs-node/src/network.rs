// Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.
// XerisCoin Network Layer - Local Alpha: 127.0.0.1 Only (Triple Consensus Broadcast Stubbed)
// US Provisional Patent #63/887,511

use tokio::net::TcpSocket;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;
use std::sync::{Arc, Mutex};
use solana_sdk::{transaction::Transaction, signature::Signature, pubkey::Pubkey};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use log::{info, error, debug};
use warp::Filter;
use crate::ledger::Ledger;
use crate::tx_pool::PrioritizedTx;
use base64;
use std::time::Instant;
use std::net::SocketAddr;
use bincode;

#[derive(Serialize, Deserialize)]
pub enum NetworkMessage {
    Transaction(Transaction),
    Block(u64, Vec<u8>, u64),
    AuthRequest(Signature, String),
}

#[derive(Serialize, Deserialize)]
struct SubmitTransactionRequest {
    tx: String, // Base64-encoded transaction
}

pub struct Network {
    tx_pool: Arc<Mutex<crate::tx_pool::PriorityQueue>>,
    validators: Arc<Mutex<Vec<Pubkey>>>,
    #[allow(dead_code)]
    whitelisted_ips: HashMap<String, bool>,
    connections_per_ip: HashMap<String, u32>,
    authenticated_nodes: HashMap<String, bool>,
    #[allow(dead_code)]
    last_connection: HashMap<String, Instant>,
    ledger: Arc<Mutex<Ledger>>,
}

impl Network {
    pub fn new(
        tx_pool: Arc<Mutex<crate::tx_pool::PriorityQueue>>,
        validators: Arc<Mutex<Vec<Pubkey>>>,
        ledger: Arc<Mutex<Ledger>>,
    ) -> Self {
        let mut whitelisted_ips = HashMap::new();
        // Local Alpha: Whitelist only localhost (override original LAN)
        whitelisted_ips.insert("127.0.0.1".to_string(), true);
        whitelisted_ips.insert("::1".to_string(), true); // IPv6 localhost
        Network {
            tx_pool,
            validators,
            whitelisted_ips,
            connections_per_ip: HashMap::new(),
            authenticated_nodes: HashMap::new(),
            last_connection: HashMap::new(),
            ledger,
        }
    }

    pub fn broadcast_transaction(&mut self, tx: &Transaction) {
        // Local Alpha: Stubbed to local queue only (no external broadcast)
        let mut tx_pool = self.tx_pool.lock().unwrap();
        if tx_pool.len() < 10_000 {
            let fee = (10.0 * 1_000_000_000.0 * 0.001) as u64;
            tx_pool.push(PrioritizedTx {
                tx: tx.clone(),
                fee,
            });
            info!("Local Alpha: Gulf Stream TX forwarded locally {:?}", tx.signatures[0]);
        } else {
            info!("Local Alpha: TX pool full, dropping {:?}", tx.signatures[0]);
        }
    }

    pub fn broadcast_block(&mut self, slot: u64, hash: &[u8], nonce: u64) {
        // Local Alpha: Stubbed to local echo only
        info!("Local Alpha: Broadcast block slot={} hash={:x?} nonce={} (isolated)", slot, hash, nonce);
    }

    #[allow(dead_code)]
    pub fn is_whitelisted(&self, ip: &str) -> bool {
        self.whitelisted_ips.contains_key(ip)
    }

    #[allow(dead_code)]
    pub fn increment_connection(&mut self, ip: &str) -> bool {
        if let Some(last) = self.last_connection.get(ip) {
            if Instant::now().duration_since(*last).as_secs() < 1 {
                info!("Rate limit exceeded for IP: {}", ip);
                return false;
            }
        }
        self.last_connection.insert(ip.to_string(), Instant::now());
        let count = self.connections_per_ip.entry(ip.to_string()).or_insert(0);
        *count += 1;
        *count <= 5
    }

    pub fn decrement_connection(&mut self, ip: &str) {
        if let Some(count) = self.connections_per_ip.get_mut(ip) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.connections_per_ip.remove(ip);
            }
        }
    }

    pub fn authenticate_node(&mut self, node_id: &str, _signature: &Signature, _pubkey: &Pubkey) -> bool {
        self.authenticated_nodes.insert(node_id.to_string(), true);
        true
    }
}

pub async fn start_network(ledger: Arc<Mutex<Ledger>>) {
    // Local Alpha: Bind to 127.0.0.1 only (override original 0.0.0.0)
    let tcp_addr: SocketAddr = "127.0.0.1:4000".parse().expect("Invalid TCP address");
    let http_addr: SocketAddr = "127.0.0.1:4001".parse().expect("Invalid HTTP address");
    debug!("Local Alpha: Attempting to bind TCP socket to {}", tcp_addr);
    let socket = match TcpSocket::new_v4() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to create TCP socket: {}", e);
            return;
        }
    };
    debug!("Created TCP socket");
    if let Err(e) = socket.set_reuseaddr(true) {
        error!("Failed to set SO_REUSEADDR: {}", e);
        return;
    }
    debug!("Set SO_REUSEADDR on TCP socket");
    if let Err(e) = socket.bind(tcp_addr) {
        error!("Failed to bind TCP socket to {}: {}", tcp_addr, e);
        return;
    }
    debug!("Successfully bound TCP socket to {}", tcp_addr);
    let listener = match socket.listen(100) {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to listen on TCP socket: {}", e);
            return;
        }
    };
    debug!("Listening on TCP socket with backlog 100");
    let semaphore = Arc::new(Semaphore::new(100));
    let network = Arc::new(Mutex::new(Network::new(
        Arc::new(Mutex::new(crate::tx_pool::PriorityQueue::new())),
        Arc::new(Mutex::new(vec![Pubkey::new_unique()])),
        ledger.clone(),
    )));
    let ledger_airdrop = ledger.clone();
    let network_submit = network.clone();
    let ledger_submit = ledger.clone();

    // Local Alpha: Add /work for miner
    let work_route = warp::path("work")
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "work": "local_slot_data",
                "poh_hash": "local_poh_hash",
                "target": "0000ffff"  // Easy local target for alpha
            }))
        });

// Local Alpha: Add /submit_block for miner
let submit_block_route = warp::path("submit_block")
    .and(warp::post())
    .and(warp::body::json())
    .map(move |body: serde_json::Value| {
        info!("Local Alpha: Block submitted: {:?}", body);
        warp::reply::with_status("Block Accepted (Local Alpha)", warp::http::StatusCode::OK)
    });

    let airdrop = warp::path!("airdrop" / String / u64)
        .map(move |address: String, amount: u64| {
            match ledger_airdrop.lock() {
                Ok(mut ledger) => match ledger.airdrop(&address, amount) {
                    Ok(()) => warp::reply::json(&format!("Local Alpha: Airdrop successful - {} XRS to {}", amount / 1_000_000_000, address)),
                    Err(e) => warp::reply::json(&format!("Local Alpha: Airdrop failed: {}", e)),
                },
                Err(e) => warp::reply::json(&format!("Local Alpha: Airdrop failed: Mutex poisoned - {}", e)),
            }
        });

    let submit_transaction = warp::path!("submit_transaction")
        .and(warp::body::json())
        .map(move |req: SubmitTransactionRequest| {
            let tx_bytes = match base64::decode(&req.tx) {
                Ok(bytes) => bytes,
                Err(e) => return warp::reply::json(&format!("Local Alpha: Invalid transaction: {}", e)),
            };
            let tx: Transaction = match bincode::deserialize(&tx_bytes) {
                Ok(tx) => tx,
                Err(e) => return warp::reply::json(&format!("Local Alpha: Invalid transaction: {}", e)),
            };
            match network_submit.lock() {
                Ok(mut network) => match ledger_submit.lock() {
                    Ok(mut ledger) => {
                        let slot = ledger.get_last_block().map(|b| b.slot).unwrap_or(0u64);
                        match ledger.add_transaction(tx.clone(), slot) {
                            Ok(()) => {
                                network.broadcast_transaction(&tx);
                                warp::reply::json(&"Local Alpha: Transaction submitted successfully")
                            }
                            Err(e) => warp::reply::json(&format!("Local Alpha: Transaction failed: {}", e)),
                        }
                    }
                    Err(e) => warp::reply::json(&format!("Local Alpha: Transaction failed: Mutex poisoned - {}", e)),
                },
                Err(e) => warp::reply::json(&format!("Local Alpha: Transaction failed: Mutex poisoned - {}", e)),
            }
        });

    let routes = airdrop.or(submit_transaction).or(work_route).or(submit_block_route);

    info!("Local Alpha: P2P network started on port 4000 (127.0.0.1 only - no TLS, Patent Pending)");
    info!("Local Alpha: HTTP endpoints started on http://127.0.0.1:4001 (airdrop, submit_transaction, work, submit_block)");

    tokio::select! {
        _ = async {
            loop {
                let permit = semaphore.clone().acquire_owned().await.unwrap();
                let (socket, addr) = match listener.accept().await {
                    Ok(s) => s,
                    Err(e) => {
                        error!("TCP accept failed: {}", e);
                        continue;
                    }
                };
                let mut stream = socket;
                let ip = addr.ip().to_string();
                // Local Alpha: Block non-local IPs
                if ip != "127.0.0.1" && ip != "::1" {
                    error!("Local Alpha: External connect blocked: {}", ip);
                    continue;
                }
                let network = network.clone();
                tokio::spawn(async move {
                    let mut buf = vec![0; 1024];
                    let n = match stream.read(&mut buf).await {
                        Ok(n) => n,
                        Err(e) => {
                            error!("Read failed from {}: {}", ip, e);
                            network.lock().unwrap().decrement_connection(&ip);
                            return;
                        }
                    };
                    let challenge = hex::encode(Sha256::digest(&buf[0..n]));
                    if !challenge.starts_with("0000") {
                        info!("Local Alpha: Connection PoW failed from IP: {}", ip);
                        network.lock().unwrap().decrement_connection(&ip);
                        return;
                    }
                    if let Ok(msg) = bincode::deserialize::<NetworkMessage>(&buf[0..n]) {
                        match msg {
                            NetworkMessage::AuthRequest(signature, node_id) => {
                                let is_authenticated = {
                                    let network_guard = network.lock().unwrap();
                                    let validators = network_guard.validators.lock().unwrap();
                                    let pubkey = validators.iter().find(|&&p| p.to_string() == node_id).copied();
                                    drop(validators);
                                    if let Some(pubkey) = pubkey {
                                        let mut network_guard = network.lock().unwrap();
                                        network_guard.authenticate_node(&node_id, &signature, &pubkey)
                                    } else {
                                        false
                                    }
                                };
                                if is_authenticated {
                                    info!("Local Alpha: Authenticated node: {}", node_id);
                                    if let Err(e) = stream.write_all(b"XRS Auth Ack").await {
                                        error!("Write failed to {}: {}", ip, e);
                                    }
                                }
                            }
                            NetworkMessage::Transaction(tx) => {
                                if tx.verify().is_ok() {
                                    info!("Local Alpha: Valid tx from {}: {:?}", ip, tx.signatures[0]);
                                    network.lock().unwrap().broadcast_transaction(&tx);
                                    if let Err(e) = stream.write_all(b"XRS Tx Ack").await {
                                        error!("Write failed to {}: {}", ip, e);
                                    }
                                }
                            }
                            NetworkMessage::Block(slot, hash, nonce) => {
                                info!(
                                    "Local Alpha: Valid block {} from {}: hash={:x?}, nonce={}",
                                    slot, ip, hash, nonce
                                );
                                network.lock().unwrap().broadcast_block(slot, &hash, nonce);
                                if let Err(e) = stream.write_all(b"XRS Block Ack").await {
                                    error!("Write failed to {}: {}", ip, e);
                                }
                            }
                        }
                    }
                    network.lock().unwrap().decrement_connection(&ip);
                    drop(permit);
                });
            }
        } => {}
        _ = warp::serve(routes).run(http_addr) => {
            debug!("Local Alpha: Warp server running on port 4001");
        }
    }
}