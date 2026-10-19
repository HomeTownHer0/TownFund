//! tip: send USDC straight to the person a project names for tips: on a job,
//! the contractor doing the work; on a Community project, the creator.
//! No fee, not refundable, never enters the vault.
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{constants::*, error::TownFundError, events::Tipped, state::*};

#[derive(Accounts)]
pub struct Tip<'info> {
    pub tipper: Signer<'info>,

    #[account(
        seeds = [PROJECT_SEED, project.creator.as_ref(), &project.project_id.to_le_bytes()],
        bump = project.bump,
    )]
    pub project: Account<'info, Project>,

    #[account(address = USDC_MINT @ TownFundError::WrongMint)]
    pub mint: Account<'info, Mint>,

    #[account(mut, token::mint = mint, token::authority = tipper)]
    pub tipper_token: Account<'info, TokenAccount>,

    /// Interlock: must be a USDC account owned by the project's tip
    /// recipient (see Project::tip_recipient).
    #[account(
        mut,
        token::mint = mint,
        constraint = creator_token.owner == project.tip_recipient() @ TownFundError::WrongPayeeAccount,
    )]
    pub creator_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_tip(ctx: Context<Tip>, amount: u64) -> Result<()> {
    require!(amount > 0, TownFundError::TipZero);

    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.tipper_token.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.creator_token.to_account_info(),
                authority: ctx.accounts.tipper.to_account_info(),
            },
        ),
        amount,
        USDC_DECIMALS,
    )?;

    emit!(Tipped {
        project: ctx.accounts.project.key(),
        tipper: ctx.accounts.tipper.key(),
        creator: ctx.accounts.project.tip_recipient(),
        amount,
    });
    Ok(())
}
