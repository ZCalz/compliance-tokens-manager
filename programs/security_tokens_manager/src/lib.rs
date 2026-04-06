pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("Ev7Aerjb5EQSAi2wAiBqJBatZSyjCGhqzxDZk6rjEnpS");

#[program]
pub mod security_tokens_manager {
    use super::*;

    /// Create a new Token-2022 security token mint with KYC/AML extensions.
    /// Each call produces an independent mint with its own compliance config.
    pub fn create_mint(ctx: Context<CreateMint>, args: CreateMintArgs) -> Result<()> {
        instructions::create_mint::handler(ctx, args)
    }

    /// Mint tokens to a KYC-verified investor token account.
    pub fn issue_tokens(ctx: Context<IssueTokens>, amount: u64) -> Result<()> {
        instructions::issue_tokens::handler(ctx, amount)
    }

    /// Register on-chain KYC approval for an investor's token account.
    pub fn register_kyc(ctx: Context<RegisterKyc>, args: RegisterKycArgs) -> Result<()> {
        instructions::register_kyc::handler(ctx, args)
    }

    /// Revoke or suspend a KYC approval. Transfers will fail immediately.
    pub fn revoke_kyc(ctx: Context<RevokeKyc>, args: RevokeKycArgs) -> Result<()> {
        instructions::revoke_kyc::handler(ctx, args)
    }

    /// Freeze a token account. Halts all transfers for that account.
    pub fn freeze_account(ctx: Context<FreezeAccount>) -> Result<()> {
        instructions::freeze_account::handler(ctx)
    }

    /// Thaw a frozen token account, restoring transfer ability.
    pub fn thaw_account(ctx: Context<ThawAccount>) -> Result<()> {
        instructions::thaw_account::handler(ctx)
    }

    /// Force-transfer tokens from any account using the PermanentDelegate.
    /// Reserved for regulatory enforcement (court orders, seizure).
    pub fn forced_transfer(
        ctx: Context<ForcedTransfer>,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        instructions::forced_transfer::handler(ctx, amount, decimals)
    }
}
