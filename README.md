# mollusk-helper

A simplified wrapper around [mollusk-svm](https://github.com/anza-xyz/mollusk) for easier Solana program testing with atomic transaction support.

## Features

- **Unified API**: All functionality accessible through `MolluskContextHelper`
- **Atomic Transactions**: Run multiple instructions with snapshot/restore semantics
- **SPL Token Support**: Built-in helpers for mints, token accounts, and transfers
- **Keypair Management**: Store and retrieve keypairs by name for signing
- **Add Custom Programs**: Dynamically add programs after context creation

## Default Programs

The following programs are included by default:

| Program | ID |
|---------|-----|
| System Program | `11111111111111111111111111111111` |
| SPL Token | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| SPL Token 2022 | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` |
| Associated Token | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` |
| Memo | `MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr` |
| Memo v1 | `Memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo` |
| BPF Loader v2 | `BPFLoader2111111111111111111111111111111111` |
| BPF Loader v3 | `BPFLoaderUpgradeab1e11111111111111111111111` |
| BPF Loader v1 | `BPFLoader1111111111111111111111111111111111` |
| Loader v4 | `LoaderV411111111111111111111111111111111111` |

## Installation

Add to your `Cargo.toml`:

```toml
[dev-dependencies]
mollusk-helper = { git = "https://github.com/ohaddahan/mollusk-helper" }
```

## Quick Start

```rust
use mollusk_helper::prelude::*;

// Create context with your program
let elf = std::fs::read("path/to/your/program.so").unwrap();
let ctx = MolluskContextHelper::new(&program_id, &elf);

// Or without a custom program (for testing system/token operations)
let ctx = MolluskContextHelper::new_without_program();

// Fund accounts
ctx.fund_account(&alice, 1_000_000);
ctx.fund_account(&bob, 0);

// Execute single instruction
ctx.transfer_sol(&alice, &bob, 500_000)?;
```

## Adding Custom Programs

You can add programs dynamically after context creation:

```rust
let mut ctx = MolluskContextHelper::new_without_program();

// Add program with default loader (v3/upgradeable)
let elf = std::fs::read("path/to/program.so").unwrap();
ctx.add_program(&program_id, &elf);

// Or specify the loader version
ctx.add_program_with_loader(&program_id, &elf, ProgramLoader::V2);
```

### Constructor Variants

```rust
// With custom program (loader v3 by default)
let ctx = MolluskContextHelper::new(&program_id, &elf);

// With custom timestamp
let ctx = MolluskContextHelper::new_with_timestamp(&program_id, &elf, unix_timestamp);

// With specific loader
let ctx = MolluskContextHelper::new_with_loader(&program_id, &elf, ProgramLoader::V2);

// Full control
let ctx = MolluskContextHelper::new_with_options(
    &program_id,
    &elf,
    ProgramLoader::V3,
    unix_timestamp,
);

// Without custom program
let ctx = MolluskContextHelper::new_without_program();
```

## Atomic Transactions

Run multiple instructions as an atomic unit. If any instruction fails, all changes are rolled back:

```rust
let result = ctx
    .transaction()
    .add_instruction(transfer_ix_1)
    .add_instruction(transfer_ix_2)
    .execute()?;

// On success: both transfers applied
// On failure: neither transfer applied (state restored)
```

### Transaction Methods

- `execute()` - Stops on first failure, rolls back all changes
- `execute_allow_failures()` - Runs all instructions, rolls back if any failed
- `dry_run()` - Executes but always restores original state

## Token Operations

```rust
// Create mint and token account
ctx.create_mint(&mint, &authority, 9);
ctx.create_token_account(&token_account, &mint, &owner, 0);

// Mint tokens
ctx.mint_to(&mint, &token_account, &authority, 1_000_000)?;

// Check balance
let balance = ctx.get_token_balance(&token_account)?;

// Transfer tokens
ctx.transfer_tokens(&source, &destination, &authority, 500_000)?;
```

## Keypair Management

```rust
let keypair = Keypair::new();
ctx.store_keypair("signer", keypair)?;

let pubkey = ctx.get_keypair_pubkey("signer")?;
let signature = ctx.sign_with("signer", &message)?;
```

## Program ID Constants

```rust
MolluskContextHelper::system_program()
MolluskContextHelper::token_program()
MolluskContextHelper::token_2022_program()
MolluskContextHelper::associated_token_program()
MolluskContextHelper::memo_program()
MolluskContextHelper::memo_v1_program()
MolluskContextHelper::address_lookup_table_program()
MolluskContextHelper::compute_budget_program()
MolluskContextHelper::native_mint()
```

## API Reference

### Constructors

| Method | Description |
|--------|-------------|
| `new(program_id, elf_bytes)` | Create with custom program (loader v3) |
| `new_with_timestamp(...)` | Create with custom timestamp |
| `new_with_loader(...)` | Create with specific loader |
| `new_with_options(...)` | Full control over all options |
| `new_without_program()` | Create without custom program |

### Program Management

| Method | Description |
|--------|-------------|
| `add_program(program_id, elf)` | Add program with loader v3 |
| `add_program_with_loader(...)` | Add program with specific loader |

### Instruction Processing

| Method | Description |
|--------|-------------|
| `process_instruction(ix)` | Execute single instruction, return `Result` |
| `process_instruction_unchecked(ix)` | Execute without error handling |
| `transaction()` | Start building atomic transaction |

### Account Management

| Method | Description |
|--------|-------------|
| `add_account(pubkey, account)` | Add account to store |
| `get_account(pubkey)` | Get account from store |
| `get_balance(pubkey)` | Get SOL balance |
| `fund_account(pubkey, lamports)` | Create funded system account |

### Token Helpers

| Method | Description |
|--------|-------------|
| `create_mint(pubkey, authority, decimals)` | Create mint account |
| `create_token_account(pubkey, mint, owner, amount)` | Create token account |
| `create_native_token_account(pubkey, owner, lamports)` | Create wSOL account |
| `get_token_balance(pubkey)` | Get token balance |
| `mint_to(mint, dest, auth, amount)` | Mint tokens |
| `transfer_tokens(src, dest, auth, amount)` | Transfer tokens |
| `get_associated_token_address(wallet, mint)` | Derive ATA address |

## License

MIT
