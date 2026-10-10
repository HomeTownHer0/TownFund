//! refund: a backer takes back 100% of their pledge from a failed or
//! cancelled project. No fee. Their receipt is closed and its rent goes
//! back to them too (D-b). A closed receipt can't be refunded twice.
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{constants::*, error::TownFundError, events::Refunded, state::*};

#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,

    #[account(
        mut,
        seeds = [PROJECT_SEED, project.creator.as_ref(), &project.project_id.to_le_bytes()],
        bump = project.bump,
    )]
    pub project: Account<'info, Project>,

    /// This backer's receipt for this project. Closed here; rent -> backer.
    #[account(
        mut,
        seeds = [PLEDGE_SEED, project.key().as_ref(), backer.key().as_ref()],
        bump = pledge.bump,
        close = backer,
    )]
    pub pledge: Account<'info, Pledge>,

    #[account(mut, address = project.vault)]
    pub vault: Account<'info, TokenAccount>,

    #[account(address = USDC_MINT @ TownFundError::WrongMint)]
    pub mint: Account<'info, Mint>,

    /// Refund goes to the backer's own USDC account.
    #[account(mut, token::mint = mint, token::authority = backer)]
    pub backer_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_refund(ctx: Context<Refund>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    require!(ctx.accounts.project.refunds_open(now), TownFundError::RefundsNotOpen);

    let amount = ctx.accounts.pledge.amount;
    let project = &ctx.accounts.project;
    let creator = project.creator;
    let id = project.project_id.to_le_bytes();
    let bump = [project.bump];
    let seeds: &[&[u8]] = &[PROJECT_SEED, creator.as_ref(), &id, &bump];

    token::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.backer_token.to_account_info(),
                authority: ctx.accounts.project.to_account_info(),
            },
            &[seeds],
        ),
        amount,
        USDC_DECIMALS,
    )?;

    let project = &mut ctx.accounts.project;
    project.total_pledged = project
        .total_pledged
        .checked_sub(amount)
        .ok_or(TownFundError::MathOverflow)?;
    ctx.accounts.pledge.refunded = true;

    emit!(Refunded {
        project: project.key(),
        backer: ctx.accounts.backer.key(),
        amount,
    });
    Ok(())
}
