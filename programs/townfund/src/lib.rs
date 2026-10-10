pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv");

#[program]
pub mod townfund {
    use super::*;

    /// Create a fundraiser and its empty vault.
    pub fn initialize_project(
        ctx: Context<InitializeProject>,
        project_id: u64,
        kind: ProjectKind,
        target: u64,
        duration_secs: i64,
        title: String,
        candidates: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::initialize_project::handle_initialize_project(
            ctx,
            project_id,
            kind,
            target,
            duration_secs,
            title,
            candidates,
        )
    }

    /// Create TownFund's fee account. Runs once, ever.
    pub fn initialize_treasury(ctx: Context<InitializeTreasury>) -> Result<()> {
        instructions::initialize_treasury::handle_initialize_treasury(ctx)
    }

    /// Pledge USDC to a project. Community projects: `vote` = candidate
    /// number on the first pledge; must be None for top-ups.
    pub fn pledge(ctx: Context<MakePledge>, amount: u64, vote: Option<u8>) -> Result<()> {
        instructions::pledge::handle_pledge(ctx, amount, vote)
    }

    /// Tip the creator directly. No fee, not refundable.
    pub fn tip(ctx: Context<Tip>, amount: u64) -> Result<()> {
        instructions::tip::handle_tip(ctx, amount)
    }

    /// After the deadline, anyone: pay out 95/5 if target hit, else mark Failed.
    pub fn settle(ctx: Context<Settle>) -> Result<()> {
        instructions::settle::handle_settle(ctx)
    }

    /// Creator stops the project before its deadline. Refunds open.
    pub fn cancel_project(ctx: Context<CancelProject>) -> Result<()> {
        instructions::cancel_project::handle_cancel_project(ctx)
    }

    /// Backer takes back 100% from a failed or cancelled project.
    pub fn refund(ctx: Context<Refund>) -> Result<()> {
        instructions::refund::handle_refund(ctx)
    }

    /// After success: mint the backer a soulbound Metaplex Core badge NFT.
    pub fn mint_backer_badge(ctx: Context<MintBackerBadge>) -> Result<()> {
        instructions::mint_backer_badge::handle_mint_backer_badge(ctx)
    }
}
