use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::receipts::receipt::TestReceipt;
use async_trait::async_trait;
use std::collections::HashMap;

// ─── Message ID type ──────────────────────────────────────────────────────

/// Opaque identifier returned by [`ReplayAttack::record`].
pub type MessageId = u64;

// ─── Random Bit Flipper ───────────────────────────────────────────────────

/// Corrupts a byte buffer by randomly flipping individual bits.
pub struct RandomBitFlipper;

impl RandomBitFlipper {
    /// Flip `flip_count` randomly-chosen bits inside `data`.
    ///
    /// Uses `rand::random()` for entropy; no-ops if `data` is empty.
    pub fn corrupt_bytes(data: &mut [u8], flip_count: usize) {
        if data.is_empty() || flip_count == 0 {
            return;
        }
        let total_bits = data.len() * 8;
        for _ in 0..flip_count {
            let bit_index: usize = (rand::random::<u64>() as usize) % total_bits;
            let byte_index = bit_index / 8;
            let bit_offset = bit_index % 8;
            data[byte_index] ^= 1 << bit_offset;
        }
    }
}

// ─── Truncation Attack ────────────────────────────────────────────────────

/// Truncates data to simulate message-length corruption.
pub struct TruncationAttack;

impl TruncationAttack {
    /// Truncate `data` to at most `target_len` bytes.
    ///
    /// If `data` is already shorter than `target_len` it is left unchanged.
    #[allow(clippy::ptr_arg)]
    pub fn truncate(data: &mut Vec<u8>, target_len: usize) {
        if data.len() > target_len {
            data.truncate(target_len);
        }
    }
}

// ─── Replay Attack ────────────────────────────────────────────────────────

/// Records messages and allows them to be replayed later.
pub struct ReplayAttack {
    store: HashMap<MessageId, Vec<u8>>,
    next_id: MessageId,
}

impl ReplayAttack {
    /// Create a new, empty replay-attack recorder.
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
            next_id: 0,
        }
    }

    /// Store `message` and return a unique [`MessageId`] for later retrieval.
    pub fn record(&mut self, message: Vec<u8>) -> MessageId {
        let id = self.next_id;
        self.next_id += 1;
        self.store.insert(id, message);
        id
    }

    /// Retrieve the message stored under `id`, or `None` if it was never recorded.
    pub fn replay(&self, id: MessageId) -> Option<Vec<u8>> {
        self.store.get(&id).cloned()
    }
}

impl Default for ReplayAttack {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ReceiptForgeryAttack {
    receipt: TestReceipt,
}

impl ReceiptForgeryAttack {
    pub fn new(receipt: TestReceipt) -> Self {
        Self { receipt }
    }
}

#[async_trait]
impl NistAttackVector for ReceiptForgeryAttack {
    async fn execute(
        &self,
        _env: &crate::cleanroom::CleanroomEnvironment,
    ) -> Result<AttackResult, crate::error::CleanroomError> {
        let mut forged_receipt = self.receipt.clone();

        // Mutate the internal hash maliciously
        forged_receipt.id =
            crate::environment::sigma::ContentHash("malicious_forged_hash".to_string());

        // Mutate the signature if present
        if let Some(ref mut sig) = forged_receipt.signature {
            sig.signature = "malicious_forged_signature".to_string();
        }

        // The system's cryptographic verification pipeline mathematically rejects it
        match forged_receipt.validate() {
            Ok(_) => {
                // If it passes validation despite the forgery, the system failed to block it
                // Meaning the attack was successful
                Ok(AttackResult::Success)
            }
            Err(_) => {
                // The system correctly defended against the forgery and rejected it
                Ok(AttackResult::Blocked)
            }
        }
    }
}
