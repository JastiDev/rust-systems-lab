use rust_systems_lab::{
    Ledger, LedgerError, StateTransition, Transaction, TransactionValidationError, Validate,
};

// helper function to avoid duplicated codes

fn ledger_setup() -> (Ledger, String, String) {
    let mut ledger = Ledger::new();
    let id0 = "alice".to_string();
    let id1 = "bob".to_string();
    assert_eq!(ledger.create_account(&id0, 100), Ok(()));
    assert_eq!(ledger.create_account(&id1, 100), Ok(()));
    (ledger, id0, id1)
}

#[test]
fn creates_account() {
    let (ledger, id0, _) = ledger_setup();
    let account = ledger
        .account(&id0)
        .expect("Account should exist in the ledger");
    assert_eq!(account.balance, 100);
    assert_eq!(account.nonce, 0);
}

#[test]
fn rejects_duplicate_account() {
    let (mut ledger, id0, _) = ledger_setup();
    let original_ledger = ledger.clone();
    assert_eq!(
        ledger.create_account(&id0, 100),
        Err(LedgerError::AccountAlreadyExists(id0)),
    );
    assert_eq!(original_ledger, ledger);
}

// helper to avoid duplication
fn preserves_balance_after_error(ledger: Ledger, id0: impl Into<String>, id1: impl Into<String>) {
    let alice = ledger.account(id0).expect("Alice account should exist.");
    let bob = ledger.account(id1).expect("Bob account should exist.");

    assert_eq!(alice.balance, 100);
    assert_eq!(bob.balance, 100);
    assert_eq!(alice.nonce, 0);
}

#[test]
fn applies_valid_transfer() {
    let (mut ledger, id0, id1) = ledger_setup();
    let transaction = Transaction {
        sender: id0.clone(),
        receiver: id1.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(ledger.apply_transaction(transaction), Ok(()));

    let alice = ledger.account(id0).expect("Alice account should exist.");
    let bob = ledger.account(id1).expect("Bob account should exist.");

    assert_eq!(alice.balance, 90);
    assert_eq!(bob.balance, 110);
    assert_eq!(alice.nonce, 1);
}

#[test]
fn rejects_zero_amount_during_validation() {
    let transaction = Transaction {
        sender: "alice".to_string(),
        receiver: "bob".to_string(),
        amount: 0,
        nonce: 0,
    };

    assert!(matches!(
        transaction.validate(),
        Err(TransactionValidationError::ZeroAmount),
    ));
}

#[test]
fn rejects_same_sender_and_receiver_during_validation() {
    let transaction = Transaction {
        sender: "alice".to_string(),
        receiver: "alice".to_string(),
        amount: 10,
        nonce: 0,
    };

    assert!(matches!(
        transaction.validate(),
        Err(TransactionValidationError::SelfTransfer),
    ));
}

#[test]
fn accepts_structurally_valid_transfer() {
    let transaction = Transaction {
        sender: "alice".to_string(),
        receiver: "bob".to_string(),
        amount: 10,
        nonce: 0,
    };

    assert!(transaction.validate().is_ok());
}

#[test]
fn applies_transaction_through_state_transition_trait() {
    let (mut ledger, id0, id1) = ledger_setup();
    let transaction = Transaction {
        sender: id0.clone(),
        receiver: id1.clone(),
        amount: 10,
        nonce: 0,
    };

    assert_eq!(StateTransition::apply(&mut ledger, transaction), Ok(()));

    let alice = ledger.account(&id0).expect("Alice account should exist.");
    let bob = ledger.account(&id1).expect("Bob account should exist.");

    assert_eq!(alice.balance, 90);
    assert_eq!(bob.balance, 110);
    assert_eq!(alice.nonce, 1);
}

#[test]
fn failed_state_transition_does_not_mutate_ledger() {
    let (mut ledger, id0, id1) = ledger_setup();
    let original_ledger = ledger.clone();

    let transaction = Transaction {
        sender: id0,
        receiver: id1,
        amount: 500,
        nonce: 0,
    };

    assert_eq!(
        ledger.apply(transaction),
        Err(LedgerError::InsufficientBalance {
            available: 100,
            requested: 500,
        }),
    );
    assert_eq!(original_ledger, ledger);
}

#[test]
fn rejects_unknown_sender() {
    let (mut ledger, id0, id1) = ledger_setup();

    let id3 = String::from("James");
    let transaction = Transaction {
        sender: id3.clone(),
        receiver: id1.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(
        ledger.apply_transaction(transaction),
        Err(LedgerError::SenderNotFound(id3))
    );

    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn rejects_unknown_receiver() {
    let (mut ledger, id0, id1) = ledger_setup();

    let id3 = String::from("James");
    let transaction = Transaction {
        sender: id0.clone(),
        receiver: id3.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(
        ledger.apply_transaction(transaction),
        Err(LedgerError::ReceiverNotFound(id3))
    );

    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn rejects_insufficient_balance() {
    let (mut ledger, id0, id1) = ledger_setup();

    let transaction = Transaction {
        sender: id0.clone(),
        receiver: id1.clone(),
        amount: 500,
        nonce: 0,
    };
    assert_eq!(
        ledger.apply_transaction(transaction),
        Err(LedgerError::InsufficientBalance {
            available: 100,
            requested: 500
        })
    );

    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn rejects_wrong_nonce() {
    let (mut ledger, id0, id1) = ledger_setup();

    let transaction = Transaction {
        sender: id0.clone(),
        receiver: id1.clone(),
        amount: 10,
        nonce: 110,
    };
    assert_eq!(
        ledger.apply_transaction(transaction),
        Err(LedgerError::IncorrectNonce {
            expected: 0,
            received: 110,
        })
    );

    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn preserves_total_supply() {
    let (mut ledger, id0, id1) = ledger_setup();

    let total_original = ledger.total_supply();
    assert_eq!(total_original, 200);

    let transaction = Transaction {
        sender: id0.clone(),
        receiver: id1.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(ledger.apply_transaction(transaction), Ok(()));

    let total_later = ledger.total_supply();
    assert_eq!(total_later, 200);
}
