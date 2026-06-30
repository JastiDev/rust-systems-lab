use rust_systems_lab::Ledger;

fn ledger_with_insertion_order(order: &[(&str, u64)]) -> Ledger {
    let mut ledger = Ledger::new();
    for (id, balance) in order {
        ledger
            .create_account(*id, *balance)
            .expect("account should be created");
    }
    ledger
}

fn sample_ledger() -> Ledger {
    ledger_with_insertion_order(&[("alice", 100), ("bob", 100)])
}

#[test]
fn same_state_with_different_insertion_order_has_same_commitment() {
    let forward = ledger_with_insertion_order(&[("alice", 100), ("bob", 100), ("carol", 50)]);
    let reverse = ledger_with_insertion_order(&[("carol", 50), ("bob", 100), ("alice", 100)]);

    assert_eq!(
        forward
            .state_commitment()
            .expect("commitment should succeed"),
        reverse
            .state_commitment()
            .expect("commitment should succeed"),
    );
}

#[test]
fn changing_balance_changes_state_commitment() {
    let mut ledger = sample_ledger();
    let before = ledger
        .state_commitment()
        .expect("commitment should succeed");

    ledger
        .accounts
        .get_mut("alice")
        .expect("alice should exist")
        .balance = 90;

    let after = ledger
        .state_commitment()
        .expect("commitment should succeed");

    assert_ne!(before, after);
}

#[test]
fn changing_nonce_changes_state_commitment() {
    let mut ledger = sample_ledger();
    let before = ledger
        .state_commitment()
        .expect("commitment should succeed");

    ledger
        .accounts
        .get_mut("alice")
        .expect("alice should exist")
        .nonce = 1;

    let after = ledger
        .state_commitment()
        .expect("commitment should succeed");

    assert_ne!(before, after);
}

#[test]
fn adding_account_changes_state_commitment() {
    let mut ledger = sample_ledger();
    let before = ledger
        .state_commitment()
        .expect("commitment should succeed");

    ledger
        .create_account("carol", 50)
        .expect("account should be created");

    let after = ledger
        .state_commitment()
        .expect("commitment should succeed");

    assert_ne!(before, after);
}

#[test]
fn state_commitment_is_repeatable() {
    let ledger = sample_ledger();

    let first = ledger
        .state_commitment()
        .expect("commitment should succeed");
    let second = ledger
        .state_commitment()
        .expect("commitment should succeed");

    assert_eq!(first, second);
}
