# Architecture: Token-2022 Security Token Manager

## Overview

This project is an Anchor-based Solana program suite for issuing and managing regulated
security tokens using the Token-2022 (Token Extensions) program. Compliance with KYC/AML
requirements is enforced at the protocol level via on-chain transfer hooks, a KYC registry,
and account state controls — not just off-chain policy.

The manager is a **token factory**: a single deployed program that can create and manage
an unlimited number of independent Token-2022 mints. Each mint you create gets its own
name, symbol, metadata, KYC registry, and compliance configuration. Tokens are isolated
from each other — KYC approval for one does not carry over to another.

**See also:** [Tokenized stocks vs this repository (architecture comparison)](tokenized-stocks-architecture.md) — how typical on-chain equity-style programs relate to this design.

---

## Token Factory: Creating New Tokens

Call `create_mint` once per security token you want to issue. You supply the name, symbol,
and configuration at call time — the program enforces the rest.

```
// Issue a private equity token
create_mint {
    name:              "Acme Corp Series A",
    symbol:            "ACME-A",
    uri:               "https://example.com/acme-a/offering.json",  // legal docs, metadata
    decimals:          6,
    jurisdiction_allowlist: ["US"],
    required_kyc_level: Accredited,
    daily_transfer_limit:   1_000_000_000_000,   // in base units
    monthly_transfer_limit: 10_000_000_000_000,
}

// Issue a real estate fund token (different rules, same program)
create_mint {
    name:              "Real Estate Fund II",
    symbol:            "REF-II",
    uri:               "https://example.com/ref-ii/offering.json",
    decimals:          6,
    jurisdiction_allowlist: ["US", "DE", "GB"],
    required_kyc_level: Accredited,
    daily_transfer_limit:   500_000_000_000,
    monthly_transfer_limit: 5_000_000_000_000,
}

// Issue a green bond (open to basic KYC, EU only)
create_mint {
    name:              "Green Bond 2025",
    symbol:            "GB25",
    uri:               "https://example.com/gb25/offering.json",
    decimals:          2,
    jurisdiction_allowlist: ["DE", "FR", "NL", "ES"],
    required_kyc_level: Basic,
    daily_transfer_limit:   250_000_000,
    monthly_transfer_limit: 2_500_000_000,
}
```

Each mint gets its own:
- `TokenConfig` PDA: compliance rules, role assignments, transfer limits
- `ExtraAccountMetaList` PDA: hook account registry scoped to the mint
- Independent KYC registry: `KycRecord` PDAs are `[mint, wallet]` — approval is per-token
- Independent token supply: issue, burn, and manage each token separately

---

## How It Works: The Full Process

### 1. Token Issuance (Issuer Flow)
The issuer creates a Token-2022 mint with a specific set of extensions enabled at mint
creation time. Extensions cannot be added after the mint is initialized — choices are final.

```
Issuer
  │
  ├─▶ create_mint (security-token-manager program)
  │     ├── Extension: TransferHook → points to transfer-hook program
  │     ├── Extension: DefaultAccountState(Frozen) → all new ATAs start frozen
  │     ├── Extension: PermanentDelegate → issuer can force-transfer or burn
  │     ├── Extension: MintCloseAuthority → issuer can close mint at end-of-life
  │     ├── Extension: MetadataPointer + TokenMetadata → ISIN, name, symbol, etc.
  │     └── Extension: RequiredMemo → every transfer must carry a memo (audit trail)
  │
  └─▶ initialize_extra_account_metas (transfer-hook program)
        └── Registers the PDA accounts the hook needs to read during transfers
```

### 2. KYC Onboarding (Investor Flow)
New investors go through an off-chain KYC provider (e.g. Civic, Fractal, Parallel Markets).
Once approved, the KYC operator writes an on-chain `KycRecord` PDA for their wallet.

```
Investor completes KYC off-chain
  │
  └─▶ KYC Operator calls: register_kyc (security-token-manager)
        ├── Creates KycRecord PDA for investor wallet
        │     ├── kyc_level: (basic | accredited | institutional)
        │     ├── jurisdiction: ISO 3166-1 alpha-2 country code
        │     ├── status: Active
        │     └── expires_at: Unix timestamp
        │
        └── Issuer/Operator calls: thaw_account
              └── Unfreezes investor's ATA so they can receive tokens
```

### 3. Token Transfer (Runtime Flow)
Every transfer through Token-2022 automatically invokes the transfer hook. This is enforced
by the protocol — it cannot be bypassed by the sender.

```
Sender initiates transfer
  │
  └─▶ Token-2022 program executes transfer
        │
        └─▶ (automatic) Calls transfer-hook program: execute
                ├── Load KycRecord PDA for source wallet
                │     ├── FAIL if record missing → KYC_NOT_FOUND
                │     ├── FAIL if status != Active → KYC_REVOKED
                │     └── FAIL if expires_at < now → KYC_EXPIRED
                │
                ├── Load KycRecord PDA for destination wallet
                │     └── (same checks as above)
                │
                ├── Check jurisdiction compatibility
                │     └── FAIL if transfer between restricted jurisdiction pairs
                │
                ├── Check transfer velocity (AML)
                │     └── FAIL if sender exceeds daily/monthly limit
                │
                ├── Emit TransferValidated event (for off-chain AML monitoring)
                │
                └── Return OK → Token-2022 completes the transfer
```

### 4. Compliance Actions (Issuer/Regulator Flow)
The `PermanentDelegate` extension and freeze authority give the issuer the tools required
by most securities regulations.

```
Compliance action needed
  │
  ├─▶ freeze_account   → Immediately halts all transfers for a wallet (court order, AML hold)
  ├─▶ thaw_account     → Restores transfer ability after review
  ├─▶ forced_transfer  → Move tokens from any wallet (legal seizure, estate settlement)
  │     └── Uses PermanentDelegate — no user signature required
  ├─▶ revoke_kyc       → Marks KycRecord as Revoked; transfer hook will start failing
  └─▶ burn             → Destroy tokens (corporate action, redemption)
```

---

## On-Chain Program Architecture

Two Anchor programs. One manages the token lifecycle and KYC state. The other is the
transfer hook, called automatically by Token-2022 on every transfer.

```
programs/
├── security-token-manager/          # Program ID: issuer-facing operations
│   └── src/
│       ├── lib.rs                   # Entrypoint, instruction routing
│       ├── instructions/
│       │   ├── create_mint.rs       # Initialize Token-2022 mint with extensions
│       │   ├── issue_tokens.rs      # Mint tokens to a verified investor
│       │   ├── register_kyc.rs      # Write KycRecord PDA for a wallet
│       │   ├── revoke_kyc.rs        # Mark KycRecord as revoked
│       │   ├── freeze_account.rs    # Freeze a token account
│       │   ├── thaw_account.rs      # Unfreeze a token account
│       │   └── forced_transfer.rs   # Transfer using PermanentDelegate
│       ├── state/
│       │   ├── token_config.rs      # Per-mint configuration (limits, jurisdiction rules)
│       │   └── kyc_record.rs        # KYC state per wallet
│       └── errors.rs
│
└── transfer-hook/                   # Program ID: called by Token-2022 on every transfer
    └── src/
        ├── lib.rs
        ├── instructions/
        │   ├── initialize_extra_account_metas.rs  # Registers hook's PDAs with the mint
        │   └── execute.rs                         # Core KYC/AML validation logic
        ├── state/
        │   └── extra_account_metas.rs             # PDA layout for the hook's accounts
        └── errors.rs
```

---

## State / Account Design

### `KycRecord` PDA
Seeds: `["kyc", mint, wallet]`

```rust
pub struct KycRecord {
    pub wallet: Pubkey,          // The investor's wallet
    pub mint: Pubkey,            // Which security token this applies to
    pub kyc_level: KycLevel,     // Basic | Accredited | Institutional
    pub jurisdiction: [u8; 2],   // ISO 3166-1 alpha-2 (e.g. b"US", b"DE")
    pub status: KycStatus,       // Active | Revoked | Expired
    pub expires_at: i64,         // Unix timestamp; 0 = never expires
    pub kyc_operator: Pubkey,    // Who registered this record (for audit)
    pub registered_at: i64,
    pub bump: u8,
}

pub enum KycLevel  { Basic, Accredited, Institutional }
pub enum KycStatus { Active, Revoked, Suspended }
```

### `TokenConfig` PDA
Seeds: `["config", mint]`

```rust
pub struct TokenConfig {
    pub mint: Pubkey,
    pub issuer: Pubkey,                  // Can perform all admin operations
    pub kyc_operator: Pubkey,            // Can register/revoke KYC records
    pub compliance_officer: Pubkey,      // Can freeze/thaw accounts
    pub transfer_hook_program: Pubkey,
    pub required_kyc_level: KycLevel,   // Minimum level to hold this token
    pub jurisdiction_allowlist: Vec<[u8; 2]>, // Permitted investor jurisdictions
    pub daily_transfer_limit: u64,       // AML velocity control (in token base units)
    pub monthly_transfer_limit: u64,
    pub bump: u8,
}
```

### `TransferRecord` PDA (AML velocity tracking)
Seeds: `["transfers", mint, wallet, day_bucket]`

```rust
pub struct TransferRecord {
    pub wallet: Pubkey,
    pub mint: Pubkey,
    pub day_bucket: u32,        // days since Unix epoch → resets daily
    pub month_bucket: u32,      // months since Unix epoch → resets monthly
    pub daily_volume: u64,
    pub monthly_volume: u64,
    pub bump: u8,
}
```

### `ExtraAccountMetaList` PDA
Seeds: `["extra-account-metas", mint]`  
Required by the Token-2022 transfer hook interface. Tells Token-2022 which additional
accounts to pass to the hook program during `execute`.

---

## Token-2022 Extensions Selected

| Extension               | Why It's Needed                                                         |
|-------------------------|-------------------------------------------------------------------------|
| `TransferHook`          | Enforce KYC/AML checks on every transfer — cannot be bypassed           |
| `DefaultAccountState`   | New ATAs start Frozen; investors must be explicitly approved before use  |
| `PermanentDelegate`     | Forced transfers and burns for regulatory/legal requirements             |
| `MintCloseAuthority`    | Token lifecycle management; close mint when security is retired          |
| `MetadataPointer`       | Point to on-chain metadata (TokenMetadata extension)                    |
| `TokenMetadata`         | Store ISIN, security name, issuer name, legal doc URI directly on-chain  |
| `RequiredMemo`          | Every transfer must include a memo — provides AML audit trail            |
| `TransferFee`           | (Optional) Built-in fee for secondary market transactions               |

Extensions NOT used: `NonTransferable` (defeats the purpose), `ConfidentialTransfers`
(conflicts with AML transparency requirements in most jurisdictions).

---

## Roles & Access Control

| Role               | Capabilities                                                   |
|--------------------|----------------------------------------------------------------|
| Issuer             | create_mint, issue_tokens, update_config, forced_transfer, burn |
| KYC Operator       | register_kyc, revoke_kyc, update_kyc_expiry                    |
| Compliance Officer | freeze_account, thaw_account, forced_transfer                  |
| Investor           | transfer (subject to hook validation)                          |

All roles are stored in `TokenConfig`. The issuer can reassign roles. Multiple mints can
have different role assignments (e.g. different KYC operators per token issuance).

---

## Off-Chain Infrastructure

The on-chain programs enforce compliance at execution time. Off-chain infrastructure handles
screening, monitoring, and reporting.

```
┌──────────────────────────────────────────────────────────────────┐
│                     Off-Chain Layer                               │
│                                                                   │
│  ┌──────────────┐   ┌───────────────┐   ┌────────────────────┐  │
│  │  KYC Provider│   │ Admin Dashboard│   │  AML Monitor       │  │
│  │  (Civic,     │──▶│ (issue tokens, │   │  (watch events,    │  │
│  │  Fractal,    │   │  manage roles, │   │  pattern detection,│  │
│  │  Parallel)   │   │  freeze accts) │   │  SAR filing)       │  │
│  └──────┬───────┘   └───────────────┘   └──────────┬─────────┘  │
│         │                                           │             │
│         │ register_kyc (on success)                 │ reads events│
│         ▼                                           ▼             │
└──────────────────────────────────────────────────────────────────┘
          │                                           │
          ▼                                           ▼
   On-chain KycRecord PDA               TransferValidated events
   (wallet approved or revoked)         (emitted by transfer-hook)
```

### KYC Provider Integration
The off-chain KYC service verifies identity, then calls `register_kyc` using a
KYC operator keypair. The on-chain program trusts the operator keypair — it does not
verify the KYC process itself. This keeps the chain simple and provider-agnostic.

### AML Monitoring
- Subscribe to program events via Solana `logsSubscribe` or Helius webhooks
- Watch for `TransferValidated` events to build transaction history
- Flag patterns: rapid velocity, round-trip transfers, structuring
- On flag: compliance officer calls `freeze_account` on-chain

---

## Project Directory Layout (Target)

```
token-2022-security-tokens-manager/
├── CLAUDE.md
├── CLAUDE.local.md
├── Anchor.toml
├── Cargo.toml                        # Workspace
├── package.json                      # Client-side scripts and tests
├── tsconfig.json
│
├── programs/
│   ├── security-token-manager/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── instructions/
│   │       │   ├── mod.rs
│   │       │   ├── create_mint.rs
│   │       │   ├── issue_tokens.rs
│   │       │   ├── register_kyc.rs
│   │       │   ├── revoke_kyc.rs
│   │       │   ├── freeze_account.rs
│   │       │   ├── thaw_account.rs
│   │       │   └── forced_transfer.rs
│   │       ├── state/
│   │       │   ├── mod.rs
│   │       │   ├── token_config.rs
│   │       │   ├── kyc_record.rs
│   │       │   └── transfer_record.rs
│   │       └── errors.rs
│   │
│   └── transfer-hook/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── instructions/
│           │   ├── mod.rs
│           │   ├── initialize_extra_account_metas.rs
│           │   └── execute.rs
│           ├── state/
│           │   ├── mod.rs
│           │   └── extra_account_metas.rs
│           └── errors.rs
│
├── tests/
│   ├── security-token-manager.ts     # Anchor integration tests
│   ├── transfer-hook.ts
│   └── helpers/
│       ├── setup.ts                  # Common test fixtures
│       └── kyc.ts                    # KYC test utilities
│
├── app/                              # Optional: admin CLI or dashboard
│   └── src/
│       ├── client.ts                 # Anchor client wrappers
│       └── commands/                 # CLI commands (issue, kyc, freeze, etc.)
│
└── docs/
    └── architecture.md               # This file
```

---

## Key Constraints & Design Decisions

**Extensions are immutable after mint creation.**
All Token-2022 extensions must be decided at `create_mint` time. There is no way to add
or remove extensions after initialization. The `TokenConfig` PDA can be updated, but the
mint's extension list cannot.

**Transfer hook receives limited accounts.**
The Token-2022 runtime passes a fixed set of accounts to `execute`. Any additional accounts
(like `KycRecord` PDAs) must be pre-registered via `initialize_extra_account_metas` and
will be passed in deterministically based on the mint address. This means the hook can
only read accounts whose addresses can be derived from the transfer's public inputs.

**KYC expiry enforcement.**
`expires_at` in `KycRecord` is checked at transfer time, not re-approval time. This means
a holder's transfers will begin failing automatically when KYC expires. They must re-verify
off-chain and have their record updated on-chain to resume transfers.

**AML velocity resets are bucket-based.**
`TransferRecord` uses day/month buckets (days since Unix epoch) rather than rolling windows.
This is simpler and cheaper on-chain. A true rolling window would require storing a transfer
history, which is impractical at scale.

**`RequiredMemo` and UX.**
Requiring a memo on every transfer affects wallets and dApps that don't natively support
memos. This is a deliberate compliance trade-off. The client SDK should always attach a
memo automatically.

---

## Build Sequence (When Ready to Implement)

1. Initialize Anchor workspace with two programs
2. Implement `security-token-manager` state structs and `create_mint`
3. Implement `transfer-hook`: `initialize_extra_account_metas` and `execute` (stub)
4. Wire up `create_mint` to configure the hook's program ID
5. Implement KYC instructions: `register_kyc`, `revoke_kyc`
6. Implement `execute` with full KYC and velocity checks
7. Implement account management: `freeze_account`, `thaw_account`, `forced_transfer`
8. Write integration tests covering: happy path, expired KYC, revoked KYC, velocity limit,
   jurisdiction block, frozen account, forced transfer
9. Implement client SDK wrappers
10. Deploy to devnet and run end-to-end tests
