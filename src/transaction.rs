#[derive(Debug)]
pub struct Account {
    pub owner: String,
    pub balance: u64,
}

#[derive(Debug)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

pub fn has_enough_balance(account: &Account, amount: u64) -> bool {
    account.balance >= amount
}
