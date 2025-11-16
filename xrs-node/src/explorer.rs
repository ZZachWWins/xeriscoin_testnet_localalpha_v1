// Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.
// XerisCoin Explorer - Local-Only Warp Server (127.0.0.1:8081)
// Serves /blocks & /balances JSON from Ledger (Triple Consensus View)

use std::sync::{Arc, Mutex};
use warp::Filter;
use crate::ledger::Ledger;
use log::{info, debug};
use serde_json;

pub async fn start_explorer(ledger: Arc<Mutex<Ledger>>) -> Result<(), Box<dyn std::error::Error>> {
    let ledger_blocks = ledger.clone();
    let ledger_balances = ledger.clone();

    let blocks = warp::path("blocks").map(move || {
        debug!("Local Alpha: Handling /blocks request");
        let ledger = ledger_blocks.lock().unwrap();
        debug!("Local Alpha: Blocks in ledger: {:?}", ledger.blocks);
        serde_json::to_string(&ledger.blocks).unwrap()
    });

    let balances = warp::path("balances").map(move || {
        debug!("Local Alpha: Handling /balances request");
        let ledger = ledger_balances.lock().unwrap();
        debug!("Local Alpha: Balances in ledger: {:?}", ledger.balances);
        serde_json::to_string(&ledger.balances).unwrap()
    });

    let routes = blocks.or(balances);
    let addr: std::net::SocketAddr = "127.0.0.1:8081".parse().expect("Invalid address");
    info!("Local Alpha: Blockchain explorer started on http://127.0.0.1:8081 (Patent Pending)");
    warp::serve(routes).run(addr).await;
    Ok(())
}