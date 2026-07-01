# rust-systems-lab

A minimal, deterministic account-based ledger and blockchain state machine written in Rust. The project explores protocol fundamentals—transaction identity, ledger invariants, state commitments, and atomic block execution—without networking, consensus, or cryptography.

## Project purpose

This is a **systems learning lab**, not a production blockchain. It implements a single-node chain where:

- Accounts hold token balances and send nonces.
- Transfers are validated and applied atomically.
- Blocks batch transactions and commit to a verifiable final ledger state.
- Every operation returns explicit errors; failed operations leave state unchanged.

The goal is to make protocol rules concrete: deterministic state transitions, content-derived transaction IDs, conservation of supply, and all-or-nothing block execution. See the `notes/` directory for deeper write-ups on [ledger invariants](notes/ledger-invariants.md), [transaction identity](notes/deterministic-transactions.md), and [atomic block execution](notes/atomic-block-execution.md).

## Architecture

The crate is organized as a layered state machine:

```
Transaction  →  Ledger  →  Block  →  Blockchain
     │              │          │            │
  validate      apply tx    header      append_block
  hash_id       state_commit  hash      (atomic)
```

| Module | Role |
|--------|------|
| `src/account.rs` | Account struct (`balance`, `nonce`) |
| `src/transaction.rs` | Transfer payload, structural validation, canonical encoding, content-derived ID |
| `src/ledger.rs` | Account map, `create_account`, `apply_transaction`, `state_commitment` |
| `src/traits/state_transition.rs` | `StateTransition` trait; ledger apply logic (validate-then-mutate) |
| `src/block.rs` | `Block`, `BlockHeader`, transaction commitment, block hash |
| `src/blockchain.rs` | Chain storage and `append_block` (the only way to advance state) |
| `src/hash.rs` | Canonical bincode encoding and SHA-256 hashing |
| `src/error.rs` | Typed errors for each layer |

A `Blockchain` holds two pieces of live state:

- **`blocks: Vec<Block>`** — ordered chain history
- **`ledger: Ledger`** — current account balances and nonces

`append_block` is the sole entry point for advancing the chain. There is no networking, mempool, or persistence layer.

## Account model

Each account is identified by a unique string ID and stores:

| Field | Type | Meaning |
|-------|------|---------|
| `balance` | `u64` | Tokens held by the account |
| `nonce` | `u64` | Number of transfers already sent from this account |

Rules:

- Accounts are created explicitly via `Ledger::create_account(id, balance)` with an initial nonce of `0`.
- Account IDs must be unique; duplicate creation returns `LedgerError::AccountAlreadyExists`.
- Only the **sender** nonce advances (by exactly 1) on a successful transfer. Receiving does not change the receiver's nonce.
- Balances are unsigned (`u64`); they cannot go negative.

## Transaction lifecycle

A `Transaction` has four fields: `sender`, `receiver`, `amount`, and `nonce`.

### 1. Structural validation

Before touching the ledger, `Validate::validate` checks:

- `amount > 0` → `ZeroAmount`
- `sender != receiver` → `SelfTransfer`

### 2. Ledger validation

`Ledger::apply_transaction` (via `StateTransition<Transaction>`) checks:

- Sender and receiver accounts exist
- `transaction.nonce == sender.nonce`
- `sender.balance >= amount`
- `receiver.balance + amount` does not overflow `u64`

All checks run inside an immutable-borrow block **before** any mutation.

### 3. Application

On success:

- Sender balance decreases by `amount`; sender nonce increases by 1
- Receiver balance increases by `amount`; receiver nonce is unchanged

On failure, the ledger is unchanged (validate-then-mutate).

### 4. Identity

Transaction IDs are **content-derived**, not assigned:

```
Transaction  →  canonical_bytes()  →  SHA-256  →  HashedId
```

Encoding uses bincode with `config::standard()`. Identical content always produces the same ID on every machine.

### 5. Inclusion in a block

Transactions are ordered in a block's `transactions` vector. The block header's `transaction_commitment` is the SHA-256 hash of each transaction's `hash_id`, concatenated in order.

## Block execution

`Blockchain::append_block` validates and executes a block atomically:

1. **Height** — `header.height` must equal `blocks.len()`
2. **Previous hash** — must match the last block's hash (or `[0; 32]` for the first block)
3. **Transaction commitment** — must match `Block::transaction_commitment(&transactions)`
4. **Unique transaction IDs** — no duplicate `hash_id` values within the block
5. **Atomic execution** — clone the live ledger, apply every transaction in order; abort on first failure
6. **State commitment** — computed `ledger.state_commitment()` must match `header.state_commitment`
7. **Supply invariant** — `total_supply()` before and after must be equal
8. **Commit** — only if all checks pass: replace `self.ledger` and push the block

### Atomicity guarantee

A block either **fully succeeds** (all transactions apply, ledger and chain both update) or **fully fails** (ledger and `blocks` are identical to before the call). There is no partial application.

This is enforced by executing against a temporary ledger clone and committing once. See [notes/atomic-block-execution.md](notes/atomic-block-execution.md) for the rationale.

### Block header fields

| Field | Meaning |
|-------|---------|
| `height` | Block index in the chain |
| `previous_hash` | Hash of the previous block header |
| `transaction_commitment` | Commitment to the ordered transaction list |
| `state_commitment` | SHA-256 of the sorted account map after all transactions succeed |

`state_commitment` sorts accounts by ID, encodes with canonical bincode, and hashes with SHA-256. Insertion order into the `HashMap` does not affect the result.

## Invariants

These properties must hold at all times. Violations cause operations to be rejected rather than partially applied.

### Account invariants

- Unique account IDs
- Non-negative balances (`u64`)
- Initial nonce is zero
- Nonce advances only on send

### Transfer invariants

- Positive amount, distinct sender/receiver
- Correct nonce, sufficient balance, no overflow
- Atomic: all balance and nonce updates succeed together or none happen

### Ledger invariants

- **Conservation of supply** — `total_supply()` is unchanged by valid transfers
- **No partial updates** — validation completes before mutation

### Block invariants

- Block header fields match computed values
- No duplicate transaction IDs within a block
- Final ledger matches `state_commitment` in the header
- Total supply unchanged across block execution

### Chain invariants

- Blocks are strictly ordered by height with a valid hash chain
- Failed blocks leave both `ledger` and `blocks` unchanged

## Testing approach

Tests live in `tests/` as integration tests (50 tests total). There are no unit tests in `src/`; coverage is organized by layer:

| File | Focus |
|------|-------|
| `tests/ledger_tests.rs` | Account creation, transfer validation, supply conservation, no-mutation on error |
| `tests/transaction_tests.rs` | Canonical encoding, content-derived IDs, serialization round-trip |
| `tests/block_tests.rs` | Block header hashing sensitivity |
| `tests/state_commitment_tests.rs` | Deterministic state commitment across insertion order |
| `tests/blockchain_tests.rs` | Block validation, atomic execution, no-mutation on failure |

Every error path asserts the **no-mutation property**: after a rejected operation, balances, nonces, block count, and state commitment are unchanged. This mirrors how protocol software verifies state machine safety.

## How to run on Windows

### Prerequisites

Install [Rust](https://rustup.rs/) (stable toolchain). Open PowerShell or Command Prompt in the project root.

### Build

```powershell
cargo build
```

### Test

```powershell
cargo test
```

### Lint (Clippy, warnings as errors)

```powershell
cargo clippy --all-targets --all-features -- -D warnings
```

### Format check

```powershell
cargo fmt --all -- --check
```

To apply formatting instead of checking:

```powershell
cargo fmt --all
```

## Known limitations

- **Single node only** — no networking, peer sync, or consensus (PoW, PoS, etc.)
- **No cryptography** — transactions are not signed; account IDs are plain strings, not public keys
- **No mempool** — blocks are constructed and submitted directly; there is no transaction pool
- **No persistence** — state lives in memory; restarting the process loses the chain
- **No genesis protocol** — the first block is a convention in tests, not a hard-coded genesis spec
- **No fees or rewards** — transfers move existing tokens only; block producers are not compensated
- **No smart contracts** — only simple balance transfers are supported
- **Empty binary** — `src/main.rs` is a stub; the library is the product

## Roadmap

Planned extensions, roughly in dependency order:

1. **Cryptographic accounts** — key pairs, signed transactions, address derivation from public keys
2. **Genesis block spec** — fixed initial state and header constants instead of ad-hoc test setup
3. **Mempool** — accept, validate, and order pending transactions before block inclusion
4. **Block builder** — construct valid blocks from mempool contents with correct commitments
5. **Persistence** — serialize and reload chain state from disk
6. **CLI or REPL** — interactive commands to create accounts, submit transactions, and inspect the chain
7. **Multi-node simulation** — in-process nodes exchanging blocks to exercise determinism and fork handling
8. **Consensus stub** — longest-chain or simple BFT rules to choose the canonical fork

Contributions and experiments that stay focused on correctness and determinism are welcome.
