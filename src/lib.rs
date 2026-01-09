mod account;
mod account_store;
mod context;
mod error;
mod token;
mod transaction;

pub use context::{
    MolluskContextHelper, ProgramLoader, ADDRESS_LOOKUP_TABLE_PROGRAM_ID,
    COMPUTE_BUDGET_PROGRAM_ID, MEMO_PROGRAM_ID, MEMO_V1_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
};
pub use error::{MolluskHelperError, Result};
pub use transaction::{TransactionBuilder, TransactionResult};

pub mod prelude {
    pub use crate::context::{
        MolluskContextHelper, ProgramLoader, ADDRESS_LOOKUP_TABLE_PROGRAM_ID,
        COMPUTE_BUDGET_PROGRAM_ID, MEMO_PROGRAM_ID, MEMO_V1_PROGRAM_ID, TOKEN_2022_PROGRAM_ID,
    };
    pub use crate::error::{MolluskHelperError, Result};
    pub use crate::transaction::{TransactionBuilder, TransactionResult};

    pub use mollusk_svm::result::{Check, InstructionResult, ProgramResult};
    pub use solana_account::Account;
    pub use solana_instruction::{AccountMeta, Instruction};
    pub use solana_keypair::Keypair;
    pub use solana_pubkey::Pubkey;
    pub use solana_signer::Signer;
}
