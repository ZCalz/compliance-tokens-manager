// Must match security_tokens_manager/src/constants.rs exactly.
pub const CONFIG_SEED: &[u8] = b"config";
pub const KYC_SEED: &[u8] = b"kyc";

/// Seeds for the ExtraAccountMetaList PDA required by the SPL transfer hook interface.
/// Standard seed from spl-transfer-hook-interface.
pub const EXTRA_ACCOUNT_METAS_SEED: &[u8] = b"extra-account-metas";

/// First 8 bytes of sha256("spl-transfer-hook-interface:execute").
/// Token-2022 uses this discriminator when CPI-calling the hook program.
pub const EXECUTE_IX_DISCRIMINATOR: [u8; 8] = [105, 37, 101, 197, 75, 251, 102, 26];
