use crate::account::Account;
use crate::error::LedgerError;
use crate::hash::{canonical_encode, hash_canonical_bytes};
use crate::transaction::{Transaction, Validate};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Ledger {
    pub accounts: HashMap<String, Account>,
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

    pub(crate) fn account_mut(&mut self, _id: impl Into<String>) -> Option<&mut Account> {
        let id = _id.into();
        self.accounts.get_mut(&id)
    }

    pub fn apply_transaction(&mut self, transaction: Transaction) -> Result<(), LedgerError> {
        // 1. Validate (immutable borrows die at end of this block)
        {
            // check if `amount is zero` or `sender and receiver are the same`
            transaction.validate()?;

            // sender does not exist
            let Some(sender) = self.account(transaction.sender.clone()) else {
                return Err(LedgerError::SenderNotFound(transaction.sender.clone()));
            };

            // receiver does not exist
            let Some(receiver) = self.account(transaction.receiver.clone()) else {
                return Err(LedgerError::ReceiverNotFound(transaction.receiver.clone()));
            };

            // transaction nonce does not match sender nonce
            if sender.nonce != transaction.nonce {
                return Err(LedgerError::IncorrectNonce {
                    expected: sender.nonce,
                    received: transaction.nonce,
                });
            }

            // sender does not have enough balance
            if sender.balance < transaction.amount {
                return Err(LedgerError::InsufficientBalance {
                    available: sender.balance,
                    requested: transaction.amount,
                });
            }

            // receiver balance would overflow
            if receiver.balance > u64::MAX - transaction.amount {
                return Err(LedgerError::BalanceOverflow);
            }
        } // sender & receiver borrows end here

        // 2. Mutate — one account at a time
        let sender = self
            .account_mut(&transaction.sender)
            .ok_or_else(|| LedgerError::SenderNotFound(transaction.sender.clone()))?;
        sender.balance -= transaction.amount;
        sender.nonce += 1;

        let receiver = self
            .account_mut(&transaction.receiver)
            .ok_or_else(|| LedgerError::ReceiverNotFound(transaction.receiver.clone()))?;
        receiver.balance += transaction.amount;

        Ok(())
    }

    pub fn total_supply(&self) -> u64 {
        self.accounts.values().map(|acc| acc.balance).sum()
    }

    pub fn state_commitment(&self) -> Result<[u8; 32], LedgerError> {
        let mut sorted_accounts: Vec<(String, Account)> = self
            .accounts
            .iter()
            .map(|(id, account)| (id.clone(), account.clone()))
            .collect();
        sorted_accounts.sort_by(|(left, _), (right, _)| left.cmp(right));

        let bytes = canonical_encode(&sorted_accounts);
        Ok(*hash_canonical_bytes(&bytes).as_bytes())
    }
}
