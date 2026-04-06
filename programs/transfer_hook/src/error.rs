use anchor_lang::prelude::*;

#[error_code]
pub enum HookError {
    #[msg("Source account does not have an active KYC record")]
    SourceKycNotFound,
    #[msg("Source KYC record is not active (revoked or suspended)")]
    SourceKycNotActive,
    #[msg("Source KYC record has expired")]
    SourceKycExpired,
    #[msg("Destination account does not have an active KYC record")]
    DestinationKycNotFound,
    #[msg("Destination KYC record is not active (revoked or suspended)")]
    DestinationKycNotActive,
    #[msg("Destination KYC record has expired")]
    DestinationKycExpired,
    #[msg("KYC level is below the minimum required for this token")]
    InsufficientKycLevel,
    #[msg("Transfer between these jurisdictions is not permitted for this token")]
    JurisdictionNotAllowed,
    #[msg("Instruction is not a recognized transfer hook execute instruction")]
    UnknownInstruction,
    #[msg("Invalid KYC account: not owned by security_tokens_manager")]
    InvalidKycAccount,
    #[msg("Invalid token config account: not owned by security_tokens_manager")]
    InvalidTokenConfigAccount,
}
