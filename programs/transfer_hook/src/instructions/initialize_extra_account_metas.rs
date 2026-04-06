use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_instruction;
use spl_tlv_account_resolution::{account::ExtraAccountMeta, state::ExtraAccountMetaList};
use spl_transfer_hook_interface::instruction::ExecuteInstruction;

use crate::constants::EXTRA_ACCOUNT_METAS_SEED;

/// Register the extra accounts the hook reads on every transfer:
///
///   Index 5 — source_kyc_record       PDA ["kyc",    mint, source_token_account]
///   Index 6 — destination_kyc_record  PDA ["kyc",    mint, destination_token_account]
///   Index 7 — token_config            PDA ["config", mint]
///
/// Base account indices used in seed references:
///   0 = source token account
///   1 = mint
///   2 = destination token account
///   3 = authority
///   4 = extra_account_meta_list

#[derive(Accounts)]
pub struct InitializeExtraAccountMetas<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The ExtraAccountMetaList PDA. Seeds: ["extra-account-metas", mint].
    /// CHECK: Allocated below via CPI.
    #[account(
        mut,
        seeds = [EXTRA_ACCOUNT_METAS_SEED, mint.key().as_ref()],
        bump,
    )]
    pub extra_account_meta_list: AccountInfo<'info>,

    /// CHECK: Any valid mint. Caller provides the correct one.
    pub mint: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeExtraAccountMetas>) -> Result<()> {
    let account_metas = vec![
        // Index 5: source KYC — seeds: ["kyc", mint(1), source(0)]
        ExtraAccountMeta::new_with_seeds(
            &[
                spl_tlv_account_resolution::seeds::Seed::Literal {
                    bytes: b"kyc".to_vec(),
                },
                spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 1 },
                spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 0 },
            ],
            false,
            false,
        )?,
        // Index 6: destination KYC — seeds: ["kyc", mint(1), destination(2)]
        ExtraAccountMeta::new_with_seeds(
            &[
                spl_tlv_account_resolution::seeds::Seed::Literal {
                    bytes: b"kyc".to_vec(),
                },
                spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 1 },
                spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 2 },
            ],
            false,
            false,
        )?,
        // Index 7: token config — seeds: ["config", mint(1)]
        ExtraAccountMeta::new_with_seeds(
            &[
                spl_tlv_account_resolution::seeds::Seed::Literal {
                    bytes: b"config".to_vec(),
                },
                spl_tlv_account_resolution::seeds::Seed::AccountKey { index: 1 },
            ],
            false,
            false,
        )?,
    ];

    let account_size = ExtraAccountMetaList::size_of(account_metas.len())
        .map_err(|_| error!(crate::error::HookError::UnknownInstruction))?;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(account_size);

    let mint_key = ctx.accounts.mint.key();
    let bump = ctx.bumps.extra_account_meta_list;
    let signer_seeds: &[&[&[u8]]] =
        &[&[EXTRA_ACCOUNT_METAS_SEED, mint_key.as_ref(), &[bump]]];

    invoke_signed(
        &system_instruction::create_account(
            ctx.accounts.payer.key,
            ctx.accounts.extra_account_meta_list.key,
            lamports,
            account_size as u64,
            &crate::ID,
        ),
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.extra_account_meta_list.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    let mut data = ctx
        .accounts
        .extra_account_meta_list
        .try_borrow_mut_data()?;
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &account_metas)?;

    msg!(
        "ExtraAccountMetaList initialized for mint {} ({} extra accounts)",
        ctx.accounts.mint.key(),
        account_metas.len()
    );

    Ok(())
}
