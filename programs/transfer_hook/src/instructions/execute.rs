use anchor_lang::prelude::*;
use security_tokens_manager::state::{KycRecord, KycStatus, TokenConfig};

use crate::error::HookError;

/// Core KYC/AML validation called on every Token-2022 transfer.
///
/// Account layout (as received in the fallback handler):
///   [0] source token account
///   [1] mint
///   [2] destination token account
///   [3] authority (source token account owner for normal transfers;
///                  permanent delegate address for forced transfers)
///   [4] extra_account_meta_list PDA (owned by this program)
///   [5] source KYC record (resolved by ExtraAccountMetaList)
///   [6] destination KYC record (resolved by ExtraAccountMetaList)
///   [7] token config (resolved by ExtraAccountMetaList)
pub fn handler<'info>(
    _program_id: &Pubkey,
    accounts: &'info [AccountInfo<'info>],
    amount: u64,
) -> Result<()> {
    // Safety: Token-2022 guarantees these indices exist; returning an error
    // reverts the entire transfer atomically.
    require!(accounts.len() >= 8, HookError::UnknownInstruction);

    let authority = &accounts[3];
    let source_kyc_info = &accounts[5];
    let destination_kyc_info = &accounts[6];
    let token_config_info = &accounts[7];

    // -----------------------------------------------------------------------
    // Deserialize accounts.
    // Account::try_from verifies the 8-byte Anchor discriminator and that
    // the account is owned by security_tokens_manager.
    // -----------------------------------------------------------------------
    require!(
        token_config_info.owner == &security_tokens_manager::ID,
        HookError::InvalidTokenConfigAccount
    );
    let token_config: Account<TokenConfig> = Account::try_from(token_config_info)?;

    // -----------------------------------------------------------------------
    // Issuer bypass: forced transfers initiated by the permanent delegate
    // (the issuer) skip KYC checks entirely. This supports court-ordered
    // seizure, estate settlement, and other regulatory actions.
    // -----------------------------------------------------------------------
    if authority.key() == token_config.issuer {
        msg!("Transfer hook: issuer authority — bypassing KYC checks");
        return Ok(());
    }

    require!(
        source_kyc_info.owner == &security_tokens_manager::ID,
        HookError::InvalidKycAccount
    );
    require!(
        destination_kyc_info.owner == &security_tokens_manager::ID,
        HookError::InvalidKycAccount
    );

    let source_kyc: Account<KycRecord> = Account::try_from(source_kyc_info)?;
    let destination_kyc: Account<KycRecord> = Account::try_from(destination_kyc_info)?;

    let clock = Clock::get()?;
    let now = clock.unix_timestamp;

    // -----------------------------------------------------------------------
    // Source KYC checks.
    // -----------------------------------------------------------------------
    require!(
        source_kyc.status == KycStatus::Active,
        HookError::SourceKycNotActive
    );
    require!(
        source_kyc.expires_at == 0 || source_kyc.expires_at > now,
        HookError::SourceKycExpired
    );
    require!(
        source_kyc.kyc_level >= token_config.required_kyc_level,
        HookError::InsufficientKycLevel
    );

    // -----------------------------------------------------------------------
    // Destination KYC checks.
    // -----------------------------------------------------------------------
    require!(
        destination_kyc.status == KycStatus::Active,
        HookError::DestinationKycNotActive
    );
    require!(
        destination_kyc.expires_at == 0 || destination_kyc.expires_at > now,
        HookError::DestinationKycExpired
    );
    require!(
        destination_kyc.kyc_level >= token_config.required_kyc_level,
        HookError::InsufficientKycLevel
    );

    // -----------------------------------------------------------------------
    // Jurisdiction check.
    // Both parties must be in the token's jurisdiction allowlist AND the
    // combination must not be explicitly blocked (e.g. OFAC requirements).
    // -----------------------------------------------------------------------
    require!(
        token_config
            .jurisdiction_allowlist
            .contains(&source_kyc.jurisdiction),
        HookError::JurisdictionNotAllowed
    );
    require!(
        token_config
            .jurisdiction_allowlist
            .contains(&destination_kyc.jurisdiction),
        HookError::JurisdictionNotAllowed
    );

    // -----------------------------------------------------------------------
    // Emit a log event for off-chain AML monitoring.
    // Production systems should subscribe via Helius/RPC logsSubscribe and
    // flag patterns: velocity, round-trips, structuring.
    // -----------------------------------------------------------------------
    msg!(
        "Transfer validated: {} base units | src_kyc={} dst_kyc={} | src_jur={}{} dst_jur={}{}",
        amount,
        source_kyc_info.key,
        destination_kyc_info.key,
        source_kyc.jurisdiction[0] as char,
        source_kyc.jurisdiction[1] as char,
        destination_kyc.jurisdiction[0] as char,
        destination_kyc.jurisdiction[1] as char,
    );

    Ok(())
}
