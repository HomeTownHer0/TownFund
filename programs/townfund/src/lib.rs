pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("HaLAfSdw1W39jxiH9MBEdeAqiAzwgDH4K6xBcU6AGBgf");

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
}
