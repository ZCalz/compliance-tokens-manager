pub mod constants;
pub mod error;
pub mod instructions;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;

declare_id!("FWFApmENh3amqFF3yugs1tMdo9wj5e7LnXS18ppqEH8w");

#[program]
pub mod transfer_hook {
    use super::*;

    /// Register the extra accounts this hook needs on every transfer.
    /// Must be called once after create_mint before any transfers occur.
    pub fn initialize_extra_account_metas(
        ctx: Context<InitializeExtraAccountMetas>,
    ) -> Result<()> {
        instructions::initialize_extra_account_metas::handler(ctx)
    }

    /// Fallback handler — receives all instructions whose discriminator does
    /// not match any declared instruction above.
    ///
    /// Token-2022 calls the hook program using the SPL transfer hook interface
    /// discriminator [105, 37, 101, 197, 75, 251, 102, 26], which is different
    /// from Anchor's standard discriminator for a function named "execute".
    /// This fallback intercepts that call and routes it to our KYC logic.
    pub fn fallback<'info>(
        program_id: &Pubkey,
        accounts: &'info [AccountInfo<'info>],
        data: &[u8],
    ) -> Result<()> {
        // Expect: 8-byte discriminator + 8-byte u64 amount = 16 bytes minimum.
        if data.len() >= 16 && data[..8] == EXECUTE_IX_DISCRIMINATOR {
            let amount = u64::from_le_bytes(
                data[8..16]
                    .try_into()
                    .map_err(|_| error!(error::HookError::UnknownInstruction))?,
            );
            return instructions::execute::handler(program_id, accounts, amount);
        }

        Err(error!(error::HookError::UnknownInstruction))
    }
}
