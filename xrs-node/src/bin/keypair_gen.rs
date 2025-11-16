use solana_sdk::signature::{Keypair, Signer};
use std::fs::File;
use log::{info, debug};

fn main() {
    env_logger::init();
    let keypair = Keypair::new();
    let keypair_bytes = keypair.to_bytes().to_vec();
    debug!("Generated keypair bytes: {:?}", keypair_bytes);
    debug!("Keypair byte length: {}", keypair_bytes.len());
    let mut file = File::create("keypair.json").expect("Failed to create keypair file");
    serde_json::to_writer(&mut file, &keypair_bytes).expect("Failed to write keypair");
    info!("Keypair created at keypair.json: {:?}", keypair.pubkey());
}