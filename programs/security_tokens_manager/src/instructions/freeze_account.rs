use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;

use crate::constants::CONFIG_SEED;
use crate::error::ErrorCode;
use crate::state::TokenConfig;

#[derive(Accounts)]
pub struct FreezeAccount<'info> {
    pub compliance_officer: Signer<'info>,

    #[account(
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump = token_config.bump,
        has_one = compliance_officer @ ErrorCode::NotComplianceOfficer,
        has_one = mint,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// CHECK: Validated via token_config.has_one.
    pub mint: AccountInfo<'info>,

    /// CHECK: Must belong to this mint. Validated by the token program.
    #[account(mut)]
    pub token_account: AccountInfo<'info>,

    /// CHECK: Token-2022 program.
    #[account(address = spl_token_2022_interface::ID)]
    pub token_program: AccountInfo<'info>,
}

pub fn handler(ctx: Context<FreezeAccount>) -> Result<()> {
    invoke(
        &spl_token_2022_interface::instruction::freeze_account(
            &spl_token_2022_interface::ID,
            ctx.accounts.token_account.key,
            ctx.accounts.mint.key,
            ctx.accounts.compliance_officer.key,
            &[],
        )?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.token_account.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.compliance_officer.to_account_info(),
        ],
    )?;

    msg!("Froze token account: {}", ctx.accounts.token_account.key);
    Ok(())
}
