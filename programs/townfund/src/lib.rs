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
}
