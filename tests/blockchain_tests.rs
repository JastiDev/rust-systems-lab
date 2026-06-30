use rust_systems_lab::{Block, BlockHeader, Blockchain, ChainError, LedgerError, Transaction};

fn valid_block_with_transactions(chain: &Blockchain, transactions: Vec<Transaction>) -> Block {
    let height = chain
        .blocks
        .len()
        .try_into()
        .expect("Length should be able to converted into u64");
    let previous_hash = match chain.blocks.last() {
        Some(block) => *block.hash().as_bytes(),
        None => [0u8; 32],
    };

    let mut ledger = chain.ledger.clone();
    for transaction in &transactions {
        ledger
            .apply_transaction(transaction.clone())
            .expect("transactions should be valid for this block");
    }
    let state_commitment = ledger
        .state_commitment()
        .expect("ledger should have a state commitment");

    Block {
        header: BlockHeader {
            height,
            previous_hash,
            transaction_commitment: Block::transaction_commitment(&transactions),
            state_commitment,
        },
        transactions,
    }
}

fn valid_next_block_for_chain(chain: &Blockchain) -> Block {
    let previous_hash = match chain.blocks.last() {
        Some(block) => *block.hash().as_bytes(),
        None => [0u8; 32],
    };

    let previous_trxns: Vec<Transaction> = match chain.blocks.last() {
        Some(block) => block.transactions.clone(),
        None => Vec::new(),
    };

    let height = chain
        .blocks
        .len()
        .try_into()
        .expect("Length should be able to converted into u64");
    let state_commitment = chain
        .ledger
        .state_commitment()
        .expect("ledger should have a state commitment");

    Block {
        header: BlockHeader {
            height,
            previous_hash,
            transaction_commitment: Block::transaction_commitment(&previous_trxns),
            state_commitment,
        },
        transactions: previous_trxns,
    }
}

fn chain_with_accounts_and_genesis() -> Blockchain {
    let mut chain = Blockchain::new();
    chain.ledger.create_account("alice", 100).expect("account");
    chain.ledger.create_account("bob", 100).expect("account");
    chain
        .append_block(valid_next_block_for_chain(&chain))
        .expect("valid genesis");
    chain
}

fn uncommitted_block(chain: &Blockchain, transactions: Vec<Transaction>) -> Block {
    Block {
        header: BlockHeader {
            height: chain.blocks.len() as u64,
            previous_hash: *chain.blocks.last().unwrap().hash().as_bytes(),
            transaction_commitment: Block::transaction_commitment(&transactions),
            state_commitment: [0u8; 32],
        },
        transactions,
    }
}

#[test]
fn failed_block_does_not_change_ledger() {
    let mut chain = chain_with_accounts_and_genesis();
    let ledger_before = chain.ledger.clone();
    let state_commitment_before = chain
        .ledger
        .state_commitment()
        .expect("state commitment");

    let block = uncommitted_block(
        &chain,
        vec![Transaction::new("alice", "bob", 500, 0)],
    );
    assert!(chain.append_block(block).is_err());

    assert_eq!(chain.ledger, ledger_before);
    assert_eq!(
        chain.ledger.state_commitment().expect("state commitment"),
        state_commitment_before,
    );
    assert_eq!(chain.ledger.total_supply(), 200);
}

#[test]
fn failed_block_does_not_change_block_count() {
    let mut chain = chain_with_accounts_and_genesis();
    let block_count_before = chain.blocks.len();

    let block = uncommitted_block(
        &chain,
        vec![Transaction::new("alice", "bob", 500, 0)],
    );
    assert!(chain.append_block(block).is_err());

    assert_eq!(chain.blocks.len(), block_count_before);
}

#[test]
fn valid_prefix_invalid_second_transaction_discards_entire_block() {
    let mut chain = chain_with_accounts_and_genesis();
    let original = chain.clone();

    let transactions = vec![
        Transaction::new("alice", "bob", 10, 0),
        Transaction::new("alice", "bob", 500, 1),
    ];
    let block = uncommitted_block(&chain, transactions);

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::TransactionFailed(
            LedgerError::InsufficientBalance {
                available: 90,
                requested: 500,
            },
        )),
    );
    assert_eq!(chain, original);

    let alice = chain.ledger.account("alice").expect("alice");
    let bob = chain.ledger.account("bob").expect("bob");
    assert_eq!(alice.balance, 100);
    assert_eq!(bob.balance, 100);
    assert_eq!(alice.nonce, 0);
}

#[test]
fn wrong_nonce_midway_discards_entire_block() {
    let mut chain = chain_with_accounts_and_genesis();
    let original = chain.clone();

    let transactions = vec![
        Transaction::new("alice", "bob", 10, 0),
        Transaction::new("alice", "bob", 10, 2),
    ];
    let block = uncommitted_block(&chain, transactions);

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::TransactionFailed(
            LedgerError::IncorrectNonce {
                expected: 1,
                received: 2,
            },
        )),
    );
    assert_eq!(chain, original);

    let alice = chain.ledger.account("alice").expect("alice");
    let bob = chain.ledger.account("bob").expect("bob");
    assert_eq!(alice.balance, 100);
    assert_eq!(bob.balance, 100);
    assert_eq!(alice.nonce, 0);
}

#[test]
fn insufficient_funds_midway_discards_entire_block() {
    let mut chain = chain_with_accounts_and_genesis();
    let original = chain.clone();

    let transactions = vec![
        Transaction::new("alice", "bob", 10, 0),
        Transaction::new("alice", "bob", 10, 1),
        Transaction::new("alice", "bob", 500, 2),
    ];
    let block = uncommitted_block(&chain, transactions);

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::TransactionFailed(
            LedgerError::InsufficientBalance {
                available: 80,
                requested: 500,
            },
        )),
    );
    assert_eq!(chain, original);

    let alice = chain.ledger.account("alice").expect("alice");
    let bob = chain.ledger.account("bob").expect("bob");
    assert_eq!(alice.balance, 100);
    assert_eq!(bob.balance, 100);
    assert_eq!(alice.nonce, 0);
}

#[test]
fn rejects_wrong_block_height() {
    let mut chain = Blockchain::new();
    let original = chain.clone();
    let mut block_0 = valid_next_block_for_chain(&chain);
    block_0.header.height = 1;
    assert_eq!(
        chain.append_block(block_0.clone()),
        Err(ChainError::UnexpectedHeight {
            expected: 0,
            received: 1,
        }),
    );
    assert_eq!(chain, original);

    block_0.header.height = 0;
    chain.append_block(block_0).expect("valid");

    let original1 = chain.clone();

    let mut block_1 = valid_next_block_for_chain(&chain);
    block_1.header.height = 2;
    assert_eq!(
        chain.append_block(block_1),
        Err(ChainError::UnexpectedHeight {
            expected: 1,
            received: 2,
        }),
    );

    assert_eq!(chain, original1);
}

#[test]
fn rejects_wrong_previous_hash() {
    let mut chain = Blockchain::new();
    let original = chain.clone();

    let mut genesis = valid_next_block_for_chain(&chain);
    genesis.header.previous_hash[0] ^= 0xff;
    assert_eq!(
        chain.append_block(genesis),
        Err(ChainError::InvalidPreviousHash),
    );
    assert_eq!(chain, original);

    chain
        .append_block(valid_next_block_for_chain(&chain))
        .expect("valid genesis");

    let original_after_genesis = chain.clone();

    let mut block_1 = valid_next_block_for_chain(&chain);
    block_1.header.previous_hash[0] ^= 0xff;
    assert_eq!(
        chain.append_block(block_1),
        Err(ChainError::InvalidPreviousHash),
    );
    assert_eq!(chain, original_after_genesis);
}

#[test]
fn rejects_invalid_transaction_commitment() {
    let mut chain = Blockchain::new();
    let original = chain.clone();

    let mut genesis = valid_next_block_for_chain(&chain);
    genesis.header.transaction_commitment[0] ^= 0xff;
    assert_eq!(
        chain.append_block(genesis),
        Err(ChainError::InvalidTransactionCommitment),
    );
    assert_eq!(chain, original);

    chain.ledger.create_account("alice", 100).expect("account");
    chain.ledger.create_account("bob", 100).expect("account");
    chain
        .append_block(valid_next_block_for_chain(&chain))
        .expect("valid genesis");

    let original_after_genesis = chain.clone();
    let transactions = vec![Transaction::new("alice", "bob", 10, 0)];
    let mut block = valid_block_with_transactions(&chain, transactions.clone());

    assert_eq!(block.transactions, transactions);
    block.header.transaction_commitment[0] ^= 0xff;

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::InvalidTransactionCommitment),
    );
    assert_eq!(chain, original_after_genesis);
}

#[test]
fn rejects_duplicate_transaction_ids_inside_block() {
    let mut chain = chain_with_accounts_and_genesis();
    let original = chain.clone();

    let transaction = Transaction::new("alice", "bob", 10, 0);
    let duplicate_id = transaction.hash_id();
    let transactions = vec![transaction.clone(), transaction];
    let block = Block {
        header: BlockHeader {
            height: 1,
            previous_hash: *chain.blocks[0].hash().as_bytes(),
            transaction_commitment: Block::transaction_commitment(&transactions),
            state_commitment: [0u8; 32],
        },
        transactions,
    };

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::DuplicateTransactionId(duplicate_id)),
    );
    assert_eq!(chain, original);
}

#[test]
fn rejects_block_when_later_transaction_fails() {
    let mut chain = chain_with_accounts_and_genesis();
    let original = chain.clone();

    let mut transactions = Vec::new();
    for nonce in 0..9 {
        transactions.push(Transaction::new("alice", "bob", 10, nonce));
    }
    transactions.push(Transaction::new("alice", "bob", 500, 9));

    let block = Block {
        header: BlockHeader {
            height: 1,
            previous_hash: *chain.blocks[0].hash().as_bytes(),
            transaction_commitment: Block::transaction_commitment(&transactions),
            state_commitment: [0u8; 32],
        },
        transactions,
    };

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::TransactionFailed(
            LedgerError::InsufficientBalance {
                available: 10,
                requested: 500,
            },
        )),
    );
    assert_eq!(chain, original);
}

#[test]
fn rejects_block_with_invalid_transaction() {
    let mut chain = chain_with_accounts_and_genesis();
    let original = chain.clone();

    let transaction = Transaction::new("alice", "bob", 500, 0);
    let transactions = vec![transaction];
    let block = Block {
        header: BlockHeader {
            height: 1,
            previous_hash: *chain.blocks[0].hash().as_bytes(),
            transaction_commitment: Block::transaction_commitment(&transactions),
            state_commitment: [0u8; 32],
        },
        transactions,
    };

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::TransactionFailed(
            LedgerError::InsufficientBalance {
                available: 100,
                requested: 500,
            },
        )),
    );
    assert_eq!(chain, original);
}

#[test]
fn rejects_block_with_wrong_final_state_commitment() {
    let mut chain = chain_with_accounts_and_genesis();
    let original = chain.clone();

    let transactions = vec![Transaction::new("alice", "bob", 10, 0)];
    let mut block = valid_block_with_transactions(&chain, transactions);
    block.header.state_commitment[0] ^= 0xff;

    assert_eq!(
        chain.append_block(block),
        Err(ChainError::InvalidStateCommitment),
    );
    assert_eq!(chain, original);
}

#[test]
fn accepts_valid_block() {
    let mut chain = Blockchain::new();
    chain.ledger.create_account("alice", 100).expect("account");
    chain.ledger.create_account("bob", 100).expect("account");

    chain
        .append_block(valid_next_block_for_chain(&chain))
        .expect("genesis should be valid");
    assert_eq!(chain.blocks.len(), 1);
    assert_eq!(chain.blocks[0].header.height, 0);

    let alice = chain.ledger.account("alice").expect("alice should exist");
    let bob = chain.ledger.account("bob").expect("bob should exist");
    assert_eq!(alice.balance, 100);
    assert_eq!(bob.balance, 100);

    let block =
        valid_block_with_transactions(&chain, vec![Transaction::new("alice", "bob", 10, 0)]);
    chain.append_block(block).expect("block should be valid");

    assert_eq!(chain.blocks.len(), 2);
    assert_eq!(chain.blocks[1].header.height, 1);

    let alice = chain.ledger.account("alice").expect("alice should exist");
    let bob = chain.ledger.account("bob").expect("bob should exist");
    assert_eq!(alice.balance, 90);
    assert_eq!(bob.balance, 110);
    assert_eq!(alice.nonce, 1);
    assert_eq!(chain.ledger.total_supply(), 200);
    assert_eq!(
        chain.ledger.state_commitment().expect("state commitment"),
        chain.blocks[1].header.state_commitment,
    );
}

#[test]
fn accepted_block_increases_block_count() {
    let mut chain = chain_with_accounts_and_genesis();
    assert_eq!(chain.blocks.len(), 1);

    let block =
        valid_block_with_transactions(&chain, vec![Transaction::new("alice", "bob", 10, 0)]);
    chain.append_block(block).expect("block should be valid");

    assert_eq!(chain.blocks.len(), 2);
}

#[test]
fn accepted_block_updates_ledger() {
    let mut chain = chain_with_accounts_and_genesis();

    let alice_before = chain
        .ledger
        .account("alice")
        .expect("alice should exist")
        .clone();
    let bob_before = chain
        .ledger
        .account("bob")
        .expect("bob should exist")
        .clone();
    let total_supply_before = chain.ledger.total_supply();

    let block =
        valid_block_with_transactions(&chain, vec![Transaction::new("alice", "bob", 10, 0)]);
    chain.append_block(block).expect("block should be valid");

    let alice = chain.ledger.account("alice").expect("alice should exist");
    let bob = chain.ledger.account("bob").expect("bob should exist");

    assert_eq!(alice.balance, alice_before.balance - 10);
    assert_eq!(bob.balance, bob_before.balance + 10);
    assert_eq!(alice.nonce, alice_before.nonce + 1);
    assert_eq!(chain.ledger.total_supply(), total_supply_before);
}
