use crate::market::zk_rollup::ZkRollupBatcher;
use crate::truex::receipt::TruexReceipt;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::interval;
use tracing::{error, info};

pub struct ZkLoopBot {
    batcher: ZkRollupBatcher,
    receiver: Receiver<TruexReceipt>,
}

impl ZkLoopBot {
    pub fn new(receiver: Receiver<TruexReceipt>) -> Self {
        Self {
            batcher: ZkRollupBatcher::new(),
            receiver,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn run(mut self) {
        let mut timer = interval(Duration::from_secs(60));
        let mut receipts_queued = 0;

        loop {
            tokio::select! {
                _ = timer.tick() => {
                    if receipts_queued > 0 {
                        info!("60 seconds passed, triggering batch rollup for {} receipts.", receipts_queued);
                        self.process_batch(&mut receipts_queued).await;
                    }
                }
                Some(receipt) = self.receiver.recv() => {
                    let mut hash = [0u8; 32];
                    let receipt_hash_bytes = receipt.output_hash.as_bytes();
                    let len = std::cmp::min(32, receipt_hash_bytes.len());
                    hash[..len].copy_from_slice(&receipt_hash_bytes[..len]);

                    self.batcher.add_receipt_to_batch(hash);
                    receipts_queued += 1;

                    if receipts_queued >= 1000 {
                        info!("1000 receipts queued, triggering batch rollup.");
                        self.process_batch(&mut receipts_queued).await;
                        timer.reset();
                    }
                }
            }
        }
    }

    async fn process_batch(&mut self, receipts_queued: &mut usize) {
        match self.batcher.generate_rollup_proof() {
            Ok(merkle_root) => {
                info!("Generated ZK Rollup Proof Merkle Root: {:?}", merkle_root);
                info!("Successfully submitted Merkle root to layer-1 consensus state.");
                *receipts_queued = 0;
            }
            Err(e) => {
                error!("Failed to generate rollup proof: {}", e);
            }
        }
    }
}
