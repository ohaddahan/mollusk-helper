use crate::context::MolluskContextHelper;
use crate::error::{MolluskHelperError, Result};
use mollusk_svm::result::InstructionResult;
use solana_instruction::Instruction;

pub struct TransactionResult {
    pub instruction_results: Vec<InstructionResult>,
    pub total_compute_units: u64,
    pub total_execution_time: u64,
}

impl TransactionResult {
    pub fn is_success(&self) -> bool {
        self.instruction_results
            .iter()
            .all(|r| r.program_result.is_ok())
    }

    pub fn failed_at(&self) -> Option<usize> {
        self.instruction_results
            .iter()
            .position(|r| r.program_result.is_err())
    }

    pub fn last_result(&self) -> Option<&InstructionResult> {
        self.instruction_results.last()
    }
}

pub struct TransactionBuilder<'a> {
    context: &'a MolluskContextHelper,
    instructions: Vec<Instruction>,
}

impl<'a> TransactionBuilder<'a> {
    pub(crate) fn new(context: &'a MolluskContextHelper) -> Self {
        Self {
            context,
            instructions: Vec::new(),
        }
    }

    pub fn add_instruction(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    pub fn add_instructions(mut self, instructions: impl IntoIterator<Item = Instruction>) -> Self {
        self.instructions.extend(instructions);
        self
    }

    pub fn execute(self) -> Result<TransactionResult> {
        if self.instructions.is_empty() {
            return Ok(TransactionResult {
                instruction_results: vec![],
                total_compute_units: 0,
                total_execution_time: 0,
            });
        }

        let snapshot = self.context.snapshot_accounts();

        let mut instruction_results = Vec::with_capacity(self.instructions.len());
        let mut total_compute_units = 0u64;
        let mut total_execution_time = 0u64;

        for (index, instruction) in self.instructions.iter().enumerate() {
            let result = self.context.process_instruction_internal(instruction);

            total_compute_units += result.compute_units_consumed;
            total_execution_time += result.execution_time;

            if result.program_result.is_err() {
                self.context.restore_accounts(snapshot);

                let error = match &result.raw_result {
                    Err(e) => e.clone(),
                    Ok(_) => unreachable!(),
                };

                return Err(MolluskHelperError::TransactionFailed { index, error });
            }

            instruction_results.push(result);
        }

        Ok(TransactionResult {
            instruction_results,
            total_compute_units,
            total_execution_time,
        })
    }

    pub fn execute_allow_failures(self) -> TransactionResult {
        if self.instructions.is_empty() {
            return TransactionResult {
                instruction_results: vec![],
                total_compute_units: 0,
                total_execution_time: 0,
            };
        }

        let snapshot = self.context.snapshot_accounts();

        let mut instruction_results = Vec::with_capacity(self.instructions.len());
        let mut total_compute_units = 0u64;
        let mut total_execution_time = 0u64;
        let mut any_failed = false;

        for instruction in &self.instructions {
            let result = self.context.process_instruction_internal(instruction);

            total_compute_units += result.compute_units_consumed;
            total_execution_time += result.execution_time;

            if result.program_result.is_err() {
                any_failed = true;
            }

            instruction_results.push(result);
        }

        if any_failed {
            self.context.restore_accounts(snapshot);
        }

        TransactionResult {
            instruction_results,
            total_compute_units,
            total_execution_time,
        }
    }

    pub fn dry_run(self) -> TransactionResult {
        if self.instructions.is_empty() {
            return TransactionResult {
                instruction_results: vec![],
                total_compute_units: 0,
                total_execution_time: 0,
            };
        }

        let snapshot = self.context.snapshot_accounts();

        let mut instruction_results = Vec::with_capacity(self.instructions.len());
        let mut total_compute_units = 0u64;
        let mut total_execution_time = 0u64;

        for instruction in &self.instructions {
            let result = self.context.process_instruction_internal(instruction);

            total_compute_units += result.compute_units_consumed;
            total_execution_time += result.execution_time;

            let failed = result.program_result.is_err();
            instruction_results.push(result);

            if failed {
                break;
            }
        }

        self.context.restore_accounts(snapshot);

        TransactionResult {
            instruction_results,
            total_compute_units,
            total_execution_time,
        }
    }
}
