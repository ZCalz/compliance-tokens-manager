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

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        initialize::handler(ctx)
    }
}
