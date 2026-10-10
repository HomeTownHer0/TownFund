//! initialize_treasury: create TownFund's fee account. Runs once, ever.
//! Anyone may call it; there is nothing to choose (no owner, no settings),
//! so calling it first gives no power. A second call fails (`init`).
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
    constants::*, error::TownFundError, events::TreasuryInitialized, state::Treasury,
};

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Treasury::INIT_SPACE,
        seeds = [TREASURY_SEED],
        bump
    )]
    pub treasury: Account<'info, Treasury>,

    #[account(address = USDC_MINT @ TownFundError::WrongMint)]
    pub mint: Account<'info, Mint>,

    /// Fee vault. Authority = treasury PDA (no private key).
    #[account(
        init,
        payer = payer,
        seeds = [TREASURY_VAULT_SEED],
        bump,
        token::mint = mint,
        token::authority = treasury,
    )]
    pub treasury_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    treasury.set_inner(Treasury {
        version: ACCOUNT_VERSION,
        bump: ctx.bumps.treasury,
        vault_bump: ctx.bumps.treasury_vault,
        mint: ctx.accounts.mint.key(),
        vault: ctx.accounts.treasury_vault.key(),
        door_authority: None,
        total_fees: 0,
        reserved: [0; 64],
    });
    emit!(TreasuryInitialized {
        treasury: treasury.key(),
        vault: treasury.vault,
    });
    Ok(())
}
