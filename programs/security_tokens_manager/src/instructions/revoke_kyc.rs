use anchor_lang::prelude::*;

use crate::constants::{CONFIG_SEED, KYC_SEED};
use crate::error::ErrorCode;
use crate::state::{KycRecord, KycStatus, TokenConfig};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RevokeKycArgs {
    /// True = permanently revoked; false = temporary suspension pending review.
    pub permanent: bool,
}

#[derive(Accounts)]
pub struct RevokeKyc<'info> {
    pub kyc_operator: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = kyc_operator @ ErrorCode::NotKycOperator,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// CHECK: Verified via token_config.mint.
    #[account(address = token_config.mint)]
    pub mint: AccountInfo<'info>,

    /// CHECK: The token account whose KYC is being revoked.
    pub token_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [KYC_SEED, mint.key().as_ref(), token_account.key().as_ref()],
        bump = kyc_record.bump,
    )]
    pub kyc_record: Account<'info, KycRecord>,
}

pub fn handler(ctx: Context<RevokeKyc>, args: RevokeKycArgs) -> Result<()> {
    let record = &mut ctx.accounts.kyc_record;
    record.status = if args.permanent {
        KycStatus::Revoked
    } else {
        KycStatus::Suspended
    };

    msg!(
        "KYC {}: token_account={}",
        if args.permanent {
            "revoked"
        } else {
            "suspended"
        },
        ctx.accounts.token_account.key
    );

    Ok(())
}
