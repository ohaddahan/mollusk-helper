use solana_account::Account;
use solana_pubkey::Pubkey;

pub const SYSTEM_PROGRAM_ID: Pubkey = solana_pubkey::pubkey!("11111111111111111111111111111111");

pub fn system_account_with_lamports(lamports: u64) -> Account {
    Account::new(lamports, 0, &SYSTEM_PROGRAM_ID)
}

pub fn program_account(owner: &Pubkey, data: Vec<u8>) -> Account {
    Account {
        lamports: 1_000_000_000,
        data,
        owner: *owner,
        executable: false,
        rent_epoch: 0,
    }
}
