#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HashError {
    CannotEncode,
    CannotDecode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockError {
    Encoding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    AccountAlreadyExists(String),
    ZeroAmount,
    SelfTransfer,
    SenderNotFound(String),
    ReceiverNotFound(String),
    InsufficientBalance { available: u64, requested: u64 },
    IncorrectNonce { expected: u64, received: u64 },
    BalanceOverflow,
}

pub enum TransactionValidationError {
    ZeroAmount,
    SelfTransfer,
}

impl From<TransactionValidationError> for LedgerError {
    fn from(value: TransactionValidationError) -> Self {
        match value {
            TransactionValidationError::ZeroAmount => LedgerError::ZeroAmount,
            TransactionValidationError::SelfTransfer => LedgerError::SelfTransfer,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainError {
    UnexpectedHeight { expected: u64, received: u64 },
    InvalidPreviousHash,
    InvalidTransactionCommitment,
    DuplicateTransactionId(crate::hash::HashedId),
    TransactionFailed(LedgerError),
    InvalidStateCommitment,
    StateCommitmentFailed(LedgerError),
    TotalSupplyMismatch { expected: u64, actual: u64 },
}
