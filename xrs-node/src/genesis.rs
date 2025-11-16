use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn generate_genesis() {
    let genesis_config = r#"
{
  "total_supply": 700000000,
  "pre_mined_treasury": 200000000,
  "block_reward": 342.5,
  "halving_interval": 730000,
  "decimals": 9,
  "scarcity_note": "Limited supply: 700M XRS, 200M treasury, 500M via PoW halving (342.5 XRS/block) for $50M market cap by 2025",
  "treasury_address": "XerisTrustWallet"
}
"#;
    let path = "./xrs-genesis.json".to_string();

    if let Some(parent) = Path::new(&path).parent() {
        fs::create_dir_all(parent).expect("Failed to create directory for genesis file");
    }

    let mut file = File::create(&path).expect("Failed to create genesis file");
    file.write_all(genesis_config.as_bytes())
        .expect("Failed to write genesis config");
    println!("Genesis config created at {}. Capped supply: 700M XRS.", path);
}