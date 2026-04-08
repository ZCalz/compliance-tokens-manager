use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::system_instruction;
use spl_token_2022_interface::extension::ExtensionType;
use spl_token_2022_interface::state::Mint;

use crate::constants::CONFIG_SEED;
use crate::error::ErrorCode;
use crate::state::{KycLevel, TokenConfig};

const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

const MAX_NAME_LEN: usize = 64;
const MAX_SYMBOL_LEN: usize = 10;
const MAX_URI_LEN: usize = 256;

// ---------------------------------------------------------------------------
// Arguments
// ---------------------------------------------------------------------------

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateMintArgs {
    pub name: String,
    pub symbol: String,
    /// URL pointing to JSON metadata (offering docs, ISIN, legal disclosures).
    pub uri: String,
    pub decimals: u8,
    pub required_kyc_level: KycLevel,
    /// ISO 3166-1 alpha-2 country codes, e.g. [[b'U', b'S'], [b'D', b'E']].
    pub jurisdiction_allowlist: Vec<[u8; 2]>,
    /// AML velocity limit per UTC day, in token base units. 0 = no limit.
    pub daily_transfer_limit: u64,
    /// AML velocity limit per UTC month, in token base units. 0 = no limit.
    pub monthly_transfer_limit: u64,
}

// ---------------------------------------------------------------------------
// Accounts
// ---------------------------------------------------------------------------

#[derive(Accounts)]
#[instruction(args: CreateMintArgs)]
pub struct CreateMint<'info> {
    /// The entity issuing this security token.
    #[account(mut)]
    pub issuer: Signer<'info>,

    /// Fresh keypair for the new mint. Must be a signer.
    /// Allocated manually below because its size depends on metadata length.
    /// CHECK: New mint keypair — verified as signer.
    #[account(mut, signer)]
    pub mint: AccountInfo<'info>,

    /// Per-mint compliance configuration PDA.
    #[account(
        init,
        payer = issuer,
        space = 8 + TokenConfig::INIT_SPACE,
        seeds = [CONFIG_SEED, mint.key().as_ref()],
        bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// Wallet that may register / revoke KYC records for this token.
    /// CHECK: Role is stored in token_config.
    pub kyc_operator: AccountInfo<'info>,

    /// Wallet that may freeze / thaw token accounts for this token.
    /// CHECK: Role is stored in token_config.
    pub compliance_officer: AccountInfo<'info>,

    /// The deployed transfer hook program enforcing KYC on every transfer.
    /// CHECK: Caller is responsible for passing the correct hook program.
    pub transfer_hook_program: AccountInfo<'info>,

    /// Token-2022 program.
    /// CHECK: Verified by address constraint.
    #[account(address = spl_token_2022_interface::ID)]
    pub token_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

// ---------------------------------------------------------------------------
// Handler
// ---------------------------------------------------------------------------

pub fn handler(ctx: Context<CreateMint>, args: CreateMintArgs) -> Result<()> {
    require!(args.name.len() <= MAX_NAME_LEN, ErrorCode::NameTooLong);
    require!(
        args.symbol.len() <= MAX_SYMBOL_LEN,
        ErrorCode::SymbolTooLong
    );
    require!(args.uri.len() <= MAX_URI_LEN, ErrorCode::UriTooLong);
    require!(
        args.jurisdiction_allowlist.len() <= 30,
        ErrorCode::TooManyJurisdictions
    );

    // -----------------------------------------------------------------------
    // 1. Calculate total mint account size.
    // -----------------------------------------------------------------------
    let fixed_extensions = [
        ExtensionType::TransferHook,
        ExtensionType::DefaultAccountState,
        ExtensionType::PermanentDelegate,
        ExtensionType::MintCloseAuthority,
        ExtensionType::MetadataPointer,
    ];
    let base_size = ExtensionType::try_calculate_account_len::<Mint>(&fixed_extensions)
        .map_err(|_| error!(ErrorCode::InvalidMintSize))?;

    // TokenMetadata TLV layout:
    //   TLV header         :  4 bytes (2 type + 2 length)
    //   update_authority   : 32 (OptionalNonZeroPubkey)
    //   mint               : 32
    //   name               :  4 + len
    //   symbol             :  4 + len
    //   uri                :  4 + len
    //   additional_metadata:  4 (empty Vec prefix)
    let metadata_data_len =
        32 + 32 + 4 + args.name.len() + 4 + args.symbol.len() + 4 + args.uri.len() + 4;
    let total_size = base_size + 4 + metadata_data_len;

    // -----------------------------------------------------------------------
    // 2. Allocate the mint account.
    // -----------------------------------------------------------------------
    let rent = &ctx.accounts.rent;
    let lamports = rent.minimum_balance(total_size);

    invoke(
        &system_instruction::create_account(
            ctx.accounts.issuer.key,
            ctx.accounts.mint.key,
            lamports,
            total_size as u64,
            &spl_token_2022_interface::ID,
        ),
        &[
            ctx.accounts.issuer.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // -----------------------------------------------------------------------
    // 3. Initialize extensions (must precede initialize_mint2).
    // -----------------------------------------------------------------------

    // TransferHook
    invoke(
        &spl_token_2022_interface::extension::transfer_hook::instruction::initialize(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            Some(*ctx.accounts.issuer.key),
            Some(*ctx.accounts.transfer_hook_program.key),
        )
        .map_err(|e| ProgramError::from(e))?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    // DefaultAccountState — all new token accounts start Frozen
    invoke(
        &spl_token_2022_interface::extension::default_account_state::instruction::initialize_default_account_state(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            &spl_token_2022_interface::state::AccountState::Frozen,
        )
        .map_err(|e| ProgramError::from(e))?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    // PermanentDelegate — issuer can force-transfer or burn from any account
    invoke(
        &spl_token_2022_interface::instruction::initialize_permanent_delegate(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            ctx.accounts.issuer.key,
        )
        .map_err(|e| ProgramError::from(e))?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    // MintCloseAuthority — issuer can close the mint at end-of-life
    invoke(
        &spl_token_2022_interface::instruction::initialize_mint_close_authority(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            Some(ctx.accounts.issuer.key),
        )
        .map_err(|e| ProgramError::from(e))?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    // MetadataPointer — points to the mint itself for on-chain metadata
    invoke(
        &spl_token_2022_interface::extension::metadata_pointer::instruction::initialize(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            Some(*ctx.accounts.issuer.key),
            Some(*ctx.accounts.mint.key),
        )
        .map_err(|e| ProgramError::from(e))?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    // -----------------------------------------------------------------------
    // 4. Initialize the mint.
    // -----------------------------------------------------------------------
    invoke(
        &spl_token_2022_interface::instruction::initialize_mint2(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            ctx.accounts.issuer.key,
            Some(ctx.accounts.issuer.key),
            args.decimals,
        )
        .map_err(|e| ProgramError::from(e))?,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    // -----------------------------------------------------------------------
    // 5. Initialize TokenMetadata (must follow initialize_mint2).
    // -----------------------------------------------------------------------
    invoke(
        &spl_token_metadata_interface::instruction::initialize(
            &spl_token_2022_interface::ID,
            ctx.accounts.mint.key,
            ctx.accounts.issuer.key,
            ctx.accounts.mint.key,
            ctx.accounts.issuer.key,
            args.name.clone(),
            args.symbol.clone(),
            args.uri.clone(),
        ),
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
            ctx.accounts.issuer.to_account_info(),
        ],
    )?;

    // -----------------------------------------------------------------------
    // 6. Save compliance configuration.
    // -----------------------------------------------------------------------
    let token_config = &mut ctx.accounts.token_config;
    token_config.mint = *ctx.accounts.mint.key;
    token_config.issuer = *ctx.accounts.issuer.key;
    token_config.kyc_operator = *ctx.accounts.kyc_operator.key;
    token_config.compliance_officer = *ctx.accounts.compliance_officer.key;
    token_config.transfer_hook_program = *ctx.accounts.transfer_hook_program.key;
    token_config.required_kyc_level = args.required_kyc_level;
    token_config.jurisdiction_allowlist = args.jurisdiction_allowlist;
    token_config.daily_transfer_limit = args.daily_transfer_limit;
    token_config.monthly_transfer_limit = args.monthly_transfer_limit;
    token_config.bump = ctx.bumps.token_config;

    msg!(
        "Security token created: {} ({}) | hook: {}",
        args.name,
        args.symbol,
        ctx.accounts.transfer_hook_program.key
    );

    Ok(())
}
