use scrypt::{scrypt, Params};
   use rand::Rng;
   use std::vec::Vec;
   use crate::ledger::{Block, Ledger};
   use log::info;
   use solana_sdk::{pubkey::Pubkey, signature::Signer};

   pub fn propose_block(
       slot: u64,
       keypair: &solana_sdk::signature::Keypair,
       ledger: &std::sync::Arc<std::sync::Mutex<Ledger>>,
       poh_hash: [u8; 32],
   ) -> Result<Block, Box<dyn std::error::Error>> {
       let mut target = vec![0u8; 32];
       target[0] = 0x1f;
       let mut nonce = rand::thread_rng().gen::<u64>();
       let slot_data = format!("{:?}{}", slot, keypair.pubkey()).into_bytes();

       let ledger_guard = ledger.lock().unwrap();
       let last_block = ledger_guard.get_last_block();
       if let Some(last) = last_block {
           target = adjust_difficulty(last, slot, &ledger_guard);
       }
       let proposer_stake = ledger_guard.get_stakes().get(&keypair.pubkey()).unwrap_or(&0);
       if *proposer_stake < 1_000_000_000_000 {
           return Err("Insufficient stake to propose block".into());
       }
       drop(ledger_guard);

       loop {
           let mut input = slot_data.clone();
           input.extend_from_slice(&poh_hash);
           input.extend_from_slice(&nonce.to_be_bytes());
           let mut hash = vec![0u8; 32];
           let params = Params::new(10, 1, 1)?;
           scrypt(&input, &[], &params, &mut hash)?;
           if hash < target {
               info!("Block proposed: slot={}, hash={:x?}, nonce={}", slot, hash, nonce);
               return Ok(Block {
                   slot,
                   hash,
                   nonce,
                   transactions: Vec::new(),
               });
           }
           nonce += 1;
       }
   }

   pub fn adjust_difficulty(last_block: &Block, slot: u64, ledger: &Ledger) -> Vec<u8> {
       let mut target = last_block.hash.clone();
       let recent_blocks = ledger.blocks.iter().rev().take(10).collect::<Vec<_>>();
       let avg_block_time = if recent_blocks.len() >= 2 {
           (slot - recent_blocks.last().unwrap().slot) * 400 / recent_blocks.len() as u64
       } else {
           400
       };
       if avg_block_time > 4000 {
           target[0] = target[0].saturating_sub(1);
           info!("Difficulty adjusted easier: target[0]={}", target[0]);
       } else if avg_block_time < 3000 {
           target[0] = target[0].saturating_add(1);
           info!("Difficulty adjusted harder: target[0]={}", target[0]);
       }
       if target[0] < 0x1a {
           target[0] = 0x1a;
       }
       if target[0] > 0x1f {
           target[0] = 0x1f;
       }
       target
   }

   pub fn vote(
       block: &Block,
       validators: &[Pubkey],
       ledger: &std::sync::Arc<std::sync::Mutex<Ledger>>,
   ) -> Result<(), Box<dyn std::error::Error>> {
       let ledger_guard = ledger.lock().unwrap();
       let stakes = ledger_guard.get_stakes();
       let total_stake: u64 = stakes.values().sum();
       let mut votes: u64 = 0;
       for validator in validators {
           if let Some(stake) = stakes.get(validator) {
               votes += stake;
           }
       }
       if votes >= (total_stake * 2 / 3) {
           info!(
               "Block {} voted by {} XRS (required: {} XRS)",
               block.slot,
               votes / 1_000_000_000,
               (total_stake * 2 / 3) / 1_000_000_000
           );
           Ok(())
       } else {
           Err("Insufficient stake votes".into())
       }
   }