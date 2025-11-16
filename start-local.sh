# XerisCoin Local Alpha v0.1.0

**Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.**

Run your **private XerisCoin chain locally**—test the patent-pending triple consensus (PoH 400ms + scrypt PoW + 7% PoS staking) without risking mainnet.

## Features
- **Isolated Node**: 700M XRS supply, airdrops, TX pool, Merkle proofs.
- **GPU Miner**: OpenCL scrypt (RX 390+ optimized), local hashing.
- **Wallet/Tools**: Keygen, submit TXs, stake/unstake via JS/CLI.
- **Explorer**: http://127.0.0.1:8081/blocks (view local chain).

**Zero Internet Required** | No Patent Risk | Builds Hype for Solana Token.

## Quick Start (Mac/Linux)
1. Unzip & `make all` (builds binaries).
2. `./start-local.sh` (gens keys, starts node/miner, claims airdrop).
3. Interact:
   - Mine: `./minerd -l` (logs MH/s, submits local blocks).
   - TX: `./submit_tx.sh`.
   - Explore: `curl http://127.0.0.1:8081/balances`.
4. Stop: Ctrl+C.

## Windows
Use `start-local.bat` (cross-built via Makefile or CI).

## Tech
- Rust core (Solana SDK compatible).
- C++ miner (Prometheus metrics).
- Local-only: All endpoints on 127.0.0.1.

Bugs/Feedback: zachary@xerisweb.com | Whitepaper: xerisweb.com/whitepaper.pdf

**Proprietary: Do not redistribute.**