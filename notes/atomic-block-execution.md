# Atomic Block Execution

A **block** is an ordered batch of transactions. **Atomic block execution** means the block is treated as one indivisible state transition: either every transaction in the block is applied and the block is appended, or none of them are and the chain is unchanged.

This is a state machine safety rule, not a Rust-specific trick. In this project it is enforced in `src/blockchain.rs` (`append_block`) and verified in `tests/blockchain_tests.rs`.

## Our blockchain model

A `Blockchain` holds two pieces of state:

| Component | Role |
|-----------|------|
| `blocks: Vec<Block>` | Ordered chain history |
| `ledger: Ledger` | Current account balances and nonces |

Each `Block` contains:

- A list of `Transaction` values (`sender`, `receiver`, `amount`, `nonce`).
- A `BlockHeader` with `height`, `previous_hash`, `transaction_commitment`, and `state_commitment`.

`append_block` is the only way to advance the chain. It validates the block, executes its transactions against the ledger, checks that the resulting `state_commitment` matches the header, and only then updates `self.ledger` and pushes the block.

Individual transactions already have atomicity at the ledger level (see `notes/ledger-invariants.md`). Block-level atomicity extends that rule to the whole batch.

## Problem statement

Consider a block at height 1 with two transactions:

1. `Transaction::new("alice", "bob", 10, 0)` — valid
2. `Transaction::new("alice", "bob", 500, 1)` — invalid (`InsufficientBalance`)

Alice starts with balance `100` and nonce `0`.

**Wrong behavior (partial commit):** apply transaction 1, then reject transaction 2 and keep the block.

After this "half-applied" block:

- Alice: balance `90`, nonce `1`
- Bob: balance `110`, nonce `0`
- The block appears in `blocks`, but its `state_commitment` header was computed for a different final ledger

**Correct behavior (atomic execution):** reject the entire block.

After rejection:

- Alice: balance `100`, nonce `0`
- Bob: balance `100`, nonce `0`
- `blocks.len()` is unchanged
- The ledger matches the previous block's `state_commitment`

A block with 10 transactions is not valid just because the first 9 succeed. If transaction 10 fails, the whole block fails.

## Why a block must fully commit or fully fail

### 1. Determinism requires one final ledger per block

Determinism means: given the same starting chain and the same block, every correct implementation reaches the **same** final ledger.

If two nodes execute the same block differently:

- Node A keeps a valid prefix after a later transaction fails → Alice ends at `90/1`.
- Node B discards the whole block → Alice stays at `100/0`.

They now disagree on every future question: Alice's balance, her next valid nonce, the ledger's `state_commitment`, and whether height 1 exists at all. There is no single canonical state to build the next block from. The nodes disagree forever.

This is not a networking problem. It is a **state machine definition** problem: the protocol never specified what "apply this block" means when the batch is mixed valid and invalid.

### 2. The block header commits to one final state, not a prefix

Each block header stores `state_commitment`: a hash of the sorted account map after **all** transactions in the block succeed.

```text
BlockHeader.state_commitment = hash(sorted accounts after tx[0..n])
```

If you apply a prefix and stop:

- The live ledger no longer matches `state_commitment`.
- Step 6 of `append_block` (`InvalidStateCommitment`) would fail — correctly, because the header lied about the result.
- Even if you skipped that check, the next block's `previous_hash` chain would point at a block whose committed state does not match reality.

Atomic execution keeps the header, the transaction list, and the ledger aligned.

### 3. Nonces and balances are coupled across transactions inside a block

Transactions in a block run in order. Transaction 2 reads Alice's nonce and balance as left by transaction 1.

If transaction 1 commits but transaction 2 fails:

- Alice's nonce advanced, so the mempool and wallets believe she sent two transfers.
- The second transfer never completed, so Bob did not receive those tokens.
- Future blocks expect nonce `2` for Alice's next send, but clients may still be signing with nonce `1`.

Partial block application breaks replay ordering the same way a partially applied single transfer would — but at batch scale.

### 4. Conservation of supply must hold at block boundaries

Our ledger invariant: `total_supply()` is unchanged by valid transfers.

If a block partially applies, you can end up with:

- Tokens debited from a sender with no matching credit (if execution stops mid-block).
- A block recorded in history whose effects do not match any valid sequential execution.

`append_block` checks `total_supply` before and after the block. That check only makes sense if the block either fully applies or does not apply at all.

## What atomic execution means here

Formally, for a block `B` with transactions `[t₁, t₂, …, tₙ]`:

```text
append_block(B) succeeds
  ⟺  every tᵢ applies successfully in order
  ⟺  final ledger matches B.header.state_commitment
  ⟹  ledger and blocks are both updated

append_block(B) fails
  ⟹  ledger is identical to before the call
  ⟹  blocks is identical to before the call
```

There is no third outcome. No "block accepted with 9 of 10 transactions."

## How `append_block` enforces it

The implementation in `src/blockchain.rs`:

1. Run cheap structural checks first (height, `previous_hash`, `transaction_commitment`, duplicate tx IDs).
2. **Clone the ledger** into a temporary copy.
3. Apply every transaction to the temporary copy. If any returns `LedgerError`, return `ChainError::TransactionFailed` immediately.
4. Compute `state_commitment` on the temporary copy and compare to the block header.
5. Verify ledger invariants (including `total_supply`).
6. **Only on full success:** assign `self.ledger = ledger` and `self.blocks.push(block)`.

If step 3, 4, or 5 fails, the real `self.ledger` and `self.blocks` were never touched. The temporary copy is dropped.

This is validate-then-commit at block granularity, matching the validate-then-mutate pattern used inside `Ledger::apply_transaction`.

## Concrete failure scenarios in our tests

| Test | Scenario | Expected outcome |
|------|----------|------------------|
| `valid_prefix_invalid_second_transaction_discards_entire_block` | Valid tx₁, invalid tx₂ | Alice/Bob unchanged at `100/100`, nonce `0` |
| `wrong_nonce_midway_discards_entire_block` | Valid tx₁, wrong nonce on tx₂ | Entire block rejected, ledger unchanged |
| `insufficient_funds_midway_discards_entire_block` | Two valid txs, overdraft on tx₃ | Entire block rejected, ledger unchanged |
| `failed_block_does_not_change_ledger` | Single invalid tx | Ledger clone and `state_commitment` unchanged |
| `failed_block_does_not_change_block_count` | Single invalid tx | `blocks.len()` unchanged |

These tests assert the **no-mutation property** on failure: not just that an error is returned, but that the chain state is identical to before the call.

## Atomicity at two levels

| Level | Unit | Rule |
|-------|------|------|
| Transaction | One `Transaction` | All balance and nonce updates succeed together, or none happen |
| Block | One `Block` | All transactions in the block succeed together, or none happen and the block is not appended |

Block atomicity composes transaction atomicity. You cannot safely enforce block atomicity by applying transactions directly to the live ledger and "hoping" to roll back — rollback is error-prone and non-deterministic across implementations. Executing against a temporary copy and committing once is the straightforward rule: **one block, one state transition, one outcome.**

## Summary

- A block is a single step in the chain's state machine.
- Partial application creates divergent ledgers, breaks `state_commitment`, and corrupts nonce ordering.
- Atomic execution guarantees that every node applying the same block reaches the same final ledger — or rejects the block entirely with no change.
- In this codebase, that rule is implemented by executing on a cloned ledger and committing ledger + block together only after every check passes.
