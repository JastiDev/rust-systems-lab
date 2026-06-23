use crate::account::Account;
use crate::error::{LedgerError, TransferValidationError};
use crate::transaction::Transaction;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transfer {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: u64,
}

pub trait Validate {
    type Error;
    fn validate(&self) -> Result<(), Self::Error>;
}

impl Validate for Transfer {
    type Error = TransferValidationError;

    fn validate(&self) -> Result<(), TransferValidationError> {
        if self.amount == 0 {
            return Err(TransferValidationError::ZeroAmount);
        }
        if self.from == self.to {
            return Err(TransferValidationError::SelfTransfer);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ledger {
    accounts: HashMap<String, Account>,
}

pub trait StateTransition<T> {
    type Error;
    fn apply(&mut self, payload: T) -> Result<(), Self::Error>;
}

impl StateTransition<Transfer> for Ledger {
    type Error = LedgerError;

    fn apply(&mut self, transfer: Transfer) -> Result<(), Self::Error> {
        // 1. Validate (immutable borrows die at end of this block)
        {
            // check if `amount is zero` or `sender and receiver are the same`
            transfer.validate()?;

            // sender does not exist
            let Some(sender) = self.account(transfer.from.clone()) else {
                return Err(LedgerError::SenderNotFound(transfer.from.clone()));
            };

            // receiver does not exist
            let Some(receiver) = self.account(transfer.to.clone()) else {
                return Err(LedgerError::ReceiverNotFound(transfer.to.clone()));
            };

            // transfer nonce does not match sender nonce
            if sender.nonce != transfer.nonce {
                return Err(LedgerError::IncorrectNonce {
                    expected: sender.nonce,
                    received: transfer.nonce,
                });
            }

            // sender does not have enough balance
            if sender.balance < transfer.amount {
                return Err(LedgerError::InsufficientBalance {
                    available: sender.balance,
                    requested: transfer.amount,
                });
            }

            // receiver balance would overflow
            if receiver.balance > u64::MAX - transfer.amount {
                return Err(LedgerError::BalanceOverflow);
            }
        } // sender & receiver borrows end here

        // 2. Mutate — one account at a time
        // For a valid transfer:

        // sender balance decreases by amount
        let sender = self.accounts.get_mut(&transfer.from).unwrap();
        sender.balance -= transfer.amount;

        // sender nonce increases by 1
        sender.nonce += 1;

        // receiver balance increases by amount
        let receiver = self.accounts.get_mut(&transfer.to).unwrap();
        receiver.balance += transfer.amount;
        // Receiver nonce should not change.

        Ok(())
    }
}

impl Ledger {
    pub fn new() -> Self {
        Ledger {
            accounts: HashMap::new(),
        }
    }

    pub fn create_account(
        &mut self,
        _id: impl Into<String>,
        balance: u64,
    ) -> Result<(), LedgerError> {
        let id = _id.into();
        // check if account id already exists
        match self.accounts.get(&id) {
            Some(_) => Err(LedgerError::AccountAlreadyExists(id.clone())),
            None => {
                self.accounts.insert(id.clone(), Account::new(balance));
                Ok(())
            }
        }
    }

    pub fn account(&self, _id: impl Into<String>) -> Option<&Account> {
        let id = _id.into();
        self.accounts.get(&id)
    }

    pub fn apply_transfer(&mut self, transfer: Transfer) -> Result<(), LedgerError> {
        self.apply(transfer)
    }

    pub fn apply_transaction(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        let transfer = Transfer {
            from: transaction.sender,
            to: transaction.receiver,
            amount: transaction.amount,
            nonce: transaction.nonce,
        };
        self.apply_transfer(transfer)
    }

    pub fn total_supply(&self) -> u64 {
        self.accounts.values().map(|acc| acc.balance).sum()
    }
}
