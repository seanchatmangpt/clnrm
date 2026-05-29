use crate::chaos::nist_core::{AttackResult, NistAttackVector};
use crate::receipts::receipt::TestReceipt;
use async_trait::async_trait;

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
        forged_receipt.id = crate::environment::sigma::ContentHash("malicious_forged_hash".to_string());
        
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
