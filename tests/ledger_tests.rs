use rust_systems_lab::Ledger;
use rust_systems_lab::LedgerError;
use rust_systems_lab::Transfer;

#[test]
fn creates_account() {
    let mut ledger = Ledger::new();
    let id = "alice".to_string();
    assert_eq!(ledger.create_account(&id, 100), Ok(()));
}

#[test]
fn rejects_duplicate_account() {}

#[test]
fn applies_valid_transfer() {}

#[test]
fn rejects_unknown_sender() {}

#[test]
fn rejects_unknown_receiver() {}

#[test]
fn rejects_insufficient_balance() {}

#[test]
fn rejects_wrong_nonce() {}

#[test]
fn preserves_total_supply() {}
