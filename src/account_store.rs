use mollusk_svm::account_store::AccountStore;
use solana_account::Account;
use solana_address::Address;
use std::collections::HashMap;

#[derive(Default, Clone)]
pub(crate) struct InMemoryAccountStore {
    accounts: HashMap<Address, Account>,
}

impl InMemoryAccountStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }

    pub fn get_balance(&self, address: &Address) -> Option<u64> {
        self.accounts.get(address).map(|a| a.lamports)
    }

    pub fn snapshot(&self) -> HashMap<Address, Account> {
        self.accounts.clone()
    }

    pub fn restore(&mut self, snapshot: HashMap<Address, Account>) {
        self.accounts = snapshot;
    }
}

impl AccountStore for InMemoryAccountStore {
    fn get_account(&self, address: &Address) -> Option<Account> {
        self.accounts.get(address).cloned()
    }

    fn store_account(&mut self, address: Address, account: Account) {
        self.accounts.insert(address, account);
    }
}
