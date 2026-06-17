#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LedgerError {
    AccountAlreadyExists,
    SenderNotFound,
    ReceiverNotFound,
    ZeroAmount,
    SelfTransfer,
    InsufficientBalance,
    IncorrectNonce,
    BalanceOverflow,
}
