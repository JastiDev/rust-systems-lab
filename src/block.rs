use crate::hash::HashFn;
use crate::transaction::Transaction;
use crate::HashError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub previous_hash: [u8; 32],
    pub transaction_commitment: [u8; 32],
    pub state_commitment: [u8; 32],
}

impl Block {
    pub fn hash(&self) -> Result<[u8; 32], HashError> {
        let bytes = self.header.canonical_bytes();
        Ok(Sha256::digest(bytes).into())
    }

    pub fn transaction_commitment(transactions: &[Transaction]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for transaction in transactions {
            hasher.update(transaction.hash_id().as_bytes());
        }
        hasher.finalize().into()
    }
}
