use anchor_lang::prelude::*;

use crate::constants::{CONFIG_SEED, KYC_SEED};
use crate::error::ErrorCode;
use crate::state::{KycLevel, KycRecord, KycStatus, TokenConfig};

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RegisterKycArgs {
    pub kyc_level: KycLevel,
    /// ISO 3166-1 alpha-2 code, e.g. [b'U', b'S'].
    pub jurisdiction: [u8; 2],
    /// Unix timestamp after which the approval expires. 0 = never.
    pub expires_at: i64,
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
pub struct RegisterKyc<'info> {
    /// Must be the kyc_operator stored in token_config.
    #[account(mut)]
    pub kyc_operator: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = kyc_operator @ ErrorCode::NotKycOperator,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// The Token-2022 mint.
    /// CHECK: Checked implicitly via token_config.mint = mint.
    #[account(address = token_config.mint)]
    pub mint: AccountInfo<'info>,

    /// The investor's Associated Token Account for this mint.
    /// KYC is tied to the token account (not just the wallet) so the hook
    /// can derive the PDA deterministically from the token account address.
    /// CHECK: Any token account for this mint. Validated by KYC operator off-chain.
    pub token_account: AccountInfo<'info>,

    /// The wallet that owns token_account. Stored for audit purposes.
    /// CHECK: Any pubkey. Recorded as-is.
    pub wallet: AccountInfo<'info>,

    /// KYC record PDA being created.
    #[account(
        init,
        payer = kyc_operator,
        space = 8 + KycRecord::INIT_SPACE,
        seeds = [KYC_SEED, mint.key().as_ref(), token_account.key().as_ref()],
        bump,
    )]
    pub kyc_record: Account<'info, KycRecord>,

    pub system_program: Program<'info, System>,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub fn handler(ctx: Context<RegisterKyc>, args: RegisterKycArgs) -> Result<()> {
    // Verify the investor's jurisdiction is on the allowlist for this token.
    let config = &ctx.accounts.token_config;
    require!(
        config.jurisdiction_allowlist.contains(&args.jurisdiction),
        ErrorCode::JurisdictionNotAllowed
    );

    // Verify the KYC level meets the token's minimum.
    require!(
        args.kyc_level >= config.required_kyc_level,
        ErrorCode::InsufficientKycLevel
    );

    let clock = Clock::get()?;

    let record = &mut ctx.accounts.kyc_record;
    record.mint = *ctx.accounts.mint.key;
    record.token_account = *ctx.accounts.token_account.key;
    record.wallet = *ctx.accounts.wallet.key;
    record.kyc_level = args.kyc_level;
    record.jurisdiction = args.jurisdiction;
    record.status = KycStatus::Active;
    record.expires_at = args.expires_at;
    record.kyc_operator = *ctx.accounts.kyc_operator.key;
    record.registered_at = clock.unix_timestamp;
    record.bump = ctx.bumps.kyc_record;

    msg!(
        "KYC registered: token_account={} wallet={} level={} jurisdiction={}{}",
        ctx.accounts.token_account.key,
        ctx.accounts.wallet.key,
        args.kyc_level as u8,
        args.jurisdiction[0] as char,
        args.jurisdiction[1] as char,
    );

    Ok(())
}
