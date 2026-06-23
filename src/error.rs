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

pub enum TransferValidationError {
    ZeroAmount,
    SelfTransfer,
}

impl From<TransferValidationError> for LedgerError {
    fn from(value: TransferValidationError) -> Self {
        match value {
            TransferValidationError::ZeroAmount => LedgerError::ZeroAmount,
            TransferValidationError::SelfTransfer => LedgerError::SelfTransfer,
        }
    }
}
