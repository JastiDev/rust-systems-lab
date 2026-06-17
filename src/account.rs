#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub balance: u64,
    pub nonce: u64,
}

impl Account {
    pub fn new(balance: u64) -> Self {
        Self { balance, nonce: 0 }
    }
}
