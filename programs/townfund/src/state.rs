//! Parts list (BOM): the layout of every stored account.
use anchor_lang::prelude::*;

/// Who gets paid when a project succeeds.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum ProjectKind {
    /// No vote. The creator is paid.
    Creator,
    /// Backers elect one of 2-5 contractor candidates, who is paid.
    Community,
}

/// Where a project is in its life.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum ProjectStatus {
    /// Taking pledges until the deadline.
    Active,
    /// Creator cancelled before the deadline. Backers claim refunds.
    Cancelled,
    /// Target hit and paid out 95/5.
    Succeeded,
    /// Target missed. Backers claim refunds.
    Failed,
}

/// One fundraiser. Address (PDA) = ["project", creator, project_id].
#[account]
#[derive(InitSpace, Debug)]
pub struct Project {
    /// Layout version (D-d). Starts at 1.
    pub version: u8,
    /// PDA bump for this account.
    pub bump: u8,
    /// PDA bump for the vault token account.
    pub vault_bump: u8,
    pub kind: ProjectKind,
    pub status: ProjectStatus,
    /// Wallet that created the project (paid if Creator-only).
    pub creator: Pubkey,
    /// Creator's own number for this project, so one creator can run many.
    pub project_id: u64,
    /// Token accepted (always USDC_MINT).
    pub mint: Pubkey,
    /// Token account holding pledges. Only this program can move funds out.
    pub vault: Pubkey,
    /// Goal in USDC base units.
    pub target: u64,
    /// Running total pledged (base units).
    pub total_pledged: u64,
    /// Number of distinct backer wallets.
    pub backer_count: u32,
    /// Unix seconds.
    pub created_at: i64,
    /// Unix seconds. Pledges close at this time; settle opens.
    pub deadline: i64,
    #[max_len(64)]
    pub title: String,
    /// Contractor candidates (Community only; empty for Creator-only).
    #[max_len(5)]
    pub candidates: Vec<Pubkey>,
    /// Vote count per candidate index.
    pub votes: [u32; 5],
    /// Spare room for future fields (D-d). Always zero for now.
    pub reserved: [u8; 64],
}
