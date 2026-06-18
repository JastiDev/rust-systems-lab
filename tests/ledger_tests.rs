use rust_systems_lab::Ledger;
use rust_systems_lab::LedgerError;
use rust_systems_lab::Transfer;

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
    let transfer = Transfer {
        from: id0.clone(),
        to: id1.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(ledger.apply_transfer(&transfer), Ok(()));

    let alice = ledger.account(id0).expect("Alice account should exist.");
    let bob = ledger.account(id1).expect("Bob account should exist.");

    assert_eq!(alice.balance, 90);
    assert_eq!(bob.balance, 110);
    assert_eq!(alice.nonce, 1);
}

#[test]
fn rejects_unknown_sender() {
    let (mut ledger, id0, id1) = ledger_setup();
    let original_ledger = ledger.clone();

    let id3 = String::from("James");
    let transfer = Transfer {
        from: id3.clone(),
        to: id1.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(
        ledger.apply_transfer(&transfer),
        Err(LedgerError::SenderNotFound(id3))
    );

    assert_eq!(original_ledger, ledger);
    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn rejects_unknown_receiver() {
    let (mut ledger, id0, id1) = ledger_setup();
    let original_ledger = ledger.clone();

    let id3 = String::from("James");
    let transfer = Transfer {
        from: id0.clone(),
        to: id3.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(
        ledger.apply_transfer(&transfer),
        Err(LedgerError::ReceiverNotFound(id3))
    );

    assert_eq!(original_ledger, ledger);
    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn rejects_insufficient_balance() {
    let (mut ledger, id0, id1) = ledger_setup();
    let original_ledger = ledger.clone();

    let transfer = Transfer {
        from: id0.clone(),
        to: id1.clone(),
        amount: 500,
        nonce: 0,
    };
    assert_eq!(
        ledger.apply_transfer(&transfer),
        Err(LedgerError::InsufficientBalance {
            available: 100,
            requested: 500
        })
    );

    assert_eq!(original_ledger, ledger);
    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn rejects_wrong_nonce() {
    let (mut ledger, id0, id1) = ledger_setup();
    let original_ledger = ledger.clone();

    let transfer = Transfer {
        from: id0.clone(),
        to: id1.clone(),
        amount: 10,
        nonce: 110,
    };
    assert_eq!(
        ledger.apply_transfer(&transfer),
        Err(LedgerError::IncorrectNonce {
            expected: 0,
            received: 110,
        })
    );

    assert_eq!(original_ledger, ledger);
    preserves_balance_after_error(ledger, id0, id1);
}

#[test]
fn preserves_total_supply() {
    let (mut ledger, id0, id1) = ledger_setup();

    let total_original = ledger.total_supply();
    assert_eq!(total_original, 200);

    let transfer = Transfer {
        from: id0.clone(),
        to: id1.clone(),
        amount: 10,
        nonce: 0,
    };
    assert_eq!(ledger.apply_transfer(&transfer), Ok(()));

    let total_later = ledger.total_supply();
    assert_eq!(total_later, 200);
}
