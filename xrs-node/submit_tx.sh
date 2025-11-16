#!/bin/bash
# Generate transaction
TX_JSON=$(cargo run --release --bin submit_transaction)
# Submit transaction
curl -X POST http://localhost:4001/submit_transaction -H "Content-Type: application/json" -d "$TX_JSON" > patent_transaction.json