//! pledge: a backer moves USDC into a project's vault.
//! First pledge creates the backer's receipt (and casts their vote on a
//! Community project). Later pledges are top-ups; the vote stays locked.
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{constants::*, error::TownFundError, events::Pledged, state::*};

#[derive(Accounts)]
pub struct MakePledge<'info> {
    #[account(mut)]
    pub backer: Signer<'info>,

    /// The project, verified by its own address recipe.
    #[account(
        mut,
        seeds = [PROJECT_SEED, project.creator.as_ref(), &project.project_id.to_le_bytes()],
        bump = project.bump,
    )]
    pub project: Account<'info, Project>,

    /// Backer's receipt. Created on first pledge, reused for top-ups.
    #[account(
        init_if_needed,
        payer = backer,
        space = 8 + Pledge::INIT_SPACE,
        seeds = [PLEDGE_SEED, project.key().as_ref(), backer.key().as_ref()],
        bump
    )]
    pub pledge: Account<'info, Pledge>,

    #[account(address = USDC_MINT @ TownFundError::WrongMint)]
    pub mint: Account<'info, Mint>,

    /// Where the USDC comes from: must be the backer's own USDC account.
    #[account(mut, token::mint = mint, token::authority = backer)]
    pub backer_token: Account<'info, TokenAccount>,

    /// Where it goes: this project's vault, and no other account.
    #[account(mut, address = project.vault)]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_pledge(ctx: Context<MakePledge>, amount: u64, vote: Option<u8>) -> Result<()> {
    let project = &ctx.accounts.project;

    // ---- Interlocks
    require!(project.status == ProjectStatus::Active, TownFundError::ProjectNotActive);
    let now = Clock::get()?.unix_timestamp;
    require!(now < project.deadline, TownFundError::DeadlinePassed);
    require!(amount >= MIN_PLEDGE, TownFundError::PledgeTooSmall);

    let is_first_pledge = ctx.accounts.pledge.amount == 0;
    let vote_index = check_vote(project, is_first_pledge, vote)?;

    // ---- Move the money: CPI to the Token Program (backer signs)
    token::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.key(),
            TransferChecked {
                from: ctx.accounts.backer_token.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
                authority: ctx.accounts.backer.to_account_info(),
            },
        ),
        amount,
        USDC_DECIMALS,
    )?;

    // ---- Update records (checked math: fault instead of rolling over)
    let pledge = &mut ctx.accounts.pledge;
    if is_first_pledge {
        pledge.version = ACCOUNT_VERSION;
        pledge.bump = ctx.bumps.pledge;
        pledge.project = ctx.accounts.project.key();
        pledge.backer = ctx.accounts.backer.key();
        pledge.vote = vote_index;
        pledge.refunded = false;
    }
    pledge.amount = pledge
        .amount
        .checked_add(amount)
        .ok_or(TownFundError::MathOverflow)?;

    let project = &mut ctx.accounts.project;
    project.total_pledged = project
        .total_pledged
        .checked_add(amount)
        .ok_or(TownFundError::MathOverflow)?;
    if is_first_pledge {
        project.backer_count = project
            .backer_count
            .checked_add(1)
            .ok_or(TownFundError::MathOverflow)?;
        if let Some(i) = vote_index {
            let tally = &mut project.votes[i as usize];
            *tally = tally.checked_add(1).ok_or(TownFundError::MathOverflow)?;
        }
    }

    emit!(Pledged {
        project: project.key(),
        backer: pledge.backer,
        amount,
        backer_total: pledge.amount,
        project_total: project.total_pledged,
        vote: pledge.vote,
    });
    Ok(())
}

/// Community + first pledge: a valid candidate number is required.
/// Everything else (Creator-only, or a top-up): no vote allowed.
fn check_vote(project: &Project, is_first_pledge: bool, vote: Option<u8>) -> Result<Option<u8>> {
    match (project.kind, is_first_pledge, vote) {
        (ProjectKind::Community, true, None) => err!(TownFundError::VoteRequired),
        (ProjectKind::Community, true, Some(i)) => {
            require!((i as usize) < project.candidates.len(), TownFundError::InvalidVote);
            Ok(Some(i))
        }
        (_, _, Some(_)) => err!(TownFundError::VoteNotAllowed),
        (_, _, None) => Ok(None),
    }
}
