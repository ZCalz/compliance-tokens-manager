use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;

use crate::constants::{CONFIG_SEED, KYC_SEED};
use crate::error::ErrorCode;
use crate::state::{KycRecord, KycStatus, TokenConfig};

#[derive(Accounts)]
pub struct IssueTokens<'info> {
    pub issuer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = issuer @ ErrorCode::NotIssuer,
        has_one = mint,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// CHECK: Mint address validated via token_config.has_one.
    #[account(mut)]
    pub mint: AccountInfo<'info>,

    /// The investor's token account. Must be thawed before issuance.
    /// CHECK: Must belong to this mint. Validated by the token program.
    #[account(mut)]
    pub destination: AccountInfo<'info>,

    /// KYC record for the destination token account.
    #[account(
        seeds = [KYC_SEED, mint.key().as_ref(), destination.key().as_ref()],
        bump = kyc_record.bump,
    )]
    pub kyc_record: Account<'info, KycRecord>,

    /// CHECK: Token-2022 program.
    #[account(address = spl_token_2022_interface::ID)]
    pub token_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<IssueTokens>, amount: u64) -> Result<()> {
    require!(amount > 0, ErrorCode::ZeroAmount);

    let record = &ctx.accounts.kyc_record;
    require!(record.status == KycStatus::Active, ErrorCode::KycNotActive);
    if record.expires_at != 0 {
        let clock = Clock::get()?;
        require!(
            record.expires_at > clock.unix_timestamp,
            ErrorCode::KycExpired
        );
    }
    require!(
        record.kyc_level >= ctx.accounts.token_config.required_kyc_level,
        ErrorCode::InsufficientKycLevel
    );

    invoke(
        &spl_token_2022_interface::instruction::mint_to(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            ctx.accounts.destination.key,
            ctx.accounts.issuer.key,
            &[],
            amount,
        )?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.destination.to_account_info(),
            ctx.accounts.issuer.to_account_info(),
        ],
    )?;

    msg!(
        "Issued {} tokens to {} for mint {}",
        amount,
        ctx.accounts.destination.key,
        ctx.accounts.mint.key
    );

    Ok(())
}
