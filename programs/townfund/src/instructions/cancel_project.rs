//! cancel_project: the creator stops a project before its deadline.
//! Every backer can then claim a 100% refund.
use anchor_lang::prelude::*;

use crate::{constants::*, error::TownFundError, events::ProjectCancelled, state::*};

#[derive(Accounts)]
pub struct CancelProject<'info> {
    pub creator: Signer<'info>,

    /// `seeds` uses the signer's key, so only the real creator matches.
    #[account(
        mut,
        seeds = [PROJECT_SEED, creator.key().as_ref(), &project.project_id.to_le_bytes()],
        bump = project.bump,
    )]
    pub project: Account<'info, Project>,
}

pub fn handle_cancel_project(ctx: Context<CancelProject>) -> Result<()> {
    let project = &mut ctx.accounts.project;
    require!(project.status == ProjectStatus::Active, TownFundError::ProjectNotActive);
    let now = Clock::get()?.unix_timestamp;
    require!(now < project.deadline, TownFundError::DeadlinePassed);

    project.status = ProjectStatus::Cancelled;
    emit!(ProjectCancelled {
        project: project.key(),
        total_pledged: project.total_pledged,
    });
    Ok(())
}
