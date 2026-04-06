use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;

use crate::constants::CONFIG_SEED;
use crate::error::ErrorCode;
use crate::state::TokenConfig;

/// Forced transfer using the PermanentDelegate extension.
///
/// Only the issuer may call this. The transfer hook fires as normal, but
/// the hook program allows transfers where the authority is the issuer
/// (permanent delegate) regardless of the holders' KYC status.
#[derive(Accounts)]
pub struct ForcedTransfer<'info> {
    pub issuer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = issuer @ ErrorCode::NotIssuer,
        has_one = mint,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// CHECK: Validated via token_config.has_one.
    pub mint: AccountInfo<'info>,

    /// CHECK: Must belong to this mint. Source of the forced transfer.
    #[account(mut)]
    pub source: AccountInfo<'info>,

    /// CHECK: Must belong to this mint. Destination of the forced transfer.
    #[account(mut)]
    pub destination: AccountInfo<'info>,

    /// CHECK: Token-2022 program.
    #[account(address = spl_token_2022_interface::ID)]
    pub token_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<ForcedTransfer>, amount: u64, decimals: u8) -> Result<()> {
    require!(amount > 0, ErrorCode::ZeroAmount);

    // transfer_checked with the issuer as authority. Token-2022 recognises the
    // PermanentDelegate and allows this even without the holder's signature.
    // The transfer hook also fires; the hook bypasses KYC when authority == issuer.
    invoke(
        &spl_token_2022_interface::instruction::transfer_checked(
            &spl_token_2022_interface::ID,
            ctx.accounts.source.key,
            ctx.accounts.mint.key,
            ctx.accounts.destination.key,
            ctx.accounts.issuer.key,
            &[],
            amount,
            decimals,
        )
        .map_err(|e| ProgramError::from(e))?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.source.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.destination.to_account_info(),
            ctx.accounts.issuer.to_account_info(),
        ],
    )?;

    msg!(
        "Forced transfer: {} base units from {} to {}",
        amount,
        ctx.accounts.source.key,
        ctx.accounts.destination.key
    );

    Ok(())
}
