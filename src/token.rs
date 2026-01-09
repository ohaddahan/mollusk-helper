use solana_account::Account;
use solana_instruction::Instruction;
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use spl_token::state::{Account as TokenAccount, AccountState, Mint};

pub const TOKEN_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub const ASSOCIATED_TOKEN_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

pub const NATIVE_MINT: Pubkey =
    solana_pubkey::pubkey!("So11111111111111111111111111111111111111112");

pub fn create_mint_account(mint_authority: &Pubkey, decimals: u8) -> Account {
    let mint = Mint {
        mint_authority: solana_program_option::COption::Some(*mint_authority),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: solana_program_option::COption::None,
    };

    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint, &mut data).unwrap();

    Account {
        lamports: 1_000_000_000,
        data,
        owner: TOKEN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn create_token_account(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Account {
    let token_account = TokenAccount {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: solana_program_option::COption::None,
        state: AccountState::Initialized,
        is_native: solana_program_option::COption::None,
        delegated_amount: 0,
        close_authority: solana_program_option::COption::None,
    };

    let mut data = vec![0u8; TokenAccount::LEN];
    TokenAccount::pack(token_account, &mut data).unwrap();

    Account {
        lamports: 1_000_000_000,
        data,
        owner: TOKEN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn create_native_token_account(owner: &Pubkey, lamports: u64) -> Account {
    let token_account = TokenAccount {
        mint: NATIVE_MINT,
        owner: *owner,
        amount: lamports,
        delegate: solana_program_option::COption::None,
        state: AccountState::Initialized,
        is_native: solana_program_option::COption::Some(0),
        delegated_amount: 0,
        close_authority: solana_program_option::COption::None,
    };

    let mut data = vec![0u8; TokenAccount::LEN];
    TokenAccount::pack(token_account, &mut data).unwrap();

    Account {
        lamports,
        data,
        owner: TOKEN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn mint_to_instruction(
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    spl_token::instruction::mint_to(&TOKEN_PROGRAM_ID, mint, destination, authority, &[], amount)
        .unwrap()
}

pub fn transfer_instruction(
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    spl_token::instruction::transfer(
        &TOKEN_PROGRAM_ID,
        source,
        destination,
        authority,
        &[],
        amount,
    )
    .unwrap()
}

pub fn sync_native_instruction(token_account: &Pubkey) -> Instruction {
    spl_token::instruction::sync_native(&TOKEN_PROGRAM_ID, token_account).unwrap()
}

pub fn create_associated_token_account_instruction(
    payer: &Pubkey,
    wallet: &Pubkey,
    mint: &Pubkey,
) -> Instruction {
    spl_associated_token_account::instruction::create_associated_token_account_idempotent(
        payer,
        wallet,
        mint,
        &TOKEN_PROGRAM_ID,
    )
}

pub fn get_associated_token_address(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
    spl_associated_token_account::get_associated_token_address(wallet, mint)
}
