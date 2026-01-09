use solana_instruction::error::InstructionError;
use solana_program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MolluskHelperError {
    #[error("Instruction failed: {0}")]
    InstructionFailed(#[from] InstructionError),

    #[error("Program error: {0}")]
    ProgramError(#[from] ProgramError),

    #[error("Transaction failed at instruction {index}: {error}")]
    TransactionFailed {
        index: usize,
        error: InstructionError,
    },

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Keypair not found: {0}")]
    KeypairNotFound(String),

    #[error("Lock acquisition failed")]
    LockError,
}

pub type Result<T> = std::result::Result<T, MolluskHelperError>;
