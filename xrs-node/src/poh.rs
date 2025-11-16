use sha2::{Digest, Sha256};
use chrono::Utc;

pub struct PoHRecorder {
    current_hash: [u8; 32],
    slot: u64,
}

impl PoHRecorder {
    pub fn new() -> Self {
        PoHRecorder {
            current_hash: [0; 32],
            slot: 0,
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.tick();
        Ok(())
    }

    pub fn tick(&mut self) {
        let time = Utc::now().timestamp_millis() as u128;
        let mut hasher = Sha256::new();
        hasher.update(&self.current_hash);
        hasher.update(time.to_be_bytes());
        self.current_hash = hasher.finalize().into();
        self.slot += 1;
    }

    pub fn current_slot(&self) -> u64 {
        self.slot
    }

    #[allow(dead_code)]
    pub fn hash(&self) -> [u8; 32] {
        self.current_hash
    }
}