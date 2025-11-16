// Patent Pending Copyright © 2025 Xeris Web Co. All rights reserved.
// Priority TX Queue - Fee-Based (Local Alpha)
use solana_sdk::transaction::Transaction;
use std::cmp::Ordering;

#[allow(dead_code)]
#[derive(Clone, PartialEq, Eq)]
pub struct PrioritizedTx {
    pub tx: Transaction,
    pub fee: u64,
}

impl Ord for PrioritizedTx {
    fn cmp(&self, other: &Self) -> Ordering {
        other.fee.cmp(&self.fee)
    }
}

impl PartialOrd for PrioritizedTx {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[allow(dead_code)]
pub struct PriorityQueue {
    inner: std::collections::BinaryHeap<PrioritizedTx>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        PriorityQueue {
            inner: std::collections::BinaryHeap::new(),
        }
    }

    pub fn push(&mut self, tx: PrioritizedTx) {
        self.inner.push(tx);
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[allow(dead_code)]
    pub fn retain<F>(&mut self, f: F) where F: FnMut(&PrioritizedTx) -> bool {
        self.inner.retain(f);
    }

    #[allow(dead_code)]
    pub fn drain(&mut self) -> std::collections::binary_heap::Drain<'_, PrioritizedTx> {
        self.inner.drain()
    }
}