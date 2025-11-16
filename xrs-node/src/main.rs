// Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.
// XerisCoin Main Entry - Local Alpha Mode with --local-alpha Flag
// Triple Consensus Node (PoH + PoW + PoS) - US Provisional #63/887,511

use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signer, SignerError}};
use std::error::Error;
use clap::{Command, Arg};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;
use crate::network::Network;
use log::{info, error, debug};
use prometheus::{Gauge, Registry};
use env_logger;  // For env_logger::init()

mod pow;
mod poh;
mod genesis;
mod ledger;
mod network;
mod staking;
mod explorer;
mod tx_pool;

use crate::ledger::Ledger;

struct Validator {
    keypair: Keypair,
    ledger: Arc<Mutex<Ledger>>, // Shared ledger
    poh_recorder: poh::PoHRecorder,
    validators: Arc<Mutex<Vec<Pubkey>>>,
    is_bootstrap: bool,
    tx_pool: Arc<Mutex<tx_pool::PriorityQueue>>,
    network: Arc<Mutex<Network>>,
    registry: Registry,
    block_time_gauge: Gauge,
}

impl Validator {
    fn new(keypair: Keypair, ledger: Arc<Mutex<Ledger>>, is_bootstrap: bool) -> Self {
        let tx_pool = Arc::new(Mutex::new(tx_pool::PriorityQueue::new()));
        let validators = Arc::new(Mutex::new(vec![keypair.pubkey()]));
        let network = Arc::new(Mutex::new(Network::new(tx_pool.clone(), validators.clone(), ledger.clone())));
        let registry = Registry::new();
        let block_time_gauge = Gauge::new("block_time_ms", "Time to produce a block").expect("Failed to create gauge");
        registry.register(Box::new(block_time_gauge.clone())).expect("Failed to register gauge");
        Validator {
            keypair,
            ledger,
            poh_recorder: poh::PoHRecorder::new(),
            validators,
            is_bootstrap,
            tx_pool,
            network,
            registry,
            block_time_gauge,
        }
    }

    fn select_leader(&self) -> Pubkey {
        debug!("Local Alpha: Selecting validator pubkey: {}", self.keypair.pubkey());
        self.keypair.pubkey()
    }

    async fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.poh_recorder.start()?;
        info!(
            "Local Alpha: XRS {} node started: {} (Patent Pending)",
            if self.is_bootstrap { "Bootstrap" } else { "Validator" },
            self.keypair.pubkey()
        );

        loop {
            let slot = self.poh_recorder.current_slot();
            if self.keypair.pubkey() == self.select_leader() {
                debug!("Local Alpha: Validator selected as leader for slot {}", slot);
                let poh_hash = self.poh_recorder.hash();
                match pow::propose_block(slot, &self.keypair, &self.ledger, poh_hash) {
                    Ok(block) => {
                        self.ledger.lock().unwrap().add_block(block)?;
                        info!("Local Alpha: Block proposed and added for slot {}", slot);
                    }
                    Err(e) => {
                        error!("Local Alpha: Failed to propose block for slot {}: {}", slot, e);
                    }
                }
            }
            self.poh_recorder.tick();
            tokio::time::sleep(std::time::Duration::from_millis(400)).await;
        }
    }
}

fn main() {
    env_logger::init();
    let matches = Command::new("XRS Node - Local Alpha v0.1.0 - Patent Pending © 2025 Xeris")
        .arg(Arg::new("genesis").long("genesis").action(clap::ArgAction::SetTrue))
        .arg(
            Arg::new("bootstrap")
                .long("bootstrap")
                .value_names(["keypair", "ledger"])
                .num_args(2)
                .value_parser(clap::value_parser!(String))
                .help("Start bootstrap node with keypair and ledger path"),
        )
        .arg(
            Arg::new("validator")
                .long("validator")
                .value_names(["bootstrap_ip", "keypair", "ledger"])
                .num_args(3)
                .value_parser(clap::value_parser!(String))
                .help("Start validator node with bootstrap IP, keypair, and ledger path"),
        )
        .arg(Arg::new("local-alpha").long("local-alpha").action(clap::ArgAction::SetTrue)
            .help("Run local-only alpha: isolated on 127.0.0.1, temp keys, genesis init"))
        .get_matches();

    if matches.get_flag("genesis") {
        genesis::generate_genesis();
        return;
    }

    // Local Alpha Mode: Auto-init everything isolated
    if matches.get_flag("local-alpha") {
        info!("Local Alpha v0.1.0 Starting - Patent Pending © 2025 Xeris (Triple Consensus)");
        genesis::generate_genesis(); // Local genesis
        let ledger_path = "local-ledger.dat".to_string();
        let mut ledger_inner = Ledger::new(ledger_path);  // Mutable for auto-stake
        let keypair = Keypair::new(); // Temp local keypair (no file load)
        info!("Local Alpha: Temp keypair generated: {}", keypair.pubkey());

        // FIX: Auto-airdrop/stake 1000 XRS to temp keypair for proposing
        let stake_amount = 1_000_000_000_000u64;  // 1000 XRS in lamports
        if let Err(e) = ledger_inner.airdrop(&keypair.pubkey().to_string(), stake_amount) {
            error!("Local Alpha: Auto-stake failed: {}", e);
        } else {
            info!("Local Alpha: Auto-staked 1000 XRS to validator: {}", keypair.pubkey());
        }
        let ledger = Arc::new(Mutex::new(ledger_inner));  // Now wrap

        let mut validator = Validator::new(keypair, ledger.clone(), true); // Bootstrap local
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            if let Err(e) = tokio::try_join!(
                async { network::start_network(ledger.clone()).await; Ok(()) },
                async { explorer::start_explorer(ledger.clone()).await; Ok(()) },
                validator.run()
            ) {
                error!("Local Alpha: Failed to start: {}", e);
            }
        });
        return;
    }

    // Original Modes (for dev; recommend --local-alpha for release)
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create tokio runtime: {}", e);
            return;
        }
    };
    rt.block_on(async {
        let ledger = Arc::new(Mutex::new(Ledger::new("ledger.dat".to_string())));
        let ledger_clone = ledger.clone();
        debug!("Starting network, explorer, and validator");
        if let Some(values) = matches.get_many::<String>("bootstrap") {
            let parts: Vec<String> = values.cloned().collect();
            let keypair_path = &parts[0];
            let ledger_path = parts[1].to_string();
            debug!("Reading keypair from {}", keypair_path);
            let keypair_bytes = match std::fs::read(keypair_path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to read keypair file {}: {}", keypair_path, e);
                    return;
                }
            };
            debug!("Keypair bytes: {:?}", keypair_bytes);
            debug!("Keypair byte length: {}", keypair_bytes.len());
            let keypair_array: Vec<u8> = match serde_json::from_slice(&keypair_bytes) {
                Ok(arr) => arr,
                Err(e) => {
                    error!("Failed to deserialize keypair JSON: {}", e);
                    return;
                }
            };
            debug!("Keypair array: {:?}", keypair_array);
            debug!("Keypair array length: {}", keypair_array.len());
            if keypair_array.len() != 64 {
                error!("Invalid keypair length: expected 64 bytes, got {}", keypair_array.len());
                return;
            }
            let keypair = match Keypair::from_bytes(&keypair_array) {  // FIXED: Use from_bytes(&Vec<u8>) -> & [u8]
                Ok(kp) => kp,
                Err(e) => {
                    error!("Failed to create keypair from bytes: {}", e);
                    return;
                }
            };
            debug!("Starting bootstrap validator with ledger {} and pubkey {}", ledger_path, keypair.pubkey());
            let mut validator = Validator::new(keypair, ledger_clone.clone(), true);
            if let Err(e) = tokio::try_join!(
                async { network::start_network(ledger_clone.clone()).await; Ok(()) },
                async { explorer::start_explorer(ledger_clone.clone()).await; Ok(()) },
                validator.run()
            ) {
                error!("Bootstrap failed: {}", e);
            }
        } else if let Some(values) = matches.get_many::<String>("validator") {
            let parts: Vec<String> = values.cloned().collect();
            let bootstrap_ip = parts[0].to_string();
            let keypair_path = &parts[1];
            let ledger_path = parts[2].to_string();
            debug!("Reading keypair from {}", keypair_path);
            let keypair_bytes = match std::fs::read(keypair_path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    error!("Failed to read keypair file {}: {}", keypair_path, e);
                    return;
                }
            };
            debug!("Keypair bytes: {:?}", keypair_bytes);
            debug!("Keypair byte length: {}", keypair_bytes.len());
            let keypair_array: Vec<u8> = match serde_json::from_slice(&keypair_bytes) {
                Ok(arr) => arr,
                Err(e) => {
                    error!("Failed to deserialize keypair JSON: {}", e);
                    return;
                }
            };
            debug!("Keypair array: {:?}", keypair_array);
            debug!("Keypair array length: {}", keypair_array.len());
            if keypair_array.len() != 64 {
                error!("Invalid keypair length: expected 64 bytes, got {}", keypair_array.len());
                return;
            }
            let keypair = match Keypair::from_bytes(&keypair_array) {  // FIXED: Use from_bytes(&Vec<u8>) -> & [u8]
                Ok(kp) => kp,
                Err(e) => {
                    error!("Failed to create keypair from bytes: {}", e);
                    return;
                }
            };
            debug!("Starting validator with ledger {} and bootstrap IP {}", ledger_path, bootstrap_ip);
            let mut validator = Validator::new(keypair, ledger_clone.clone(), false);
            if let Err(e) = tokio::try_join!(
                async { network::start_network(ledger_clone.clone()).await; Ok(()) },
                async { explorer::start_explorer(ledger_clone.clone()).await; Ok(()) },
                validator.run()
            ) {
                error!("Validator failed to connect to {}: {}", bootstrap_ip, e);
            }
        } else {
            println!(
                "Use --genesis, --bootstrap <keypair> <ledger>, --validator <bootstrap_ip> <keypair> <ledger>, or --local-alpha"
            );
        }
    });
}