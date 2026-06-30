use crate::block::Block;
use crate::error::ChainError;
use crate::ledger::Ledger;
use crate::transaction::Transaction;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub ledger: Ledger,
}

impl Blockchain {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            ledger: Ledger::new(),
        }
    }

    pub fn append_block(&mut self, block: Block) -> Result<(), ChainError> {
        // 1. Expected height
        let expected_height = self.blocks.len() as u64;
        if block.header.height != expected_height {
            return Err(ChainError::UnexpectedHeight {
                expected: expected_height,
                received: block.header.height,
            });
        }

        // 2. Previous block hash
        let expected_previous_hash = match self.blocks.last() {
            Some(previous) => *previous.hash().as_bytes(),
            None => [0u8; 32],
        };
        if block.header.previous_hash != expected_previous_hash {
            return Err(ChainError::InvalidPreviousHash);
        }

        // 3. Transaction commitment
        let expected_transaction_commitment = Block::transaction_commitment(&block.transactions);
        if block.header.transaction_commitment != expected_transaction_commitment {
            return Err(ChainError::InvalidTransactionCommitment);
        }

        // 4. No duplicate transaction IDs inside the block
        ensure_unique_transaction_ids(&block.transactions)?;

        // 5. Atomic execution: apply every transaction to a temporary ledger.
        // If any transaction fails, the whole block is rejected and the chain
        // ledger is left unchanged.
        let supply_before = self.ledger.total_supply();
        let mut ledger = self.ledger.clone();
        for transaction in &block.transactions {
            ledger
                .apply_transaction(transaction.clone())
                .map_err(ChainError::TransactionFailed)?;
        }

        // 6. Final state commitment matches
        let expected_state_commitment = ledger
            .state_commitment()
            .map_err(ChainError::StateCommitmentFailed)?;
        if block.header.state_commitment != expected_state_commitment {
            return Err(ChainError::InvalidStateCommitment);
        }

        // 7. Ledger invariants remain valid
        validate_ledger_invariants(&ledger, supply_before)?;

        self.ledger = ledger;
        self.blocks.push(block);
        Ok(())
    }
}

fn ensure_unique_transaction_ids(transactions: &[Transaction]) -> Result<(), ChainError> {
    let mut seen = HashSet::new();
    for transaction in transactions {
        let id = transaction.hash_id();
        if !seen.insert(id) {
            return Err(ChainError::DuplicateTransactionId(id));
        }
    }
    Ok(())
}

fn validate_ledger_invariants(ledger: &Ledger, supply_before: u64) -> Result<(), ChainError> {
    let supply_after = ledger.total_supply();
    if supply_after != supply_before {
        return Err(ChainError::TotalSupplyMismatch {
            expected: supply_before,
            actual: supply_after,
        });
    }
    Ok(())
}
