use mollusk_helper::prelude::*;

#[test]
fn test_constructor_variants() {
    let _ctx1 = MolluskContextHelper::new_without_program();
    let _ctx2 = MolluskContextHelper::new_without_program_with_timestamp(1000);

    let _loader_v2 = ProgramLoader::V2;
    let _loader_v3 = ProgramLoader::V3;
}

#[test]
fn test_program_id_constants() {
    assert_eq!(MolluskContextHelper::memo_program(), MEMO_PROGRAM_ID);
    assert_eq!(MolluskContextHelper::memo_v1_program(), MEMO_V1_PROGRAM_ID);
    assert_eq!(
        MolluskContextHelper::token_2022_program(),
        TOKEN_2022_PROGRAM_ID
    );
    assert_eq!(
        MolluskContextHelper::address_lookup_table_program(),
        ADDRESS_LOOKUP_TABLE_PROGRAM_ID
    );
    assert_eq!(
        MolluskContextHelper::compute_budget_program(),
        COMPUTE_BUDGET_PROGRAM_ID
    );
}

#[test]
fn test_fund_and_transfer_sol() {
    let ctx = MolluskContextHelper::new_without_program();

    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();

    ctx.fund_account(&alice, 1_000_000);
    ctx.fund_account(&bob, 0);

    assert_eq!(ctx.get_balance(&alice), Some(1_000_000));
    assert_eq!(ctx.get_balance(&bob), Some(0));

    let result = ctx.transfer_sol(&alice, &bob, 500_000);
    assert!(result.is_ok());

    assert_eq!(ctx.get_balance(&alice), Some(500_000));
    assert_eq!(ctx.get_balance(&bob), Some(500_000));
}

#[test]
fn test_atomic_transaction_success() {
    let ctx = MolluskContextHelper::new_without_program();

    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();
    let charlie = Pubkey::new_unique();

    ctx.fund_account(&alice, 1_000_000);
    ctx.fund_account(&bob, 1_000_000);
    ctx.fund_account(&charlie, 0);

    let ix1 = solana_system_interface::instruction::transfer(&alice, &bob, 100_000);
    let ix2 = solana_system_interface::instruction::transfer(&bob, &charlie, 50_000);

    let result = ctx
        .transaction()
        .add_instruction(ix1)
        .add_instruction(ix2)
        .execute();

    assert!(result.is_ok());

    assert_eq!(ctx.get_balance(&alice), Some(900_000));
    assert_eq!(ctx.get_balance(&bob), Some(1_050_000));
    assert_eq!(ctx.get_balance(&charlie), Some(50_000));
}

#[test]
fn test_atomic_transaction_rollback() {
    let ctx = MolluskContextHelper::new_without_program();

    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();

    ctx.fund_account(&alice, 100_000);
    ctx.fund_account(&bob, 100_000);

    let ix1 = solana_system_interface::instruction::transfer(&alice, &bob, 50_000);
    let ix2 = solana_system_interface::instruction::transfer(&alice, &bob, 100_000); // Will fail

    let result = ctx
        .transaction()
        .add_instruction(ix1)
        .add_instruction(ix2)
        .execute();

    assert!(result.is_err());

    assert_eq!(ctx.get_balance(&alice), Some(100_000));
    assert_eq!(ctx.get_balance(&bob), Some(100_000));
}

#[test]
fn test_dry_run_no_state_change() {
    let ctx = MolluskContextHelper::new_without_program();

    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();

    ctx.fund_account(&alice, 1_000_000);
    ctx.fund_account(&bob, 0);

    let ix = solana_system_interface::instruction::transfer(&alice, &bob, 500_000);

    let result = ctx.transaction().add_instruction(ix).dry_run();

    assert!(result.is_success());
    assert_eq!(ctx.get_balance(&alice), Some(1_000_000));
    assert_eq!(ctx.get_balance(&bob), Some(0));
}

#[test]
fn test_token_operations() {
    let ctx = MolluskContextHelper::new_without_program();

    let mint = Pubkey::new_unique();
    let authority = Pubkey::new_unique();
    let user = Pubkey::new_unique();
    let user_token_account = Pubkey::new_unique();

    ctx.fund_account(&authority, 1_000_000);
    ctx.create_mint(&mint, &authority, 9);
    ctx.create_token_account(&user_token_account, &mint, &user, 0);

    assert_eq!(ctx.get_token_balance(&user_token_account).unwrap(), 0);

    let result = ctx.mint_to(&mint, &user_token_account, &authority, 1_000_000_000);
    assert!(result.is_ok());

    assert_eq!(
        ctx.get_token_balance(&user_token_account).unwrap(),
        1_000_000_000
    );
}

#[test]
fn test_keypair_storage() {
    let ctx = MolluskContextHelper::new_without_program();

    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();

    ctx.store_keypair("test_key", keypair).unwrap();

    let retrieved_pubkey = ctx.get_keypair_pubkey("test_key").unwrap();
    assert_eq!(pubkey, retrieved_pubkey);

    let message = b"test message";
    let signature = ctx.sign_with("test_key", message).unwrap();
    assert_eq!(signature.len(), 64);
}

#[test]
fn test_transaction_result_helpers() {
    let ctx = MolluskContextHelper::new_without_program();

    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();

    ctx.fund_account(&alice, 1_000_000);
    ctx.fund_account(&bob, 0);

    let ix1 = solana_system_interface::instruction::transfer(&alice, &bob, 100_000);
    let ix2 = solana_system_interface::instruction::transfer(&alice, &bob, 200_000);

    let result = ctx
        .transaction()
        .add_instruction(ix1)
        .add_instruction(ix2)
        .execute()
        .unwrap();

    assert!(result.is_success());
    assert!(result.failed_at().is_none());
    assert!(result.last_result().is_some());
    assert_eq!(result.instruction_results.len(), 2);
    assert!(result.total_compute_units > 0);
}

#[test]
fn test_execute_allow_failures() {
    let ctx = MolluskContextHelper::new_without_program();

    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();

    ctx.fund_account(&alice, 100_000);
    ctx.fund_account(&bob, 100_000);

    let ix1 = solana_system_interface::instruction::transfer(&alice, &bob, 50_000);
    let ix2 = solana_system_interface::instruction::transfer(&alice, &bob, 100_000); // Will fail

    let result = ctx
        .transaction()
        .add_instruction(ix1)
        .add_instruction(ix2)
        .execute_allow_failures();

    assert!(!result.is_success());
    assert_eq!(result.failed_at(), Some(1));
    assert_eq!(result.instruction_results.len(), 2);

    assert_eq!(ctx.get_balance(&alice), Some(100_000));
    assert_eq!(ctx.get_balance(&bob), Some(100_000));
}
