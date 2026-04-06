use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // --- KYC errors ---
    #[msg("KYC record not found for this token account")]
    KycNotFound,
    #[msg("KYC record is not active (revoked or suspended)")]
    KycNotActive,
    #[msg("KYC record has expired")]
    KycExpired,
    #[msg("KYC level is below the minimum required for this token")]
    InsufficientKycLevel,
    #[msg("KYC record already exists for this token account")]
    KycAlreadyRegistered,

    // --- Authority errors ---
    #[msg("Caller is not the issuer of this token")]
    NotIssuer,
    #[msg("Caller is not the KYC operator for this token")]
    NotKycOperator,
    #[msg("Caller is not the compliance officer for this token")]
    NotComplianceOfficer,

    // --- Jurisdiction errors ---
    #[msg("Investor jurisdiction is not on the allowlist for this token")]
    JurisdictionNotAllowed,

    // --- Mint errors ---
    #[msg("Failed to calculate mint account size for requested extensions")]
    InvalidMintSize,
    #[msg("Token name exceeds maximum length of 64 characters")]
    NameTooLong,
    #[msg("Token symbol exceeds maximum length of 10 characters")]
    SymbolTooLong,
    #[msg("Token URI exceeds maximum length of 256 characters")]
    UriTooLong,
    #[msg("Jurisdiction allowlist exceeds maximum of 30 entries")]
    TooManyJurisdictions,

    // --- Transfer errors ---
    #[msg("Transfer amount is zero")]
    ZeroAmount,
}
