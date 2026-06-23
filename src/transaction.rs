use crate::error::TransactionError;
use crate::TransactionValidationError;
use bincode::config;
use bincode::serde::{decode_from_slice, encode_to_vec};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransactionId([u8; 32]);

impl TransactionId {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl std::fmt::Display for TransactionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub nonce: u64,
}

pub trait Validate {
    type Error;
    fn validate(&self) -> Result<(), Self::Error>;
}

impl Validate for Transaction {
    type Error = TransactionValidationError;

    fn validate(&self) -> Result<(), TransactionValidationError> {
        if self.amount == 0 {
            return Err(TransactionValidationError::ZeroAmount);
        }
        if self.sender == self.receiver {
            return Err(TransactionValidationError::SelfTransfer);
        }
        Ok(())
    }
}

impl Transaction {
    pub fn new(
        sender: impl Into<String>,
        receiver: impl Into<String>,
        amount: u64,
        nonce: u64,
    ) -> Self {
        Self {
            sender: sender.into(),
            receiver: receiver.into(),
            amount,
            nonce,
        }
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let config = config::standard();
        encode_to_vec(self, config).expect("transaction canonical encoding should not fail")
    }

    pub fn id(&self) -> TransactionId {
        let digest = Sha256::digest(self.canonical_bytes());
        TransactionId(digest.into())
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, TransactionError> {
        let config = config::standard();
        decode_from_slice(bytes, config)
            .map(|(transaction, _)| transaction)
            .map_err(|_| TransactionError::InvalidEncoding)
    }
}
