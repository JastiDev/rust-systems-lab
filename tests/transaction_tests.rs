use rust_systems_lab::{Ledger, LedgerError, Transaction, TransactionError};

fn sample_transaction() -> Transaction {
    Transaction::new("alice", "bob", 10, 0)
}

fn ledger_setup() -> (Ledger, String, String) {
    let mut ledger = Ledger::new();
    let alice = "alice".to_string();
    let bob = "bob".to_string();
    assert_eq!(ledger.create_account(&alice, 100), Ok(()));
    assert_eq!(ledger.create_account(&bob, 100), Ok(()));
    (ledger, alice, bob)
}

#[test]
fn equal_transactions_produce_equal_bytes() {
    let left = sample_transaction();
    let right = sample_transaction();

    assert_eq!(left.canonical_bytes(), right.canonical_bytes());
}

#[test]
fn equal_transactions_produce_equal_ids() {
    let left = sample_transaction();
    let right = sample_transaction();

    assert_eq!(left.id(), right.id());
}

#[test]
fn changing_sender_changes_transaction_id() {
    let base = sample_transaction();
    let changed = Transaction::new("carol", "bob", 10, 0);

    assert_ne!(base.id(), changed.id());
}

#[test]
fn changing_receiver_changes_transaction_id() {
    let base = sample_transaction();
    let changed = Transaction::new("alice", "carol", 10, 0);

    assert_ne!(base.id(), changed.id());
}

#[test]
fn changing_amount_changes_transaction_id() {
    let base = sample_transaction();
    let changed = Transaction::new("alice", "bob", 20, 0);

    assert_ne!(base.id(), changed.id());
}

#[test]
fn changing_nonce_changes_transaction_id() {
    let base = sample_transaction();
    let changed = Transaction::new("alice", "bob", 10, 1);

    assert_ne!(base.id(), changed.id());
}

#[test]
fn canonical_serialization_is_repeatable() {
    let transaction = sample_transaction();

    assert_eq!(transaction.canonical_bytes(), transaction.canonical_bytes());
}

#[test]
fn deserialization_recreates_transaction() {
    let original = sample_transaction();
    let bytes = original.canonical_bytes();

    let decoded = Transaction::from_canonical_bytes(&bytes).expect("valid bytes should decode");

    assert_eq!(original, decoded);
}

#[test]
fn transaction_id_display_has_64_hex_chars() {
    let id = sample_transaction().id();

    assert_eq!(format!("{id}").len(), 64);
}

#[test]
fn transaction_id_display_contains_only_hex_chars() {
    let id = sample_transaction().id();

    assert!(format!("{id}").chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn invalid_transaction_bytes_return_typed_error() {
    let result = Transaction::from_canonical_bytes(&[0x00, 0x01, 0xff]);

    assert_eq!(result, Err(TransactionError::InvalidEncoding));
}

#[test]
fn failed_transaction_does_not_mutate_ledger() {
    let (mut ledger, alice, bob) = ledger_setup();
    let original_ledger = ledger.clone();

    let transaction = Transaction::new(alice, bob, 500, 0);

    assert_eq!(
        ledger.apply_transaction(transaction),
        Err(LedgerError::InsufficientBalance {
            available: 100,
            requested: 500,
        }),
    );
    assert_eq!(original_ledger, ledger);
}
