//! initialize_project: create a fundraiser and its empty vault.
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{constants::*, error::TownFundError, events::ProjectCreated, state::*};

// ---- I/O list: every account this instruction touches, with its interlocks
#[derive(Accounts)]
#[instruction(project_id: u64)]
pub struct InitializeProject<'info> {
    /// Signs and pays rent for the two new accounts.
    #[account(mut)]
    pub creator: Signer<'info>,

    /// New Project account at ["project", creator, project_id].
    /// `init` faults if it already exists, so a project_id can't be reused.
    #[account(
        init,
        payer = creator,
        space = 8 + Project::INIT_SPACE,
        seeds = [PROJECT_SEED, creator.key().as_ref(), &project_id.to_le_bytes()],
        bump
    )]
    pub project: Account<'info, Project>,

    /// Interlock: only our USDC mint is accepted.
    #[account(address = USDC_MINT @ TownFundError::WrongMint)]
    pub mint: Account<'info, Mint>,

    /// New vault at ["vault", project]. Its authority is the project PDA,
    /// which has no private key, so only this program can move funds out.
    #[account(
        init,
        payer = creator,
        seeds = [VAULT_SEED, project.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = project,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

// ---- Logic: interlock checks first, then write the record
pub fn handle_initialize_project(
    ctx: Context<InitializeProject>,
    project_id: u64,
    kind: ProjectKind,
    target: u64,
    duration_secs: i64,
    title: String,
    candidates: Vec<Pubkey>,
) -> Result<()> {
    require!(title.len() <= MAX_TITLE_LEN, TownFundError::TitleTooLong);
    require!(target >= MIN_TARGET, TownFundError::TargetTooLow);
    require!(is_valid_duration(duration_secs), TownFundError::InvalidDuration);
    check_candidates(kind, &candidates)?;

    let now = Clock::get()?.unix_timestamp;
    let deadline = now
        .checked_add(duration_secs)
        .ok_or(TownFundError::MathOverflow)?;

    let project = &mut ctx.accounts.project;
    project.set_inner(Project {
        version: ACCOUNT_VERSION,
        bump: ctx.bumps.project,
        vault_bump: ctx.bumps.vault,
        kind,
        status: ProjectStatus::Active,
        creator: ctx.accounts.creator.key(),
        project_id,
        mint: ctx.accounts.mint.key(),
        vault: ctx.accounts.vault.key(),
        target,
        total_pledged: 0,
        backer_count: 0,
        created_at: now,
        deadline,
        title,
        candidates: candidates.clone(),
        votes: [0; MAX_CANDIDATES],
        reserved: [0; 64],
    });

    emit!(ProjectCreated {
        project: project.key(),
        creator: project.creator,
        project_id,
        kind,
        target,
        deadline,
        candidate_count: candidates.len() as u8,
    });
    Ok(())
}

/// 1-60 days. A `demo` build also accepts 10 seconds to 10 minutes.
fn is_valid_duration(duration_secs: i64) -> bool {
    (MIN_DURATION..=MAX_DURATION).contains(&duration_secs)
        || (cfg!(feature = "demo")
            && (DEMO_MIN_DURATION..=DEMO_MAX_DURATION).contains(&duration_secs))
}

/// Creator-only: no candidates. Community: 2-5 distinct, real addresses.
fn check_candidates(kind: ProjectKind, candidates: &[Pubkey]) -> Result<()> {
    match kind {
        ProjectKind::Creator => {
            require!(candidates.is_empty(), TownFundError::CreatorProjectHasCandidates);
        }
        ProjectKind::Community => {
            require!(
                (MIN_CANDIDATES..=MAX_CANDIDATES).contains(&candidates.len()),
                TownFundError::WrongCandidateCount
            );
            for (i, candidate) in candidates.iter().enumerate() {
                require!(*candidate != Pubkey::default(), TownFundError::InvalidCandidate);
                require!(
                    !candidates[..i].contains(candidate),
                    TownFundError::DuplicateCandidate
                );
            }
        }
    }
    Ok(())
}
