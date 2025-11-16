// Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.
// XerisCoin Ledger - File-Based Persistence (local-ledger.dat for Alpha)
// Merkle Trees, Airdrops, TX Finality (Triple Consensus Integration)

#[allow(deprecated)]
use solana_sdk::{pubkey::Pubkey, signature::{Keypair, Signer}, transaction::Transaction, hash::Hash, system_instruction};
use std::fs::{OpenOptions, File};
use std::io::{Write, Read, BufReader, BufRead};
use std::collections::{HashMap, HashSet};
use rs_merkle::{MerkleTree, algorithms::Sha256};
use serde::{Serialize, Deserialize};
use log::{info, error, debug};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Block {
    pub slot: u64,
    pub hash: Vec<u8>,
    pub nonce: u64,
    pub transactions: Vec<Transaction>,
}

pub struct Ledger {
    path: String,
    pub balances: HashMap<String, u64>,
    stakes: HashMap<Pubkey, u64>,
    pub blocks: Vec<Block>,
    merkle_tree: Option<MerkleTree<Sha256>>,
    tx_leaves: Vec<[u8; 32]>,
    tx_hashes: HashSet<String>,
    checkpoint_interval: u64,
    finality_slots: u64,
}

impl Ledger {
    pub fn new(path: String) -> Self {
        let mut ledger = Ledger {
            path: path.clone(),
            balances: HashMap::new(),
            stakes: HashMap::new(),
            blocks: Vec::new(),
            merkle_tree: None,
            tx_leaves: Vec::new(),
            tx_hashes: HashSet::new(),
            checkpoint_interval: 1000,
            finality_slots: 10,
        };
        let treasury_pubkey = Pubkey::new_unique();
        let initial_treasury_balance = 200_000_000 * 1_000_000_000;
        ledger.balances.insert(treasury_pubkey.to_string(), initial_treasury_balance);
        ledger.stakes.insert(treasury_pubkey, 100_000_000 * 1_000_000_000);
        // Create ledger.dat file if it doesn't exist
        if !std::path::Path::new(&path).exists() {
            match File::create(&path) {
                Ok(_) => info!("Local Alpha: Ledger file created at {}", path),
                Err(e) => error!("Failed to create ledger file at {}: {}", path, e),
            }
        }
        // Read ledger.dat to restore airdrop state
        match File::open(&path) {
            Ok(file) => {
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        if line.starts_with("Airdrop: ") {
                            let parts: Vec<&str> = line.split(" XRS to ").collect();
                            if parts.len() == 2 {
                                if let Ok(amount) = parts[0].replace("Airdrop: ", "").parse::<u64>() {
                                    let address = parts[1];
                                    let amount_lamports = amount * 1_000_000_000;
                                    debug!("Local Alpha: Restoring airdrop: {} XRS to {}", amount, address);
                                    // Update balances
                                    *ledger.balances.entry(address.to_string()).or_insert(0u64) += amount_lamports;
                                    // Deduct from treasury only if sufficient balance
                                    let treasury_balance = ledger.balances.get_mut(&treasury_pubkey.to_string()).unwrap();
                                    if *treasury_balance >= amount_lamports {
                                        *treasury_balance -= amount_lamports;
                                    } else {
                                        error!("Local Alpha: Insufficient treasury balance for airdrop restoration: {} lamports", amount_lamports);
                                        continue;
                                    }
                                    // Update stakes
                                    if let Ok(pubkey) = Pubkey::try_from(address) {
                                        *ledger.stakes.entry(pubkey).or_insert(0u64) += amount_lamports;
                                        info!("Local Alpha: Restored airdrop from ledger.dat: {} XRS to {}", amount, address);
                                    } else {
                                        error!("Local Alpha: Invalid pubkey in ledger.dat: {}", address);
                                    }
                                } else {
                                    error!("Local Alpha: Invalid amount in ledger.dat: {}", parts[0]);
                                }
                            } else {
                                error!("Local Alpha: Invalid airdrop format in ledger.dat: {}", line);
                            }
                        }
                    }
                }
            }
            Err(e) => error!("Local Alpha: Failed to read ledger file at {}: {}", path, e),
        }
        info!("Local Alpha: Ledger initialized at {}. Treasury: {} XRS, pubkey: {}", path, ledger.balances.get(&treasury_pubkey.to_string()).unwrap_or(&0) / 1_000_000_000, treasury_pubkey);
        ledger
    }

    pub fn add_block(&mut self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        if self.detect_malicious(&block) {
            info!("Local Alpha: Malicious block detected: slot={}", block.slot);
            return Err("Malicious block detected".into());
        }
        if block.slot % self.checkpoint_interval == 0 {
            self.create_checkpoint(block.slot)?;
        }
        self.blocks.push(block.clone());
        let miner_pubkey = Pubkey::new_unique();
        let miner = miner_pubkey.to_string();
        let reward = self.get_block_reward(block.slot);
        *self.balances.entry(miner.clone()).or_insert(0u64) += reward;
        *self.stakes.entry(miner_pubkey).or_insert(0u64) += 1_000_000_000u64;
        info!(
            "Local Alpha: Block {} added, miner {} rewarded {} XRS",
            block.slot,
            miner,
            reward / 1_000_000_000
        );
        Ok(())
    }

    pub fn add_transaction(
        &mut self,
        tx: Transaction,
        slot: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx_hash = hex::encode(tx.signatures[0].as_ref());
        if self.tx_hashes.contains(&tx_hash) {
            info!("Local Alpha: Double-spend attempt detected: {}", tx_hash);
            return Err("Double-spend detected".into());
        }
        let mut file = match OpenOptions::new()
            .append(true)
            .open(&self.path) {
            Ok(file) => file,
            Err(e) => {
                error!("Local Alpha: Failed to open ledger file {}: {}", self.path, e);
                return Err(format!("Ledger file issue: {}", e).into());
            }
        };
        let from = tx.signatures[0].to_string();
        let to = tx.message.account_keys[1].to_string();
        let amount = 10u64 * 1_000_000_000;
        let fee = (amount as f64 * 0.001) as u64;
        if tx.verify().is_ok() {
            if let Some(balance) = self.balances.get_mut(&from) {
                if *balance >= amount + fee {
                    *balance -= amount + fee;
                    *self.balances.entry(to.clone()).or_insert(0u64) += amount;
                    let leaf: [u8; 32] = tx.signatures[0].as_ref().try_into().map_err(|_| "Invalid signature length")?;
                    self.tx_leaves.push(leaf);
                    self.tx_hashes.insert(tx_hash.clone());
                    if !self.tx_leaves.is_empty() {
                        self.merkle_tree = Some(MerkleTree::<Sha256>::from_leaves(&self.tx_leaves));
                        info!("Local Alpha: Merkle root updated: {:?}", self.merkle_tree.as_ref().unwrap().root_hex());
                    }
                    if let Err(e) = writeln!(file, "Local Alpha: Slot {} Tx: {:?}", slot, tx) {
                        error!("Write failed to {}: {}", self.path, e);
                        return Err(format!("Write failed: {}", e).into());
                    }
                    if let Err(e) = writeln!(
                        file,
                        "Transfer: {} XRS from {} to {}, fee burned: {} XRS",
                        amount / 1_000_000_000,
                        from,
                        to,
                        fee / 1_000_000_000
                    ) {
                        error!("Write failed to {}: {}", self.path, e);
                        return Err(format!("Write failed: {}", e).into());
                    }
                    if slot >= self.finality_slots {
                        if let Err(e) = writeln!(file, "Local Alpha: Tx finalized at slot {}", slot) {
                            error!("Write failed to {}: {}", self.path, e);
                            return Err(format!("Write failed: {}", e).into());
                        }
                    }
                    info!(
                        "Local Alpha: TX added: {} XRS from {} to {}, fee burned: {} XRS",
                        amount / 1_000_000_000,
                        from,
                        to,
                        fee / 1_000_000_000
                    );
                    Ok(())
                } else {
                    info!(
                        "Local Alpha: Insufficient funds for {}: balance {}, required {}",
                        from,
                        *balance / 1_000_000_000,
                        (amount + fee) / 1_000_000_000
                    );
                    Err("Insufficient funds".into())
                }
            } else {
                info!("Local Alpha: Sender not found: {}", from);
                Err("Sender not found".into())
            }
        } else {
            info!("Local Alpha: Invalid signature for tx: {}", tx_hash);
            Err("Invalid signature".into())
        }
    }

    pub fn get_block_reward(&self, slot: u64) -> u64 {
        let halvings = slot / self.checkpoint_interval;
        let base_reward = 342_500_000_000u64;
        base_reward >> halvings
    }

    pub fn airdrop(&mut self, address: &str, amount: u64) -> Result<(), Box<dyn std::error::Error>> {
        let treasury_pubkey = self.stakes.keys().next().cloned().unwrap_or(Pubkey::new_unique());
        let treasury_balance = self.balances.get(&treasury_pubkey.to_string()).copied().unwrap_or(0u64);
        // Local Alpha: Relaxed limits for testing (no /10 cap)
        if amount <= treasury_balance && amount <= 10_000_000_000_000u64 {  // Up to 10k XRS local
            *self.balances.entry(address.to_string()).or_insert(0u64) += amount;
            *self.balances.get_mut(&treasury_pubkey.to_string()).unwrap() -= amount;
            let pubkey = Pubkey::try_from(address).map_err(|_| "Invalid pubkey")?;
            *self.stakes.entry(pubkey).or_insert(0u64) += amount;
            info!("Local Alpha: Airdrop: {} XRS to {}", amount / 1_000_000_000, address);
            let mut file = match OpenOptions::new()
                .append(true)
                .open(&self.path) {
                Ok(file) => file,
                Err(e) => {
                    error!("Local Alpha: Failed to open ledger file {}: {}", self.path, e);
                    return Err(format!("Ledger file issue: {}", e).into());
                }
            };
            if let Err(e) = writeln!(file, "Local Alpha: Airdrop: {} XRS to {}", amount / 1_000_000_000, address) {
                error!("Write failed to {}: {}", self.path, e);
                return Err(format!("Write failed: {}", e).into());
            }
            Ok(())
        } else {
            Err("Local Alpha: Airdrop limit exceeded".into())
        }
    }

    #[allow(dead_code)]
    pub fn faucet(&mut self, _address: &str, _amount: u64) -> Result<(), Box<dyn std::error::Error>> {
        #[cfg(test)]
        {
            Ok(())
        }
        #[cfg(not(test))]
        {
            Err("Local Alpha: Faucet only available in testnet".into())
        }
    }

    #[allow(dead_code)]
    pub fn stress_test(&mut self, num_txs: usize) -> Result<(), Box<dyn std::error::Error>> {
        let mock_slot = self.get_last_block().map(|b| b.slot).unwrap_or(0u64);
        let mock_blockhash = self.get_last_block().map(|b| {
            if b.hash.len() == 32 { Hash::new_from_array(b.hash.clone().try_into().unwrap()) } else { Hash::default() }
        }).unwrap_or(Hash::default());
        for i in 0..num_txs {
            let mock_keypair = Keypair::new();
            let mock_tx = Transaction::new_signed_with_payer(
                &[],
                Some(&mock_keypair.pubkey()),
                &[&mock_keypair],
                mock_blockhash,
            );
            self.add_transaction(mock_tx, mock_slot + i as u64)?;
        }
        info!("Local Alpha: Stress test completed: {} transactions processed", num_txs);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn create_liquidity_pool(
        &mut self,
        _xrs_mint: Pubkey,
        pair_mint: Pubkey,
        amount: u64,
        keypair: &Keypair,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ix = system_instruction::create_account(
            &keypair.pubkey(),
            &Pubkey::new_unique(),
            amount,
            165,
            &solana_sdk::pubkey::Pubkey::new_unique(),
        );
        let mock_blockhash = self.get_last_block().map(|b| {
            if b.hash.len() == 32 { Hash::new_from_array(b.hash.clone().try_into().unwrap()) } else { Hash::default() }
        }).unwrap_or(Hash::default());
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&keypair.pubkey()),
            &[keypair],
            mock_blockhash,
        );
        let mock_slot = self.get_last_block().map(|b| b.slot).unwrap_or(0u64);
        self.add_transaction(tx, mock_slot)?;
        info!("Local Alpha: Liquidity pool created: {} XRS paired with {}", amount / 1_000_000_000, pair_mint);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_balance(&self, address: &str) -> u64 {
        *self.balances.get(address).unwrap_or(&0u64)
    }

    pub fn get_stakes(&self) -> &HashMap<Pubkey, u64> {
        &self.stakes
    }

    pub fn get_last_block(&self) -> Option<&Block> {
        self.blocks.last()
    }

    pub fn detect_malicious(&self, block: &Block) -> bool {
        let last_block = self.get_last_block();
        if let Some(last) = last_block {
            if block.slot <= last.slot || block.hash == last.hash {
                return true;
            }
        }
        false
    }

    pub fn create_checkpoint(&self, slot: u64) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = match OpenOptions::new()
            .append(true)
            .open(&self.path) {
            Ok(file) => file,
            Err(e) => {
                error!("Local Alpha: Failed to open ledger file {}: {}", self.path, e);
                return Err(format!("Checkpoint file issue: {}", e).into());
            }
        };
        if let Err(e) = writeln!(file, "Local Alpha: Checkpoint at slot {}", slot) {
            error!("Write failed to {}: {}", self.path, e);
            return Err(format!("Write failed: {}", e).into());
        }
        info!("Local Alpha: Checkpoint created at slot {}", slot);
        Ok(())
    }
}