XerisCoin Local Alpha Node v0.1.0
Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.
US Provisional Application #63/887,511
Triple Consensus (PoH + PoW + PoS) – 10k+ TPS Target
Overview
Welcome to the XerisCoin Local Alpha Node. This software lets you run a fully isolated, private blockchain on your machine—watching it mine blocks in real-time with our patent-pending triple consensus engine. See Proof-of-History (PoH) slots tick every 400ms, scrypt-based Proof-of-Work (PoW) hashes solve under dynamic difficulty, and Proof-of-Stake (PoS) validators stake 1000 XRS automatically. No internet, no peers—just pure, verifiable chain growth in your terminal.
Designed for developers, testers, and enthusiasts: Download, run, and witness 10k+ TPS potential locally. Observe slots climb from 1 to 100+ in minutes, hashes grinding nonces, and rewards accruing. This alpha proves the tech is real—before mainnet in Q1 2026.
Platform Support: macOS and Linux (x86_64).


Key Features
Genesis block with 700M XRS capped supply and 200 XRS treasury. Auto-airdrop 1000 XRS stake to a temp validator for instant mining. Local HTTP API (port 4001) for balances and blocks. Built-in explorer (port 8081) for JSON queries. Prometheus metrics for block times. Zero external dependencies beyond Rust.
Quick Start
Prerequisites
Install Rust 1.75+ via rustup.rs. Verify: rustup update. Ensure curl is available (native on macOS/Linux).
Installation
Clone the repo:
git clone https://github.com/yourusername/xrs-node.git
cd xrs-node
Build: cargo build --release
(Pre-built binaries in Releases for macOS/Linux.)
Run the Node
In your terminal:
RUST_LOG=debug ./target/release/xrs-node --local-alpha
Genesis initializes (~2s), stakes your validator, and starts mining. Leave running.
Watch the Chain Mine: Terminal View
Focus here—the heart of the experience. Terminal logs show live action:
text[INFO] Local Alpha: XRS Bootstrap node started: CHqg7iNzHFZa5SWVo6yNfsmZn9u2pBtBJfJkxMmiJTtB  
[DEBUG] Local Alpha: Validator selected as leader for slot 1  
[INFO] Block proposed: slot=1, hash=[13, 99, 2b, 79, 7e, e9, 19, 49, f3, be, a2, 59, 81, c5, 27, c3, 72, 97, 6e, a7, 32, b1, c1, e, cc, 9d, 8c, 3, 28, dd, 6f, a2], nonce=6938820065928447883  
[INFO] Local Alpha: Block 1 added, miner rewarded 342 XRS  
[INFO] Difficulty adjusted harder: target[0]=20  
[DEBUG] Local Alpha: Validator selected as leader for slot 2
Slots increment every ~400ms. Hashes solve via scrypt PoW (watch nonces climb). Rewards hit your staked pubkey. Difficulty auto-adjusts based on block times—see it tighten as mining speeds up locally.
Pro Tip: Run for 5+ minutes; screenshot slot 50+ for proof.
Verify Mining: Curl the Explorer
In a second terminal, query the chain's state (port 8081):
Current slot (watch it rise):
curl -s http://127.0.0.1:8081/blocks | jq '.[0].slot'
Recent blocks (hashes, nonces, rewards):
curl http://127.0.0.1:8081/blocks | jq '.[] | {slot: .slot, hash: (.hash | map(. | sprintf("%02x")) | join("") | .[0:16] + "..."), nonce: .nonce, reward: (.reward / 1e9 | floor)}' | head -5
Stakes (your 1000 XRS validator):
curl http://127.0.0.1:8081/stakes | jq 'map({pubkey: .pubkey[0:8] + "...", xrs: (.amount / 1e9 | floor)})'
