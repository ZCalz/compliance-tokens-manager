use anchor_lang::prelude::*;

// ---------------------------------------------------------------------------
// TokenConfig
// Seeds: ["config", mint]
// Created once per mint by create_mint. Stores compliance rules and roles.
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct TokenConfig {
    /// The Token-2022 mint this config governs.
    pub mint: Pubkey,

    /// Can issue tokens, update config, and perform forced transfers / burns.
    pub issuer: Pubkey,

    /// Can register and revoke KYC records for investor token accounts.
    pub kyc_operator: Pubkey,

    /// Can freeze and thaw individual token accounts.
    pub compliance_officer: Pubkey,

    /// Program ID of the transfer hook enforcing KYC on every transfer.
    pub transfer_hook_program: Pubkey,

    /// Minimum KYC level an investor must hold to receive or send this token.
    pub required_kyc_level: KycLevel,

    /// ISO 3166-1 alpha-2 country codes permitted to hold this token.
    /// Each entry is exactly 2 bytes, e.g. b"US", b"DE".
    #[max_len(30)]
    pub jurisdiction_allowlist: Vec<[u8; 2]>,

    /// Max tokens (in base units) a single wallet may transfer in one UTC day.
    /// 0 = no limit.
    pub daily_transfer_limit: u64,

    /// Max tokens (in base units) a single wallet may transfer in one UTC month.
    /// 0 = no limit.
    pub monthly_transfer_limit: u64,

    pub bump: u8,
}

// ---------------------------------------------------------------------------
// KycRecord
// Seeds: ["kyc", mint, token_account]
// One record per (mint, token_account) pair. Created when a KYC operator
// approves a specific investor token account. Must exist and be Active for
// transfers to succeed through the hook.
// ---------------------------------------------------------------------------

#[account]
#[derive(InitSpace)]
pub struct KycRecord {
    /// The Token-2022 mint this record belongs to.
    pub mint: Pubkey,

    /// The specific token account (ATA) being approved.
    pub token_account: Pubkey,

    /// The wallet that owns the token account. Stored for audit purposes.
    pub wallet: Pubkey,

    /// KYC tier this investor has been verified at.
    pub kyc_level: KycLevel,

    /// ISO 3166-1 alpha-2 country code of this investor, e.g. b"US".
    pub jurisdiction: [u8; 2],

    /// Current status of this KYC approval.
    pub status: KycStatus,

    /// Unix timestamp after which this record is considered expired.
    /// 0 means it never expires.
    pub expires_at: i64,

    /// The KYC operator who created this record.
    pub kyc_operator: Pubkey,

    /// Unix timestamp when this record was registered.
    pub registered_at: i64,

    pub bump: u8,
}

// ---------------------------------------------------------------------------
// Enums
// ---------------------------------------------------------------------------

#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, InitSpace,
)]
pub enum KycLevel {
    /// Basic identity verification (name, DOB, ID document). Serialised as 0.
    Basic,
    /// Accredited investor verification (income / net worth). Serialised as 1.
    Accredited,
    /// Institutional investor verification (entity docs, AML screening). Serialised as 2.
    Institutional,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum KycStatus {
    /// Investor is fully approved and may transact.
    Active,
    /// Approval has been withdrawn. Transfers will fail.
    Revoked,
    /// Temporary hold pending investigation. Transfers will fail.
    Suspended,
}
