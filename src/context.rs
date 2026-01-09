use crate::account;
use crate::account_store::InMemoryAccountStore;
use crate::error::{MolluskHelperError, Result};
use crate::token;
use crate::transaction::TransactionBuilder;
use mollusk_svm::account_store::AccountStore;
use mollusk_svm::result::{InstructionResult, ProgramResult};
use mollusk_svm::{Mollusk, MolluskContext};
use solana_account::Account;
use solana_address::Address;
use solana_hash::Hash;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::{v0, AddressLookupTableAccount, VersionedMessage};
use solana_program_pack::Pack;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use spl_token::state::Account as TokenAccount;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub const MEMO_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr");

pub const MEMO_V1_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo");

pub const TOKEN_2022_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");

pub const ADDRESS_LOOKUP_TABLE_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("AddressLookupTab1e1111111111111111111111111");

pub const COMPUTE_BUDGET_PROGRAM_ID: Pubkey =
    solana_pubkey::pubkey!("ComputeBudget111111111111111111111111111111");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramLoader {
    V2,
    V3,
}

fn add_default_programs(mollusk: &mut Mollusk) {
    mollusk_svm_programs_token::token::add_program(mollusk);
    mollusk_svm_programs_token::associated_token::add_program(mollusk);
    mollusk_svm_programs_token::token2022::add_program(mollusk);
    mollusk_svm_programs_memo::memo::add_program(mollusk);
    mollusk_svm_programs_memo::memo_v1::add_program(mollusk);
}

pub struct MolluskContextHelper {
    pub(crate) context: MolluskContext<InMemoryAccountStore>,
    keypairs: Arc<RwLock<HashMap<String, Keypair>>>,
}

impl MolluskContextHelper {
    pub fn new(program_id: &Pubkey, elf_bytes: &[u8]) -> Self {
        Self::new_with_options(
            program_id,
            elf_bytes,
            ProgramLoader::V3,
            Self::current_unix_timestamp(),
        )
    }

    pub fn new_with_timestamp(program_id: &Pubkey, elf_bytes: &[u8], unix_timestamp: u64) -> Self {
        Self::new_with_options(program_id, elf_bytes, ProgramLoader::V3, unix_timestamp)
    }

    pub fn new_with_loader(program_id: &Pubkey, elf_bytes: &[u8], loader: ProgramLoader) -> Self {
        Self::new_with_options(
            program_id,
            elf_bytes,
            loader,
            Self::current_unix_timestamp(),
        )
    }

    pub fn new_with_options(
        program_id: &Pubkey,
        elf_bytes: &[u8],
        loader: ProgramLoader,
        unix_timestamp: u64,
    ) -> Self {
        let mut mollusk = Mollusk::default();

        let loader_key = match loader {
            ProgramLoader::V2 => &mollusk_svm::program::loader_keys::LOADER_V2,
            ProgramLoader::V3 => &mollusk_svm::program::loader_keys::LOADER_V3,
        };

        mollusk.add_program_with_loader_and_elf(
            &Self::pubkey_to_address(program_id),
            loader_key,
            elf_bytes,
        );

        add_default_programs(&mut mollusk);
        mollusk.sysvars.clock.unix_timestamp = unix_timestamp as i64;

        let store = InMemoryAccountStore::new();
        let context = mollusk.with_context(store);

        Self {
            context,
            keypairs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_without_program() -> Self {
        Self::new_without_program_with_timestamp(Self::current_unix_timestamp())
    }

    pub fn new_without_program_with_timestamp(unix_timestamp: u64) -> Self {
        let mut mollusk = Mollusk::default();

        add_default_programs(&mut mollusk);
        mollusk.sysvars.clock.unix_timestamp = unix_timestamp as i64;

        let store = InMemoryAccountStore::new();
        let context = mollusk.with_context(store);

        Self {
            context,
            keypairs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add_program(&mut self, program_id: &Pubkey, elf_bytes: &[u8]) {
        self.add_program_with_loader(program_id, elf_bytes, ProgramLoader::V3);
    }

    pub fn add_program_with_loader(
        &mut self,
        program_id: &Pubkey,
        elf_bytes: &[u8],
        loader: ProgramLoader,
    ) {
        let loader_key = match loader {
            ProgramLoader::V2 => &mollusk_svm::program::loader_keys::LOADER_V2,
            ProgramLoader::V3 => &mollusk_svm::program::loader_keys::LOADER_V3,
        };

        self.context.mollusk.add_program_with_loader_and_elf(
            &Self::pubkey_to_address(program_id),
            loader_key,
            elf_bytes,
        );
    }

    pub fn current_unix_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
    }

    pub fn process_instruction(&self, instruction: &Instruction) -> Result<InstructionResult> {
        let result = self.context.process_instruction(instruction);
        match &result.program_result {
            ProgramResult::Success => Ok(result),
            ProgramResult::Failure(e) => Err(MolluskHelperError::ProgramError(e.clone())),
            ProgramResult::UnknownError(e) => Err(MolluskHelperError::InstructionFailed(e.clone())),
        }
    }

    pub fn process_instruction_unchecked(&self, instruction: &Instruction) -> InstructionResult {
        self.context.process_instruction(instruction)
    }

    pub(crate) fn process_instruction_internal(
        &self,
        instruction: &Instruction,
    ) -> InstructionResult {
        self.context.process_instruction(instruction)
    }

    pub fn transaction(&self) -> TransactionBuilder<'_> {
        TransactionBuilder::new(self)
    }

    pub(crate) fn snapshot_accounts(&self) -> HashMap<Address, Account> {
        self.context.account_store.borrow().snapshot()
    }

    pub(crate) fn restore_accounts(&self, snapshot: HashMap<Address, Account>) {
        self.context.account_store.borrow_mut().restore(snapshot);
    }

    pub fn add_account(&self, pubkey: &Pubkey, account: Account) {
        let address = Self::pubkey_to_address(pubkey);
        self.context
            .account_store
            .borrow_mut()
            .add_account(address, account);
    }

    pub fn get_account(&self, pubkey: &Pubkey) -> Option<Account> {
        let address = Self::pubkey_to_address(pubkey);
        self.context.account_store.borrow().get_account(&address)
    }

    pub fn get_balance(&self, pubkey: &Pubkey) -> Option<u64> {
        let address = Self::pubkey_to_address(pubkey);
        self.context.account_store.borrow().get_balance(&address)
    }

    pub fn fund_account(&self, pubkey: &Pubkey, lamports: u64) {
        let account = account::system_account_with_lamports(lamports);
        self.add_account(pubkey, account);
    }

    pub fn store_keypair(&self, name: &str, keypair: Keypair) -> Result<()> {
        self.keypairs
            .write()
            .map_err(|_| MolluskHelperError::LockError)?
            .insert(name.to_string(), keypair);
        Ok(())
    }

    pub fn sign_with(&self, name: &str, message: &[u8]) -> Result<[u8; 64]> {
        let keypairs = self
            .keypairs
            .read()
            .map_err(|_| MolluskHelperError::LockError)?;
        let keypair = keypairs
            .get(name)
            .ok_or_else(|| MolluskHelperError::KeypairNotFound(name.to_string()))?;
        Ok(keypair.sign_message(message).into())
    }

    pub fn get_keypair_pubkey(&self, name: &str) -> Result<Pubkey> {
        let keypairs = self
            .keypairs
            .read()
            .map_err(|_| MolluskHelperError::LockError)?;
        let keypair = keypairs
            .get(name)
            .ok_or_else(|| MolluskHelperError::KeypairNotFound(name.to_string()))?;
        Ok(keypair.pubkey())
    }

    pub fn update_unix_timestamp(&mut self, timestamp: i64) {
        self.context.mollusk.sysvars.clock.unix_timestamp = timestamp;
    }

    pub fn get_unix_timestamp(&self) -> i64 {
        self.context.mollusk.sysvars.clock.unix_timestamp
    }

    pub fn warp_to_slot(&mut self, slot: u64) {
        self.context.mollusk.warp_to_slot(slot);
    }

    pub fn create_mint(&self, mint_pubkey: &Pubkey, authority: &Pubkey, decimals: u8) {
        let account = token::create_mint_account(authority, decimals);
        self.add_account(mint_pubkey, account);
    }

    pub fn create_token_account(
        &self,
        token_account_pubkey: &Pubkey,
        mint: &Pubkey,
        owner: &Pubkey,
        amount: u64,
    ) {
        let account = token::create_token_account(mint, owner, amount);
        self.add_account(token_account_pubkey, account);
    }

    pub fn create_native_token_account(
        &self,
        token_account_pubkey: &Pubkey,
        owner: &Pubkey,
        lamports: u64,
    ) {
        let account = token::create_native_token_account(owner, lamports);
        self.add_account(token_account_pubkey, account);
    }

    pub fn get_token_balance(&self, token_account_pubkey: &Pubkey) -> Result<u64> {
        let account = self
            .get_account(token_account_pubkey)
            .ok_or_else(|| MolluskHelperError::AccountNotFound(token_account_pubkey.to_string()))?;
        let token_account =
            TokenAccount::unpack(&account.data).map_err(MolluskHelperError::ProgramError)?;
        Ok(token_account.amount)
    }

    pub fn mint_to(
        &self,
        mint: &Pubkey,
        destination: &Pubkey,
        authority: &Pubkey,
        amount: u64,
    ) -> Result<InstructionResult> {
        let ix = token::mint_to_instruction(mint, destination, authority, amount);
        self.process_instruction(&ix)
    }

    pub fn transfer_tokens(
        &self,
        source: &Pubkey,
        destination: &Pubkey,
        authority: &Pubkey,
        amount: u64,
    ) -> Result<InstructionResult> {
        let ix = token::transfer_instruction(source, destination, authority, amount);
        self.process_instruction(&ix)
    }

    pub fn sync_native(&self, token_account: &Pubkey) -> Result<InstructionResult> {
        let ix = token::sync_native_instruction(token_account);
        self.process_instruction(&ix)
    }

    pub fn transfer_sol(
        &self,
        from: &Pubkey,
        to: &Pubkey,
        lamports: u64,
    ) -> Result<InstructionResult> {
        let ix = solana_system_interface::instruction::transfer(from, to, lamports);
        self.process_instruction(&ix)
    }

    pub fn get_associated_token_address(&self, wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
        token::get_associated_token_address(wallet, mint)
    }

    pub fn create_lookup_table_account(
        key: Pubkey,
        addresses: Vec<Pubkey>,
    ) -> AddressLookupTableAccount {
        AddressLookupTableAccount { key, addresses }
    }

    pub fn build_versioned_tx_size(
        payer: &Pubkey,
        instructions: &[Instruction],
        lookup_tables: &[AddressLookupTableAccount],
    ) -> (usize, usize, i64) {
        let recent_blockhash = Hash::default();
        let message =
            v0::Message::try_compile(payer, instructions, lookup_tables, recent_blockhash)
                .expect("Failed to compile v0 message");
        let versioned_message = VersionedMessage::V0(message);

        let num_signers = instructions
            .iter()
            .flat_map(|ix| ix.accounts.iter())
            .filter(|a| a.is_signer)
            .map(|a| a.pubkey)
            .collect::<std::collections::HashSet<_>>()
            .len()
            .max(1);

        let message_bytes =
            bincode::serialize(&versioned_message).expect("Failed to serialize message");
        let tx_size = 1 + (num_signers * 64) + message_bytes.len();
        let tx_limit: usize = 1232;
        let remaining = tx_limit as i64 - tx_size as i64;

        (tx_size, tx_limit, remaining)
    }

    pub fn pubkey_to_address(pubkey: &Pubkey) -> Address {
        Address::new_from_array(pubkey.to_bytes())
    }

    pub fn address_to_pubkey(address: &Address) -> Pubkey {
        Pubkey::from(address.to_bytes())
    }

    pub fn system_program() -> Pubkey {
        solana_pubkey::pubkey!("11111111111111111111111111111111")
    }

    pub fn token_program() -> Pubkey {
        token::TOKEN_PROGRAM_ID
    }

    pub fn token_2022_program() -> Pubkey {
        TOKEN_2022_PROGRAM_ID
    }

    pub fn associated_token_program() -> Pubkey {
        token::ASSOCIATED_TOKEN_PROGRAM_ID
    }

    pub fn memo_program() -> Pubkey {
        MEMO_PROGRAM_ID
    }

    pub fn memo_v1_program() -> Pubkey {
        MEMO_V1_PROGRAM_ID
    }

    pub fn address_lookup_table_program() -> Pubkey {
        ADDRESS_LOOKUP_TABLE_PROGRAM_ID
    }

    pub fn compute_budget_program() -> Pubkey {
        COMPUTE_BUDGET_PROGRAM_ID
    }

    pub fn native_mint() -> Pubkey {
        token::NATIVE_MINT
    }

    pub fn rent_sysvar() -> Pubkey {
        solana_pubkey::pubkey!("SysvarRent111111111111111111111111111111111")
    }

    pub fn add_program_account(&self, pubkey: &Pubkey, owner: &Pubkey, data: Vec<u8>) {
        let account = account::program_account(owner, data);
        self.add_account(pubkey, account);
    }

    pub fn create_associated_token_account_instruction(
        payer: &Pubkey,
        wallet: &Pubkey,
        mint: &Pubkey,
    ) -> Instruction {
        token::create_associated_token_account_instruction(payer, wallet, mint)
    }
}
