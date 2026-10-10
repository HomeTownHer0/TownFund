//! Rated values: the fixed limits of the machine.
//! USDC has 6 decimals, so 1 USDC = 1_000_000 base units.
use anchor_lang::prelude::*;

/// The only token this program accepts: Circle's official Devnet USDC
/// (classic SPL Token, 6 decimals). Get it free at faucet.circle.com.
/// Tests create a mint at this exact address. Change before any Mainnet use.
pub const USDC_MINT: Pubkey = pubkey!("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU");
#[constant]
pub const USDC_DECIMALS: u8 = 6;

// PDA seeds (the "address recipe" for each program-owned account)
#[constant]
pub const PROJECT_SEED: &[u8] = b"project";
#[constant]
pub const VAULT_SEED: &[u8] = b"vault";
#[constant]
pub const PLEDGE_SEED: &[u8] = b"pledge";
#[constant]
pub const TREASURY_SEED: &[u8] = b"treasury";
#[constant]
pub const TREASURY_VAULT_SEED: &[u8] = b"treasury_vault";
#[constant]
pub const BADGE_SEED: &[u8] = b"badge";
/// PDA that is the update authority of every badge NFT. It never signs
/// an update, so badges can never be edited.
#[constant]
pub const BADGE_AUTHORITY_SEED: &[u8] = b"badge_authority";

/// Link stored in every backer badge NFT.
pub const BADGE_URI: &str = "https://github.com/HomeTownHer0/townfund";

// Money limits
#[constant]
pub const MIN_PLEDGE: u64 = 1_000_000; // 1 USDC
#[constant]
pub const MIN_TARGET: u64 = 10_000_000; // 10 USDC
#[constant]
pub const FEE_BPS: u16 = 500; // 5% to treasury (basis points: 10_000 = 100%)

// Time limits (seconds)
#[constant]
pub const MIN_DURATION: i64 = 86_400; // 1 day
#[constant]
pub const MAX_DURATION: i64 = 60 * 86_400; // 60 days
/// Demo build only (`--features demo`): any length from 10 seconds to
/// 10 minutes, so a full create -> pledge -> settle run fits on stage.
#[constant]
pub const DEMO_MIN_DURATION: i64 = 10; // 10 seconds
#[constant]
pub const DEMO_MAX_DURATION: i64 = 600; // 10 minutes

// Project shape (internal sizes; not published in the IDL)
pub const MAX_TITLE_LEN: usize = 64; // bytes
pub const MIN_CANDIDATES: usize = 2;
pub const MAX_CANDIDATES: usize = 5;

/// Layout version stamped on every stored account (D-d).
#[constant]
pub const ACCOUNT_VERSION: u8 = 1;
