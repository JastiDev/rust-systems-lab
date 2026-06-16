mod transaction;

use transaction::{Account, Transaction};

use crate::transaction::has_enough_balance;

fn main() {
    let alice = Account {
        owner: String::from("Alice"),
        balance: 100,
    };

    let bob = Account {
        owner: String::from("Bob"),
        balance: 50,
    };

    let tx = Transaction {
        from: String::from("Alice"),
        to: String::from("Bob"),
        amount: 25,
    };

    println!("Alice account: {:?}", alice);
    println!("Bob Account: {:?}", bob);
    println!("Transaction: {:?}", tx);

    let can_send = has_enough_balance(&alice, tx.amount);

    println!("Can Alice send this transaction? {}", can_send);
}
