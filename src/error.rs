#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    InvalidEncoding,
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
