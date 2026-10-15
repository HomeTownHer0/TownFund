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
    /// Layout version. Starts at 1.
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
    /// Spare room for future fields. Always zero for now.
    pub reserved: [u8; 64],
}

/// TownFund's fee account. One per program. Address = ["treasury"].
/// Program-owned (PDA, no private key). No withdraw path in V1.
#[account]
#[derive(InitSpace, Debug)]
pub struct Treasury {
    pub version: u8,
    pub bump: u8,
    pub vault_bump: u8,
    /// Token accepted (always USDC_MINT).
    pub mint: Pubkey,
    /// Token account holding fees, at ["treasury_vault"].
    pub vault: Pubkey,
    /// Reserved for the future "door" (multisig or vote) that can spend
    /// treasury funds. Always None in V1: nothing can spend.
    pub door_authority: Option<Pubkey>,
    /// Lifetime fees received (base units).
    pub total_fees: u64,
    /// Spare room.
    pub reserved: [u8; 64],
}

/// One backer's receipt for one project. Address = ["pledge", project, backer].
#[account]
#[derive(InitSpace, Debug)]
pub struct Pledge {
    pub version: u8,
    pub bump: u8,
    pub project: Pubkey,
    pub backer: Pubkey,
    /// Total pledged by this backer, including top-ups (base units).
    pub amount: u64,
    /// Candidate index voted for (Community only). Locked after first pledge.
    pub vote: Option<u8>,
    /// Set when the backer has taken their refund.
    pub refunded: bool,
    /// Spare room.
    pub reserved: [u8; 32],
}

impl Project {
    /// Community: candidate with the most votes. Tie -> lowest number.
    pub fn winning_candidate(&self) -> Option<u8> {
        if self.candidates.is_empty() {
            return None;
        }
        let mut best = 0usize;
        for i in 1..self.candidates.len() {
            if self.votes[i] > self.votes[best] {
                best = i;
            }
        }
        Some(best as u8)
    }

    /// Refunds are open when the project was cancelled, failed at settle,
    /// or is past its deadline without reaching target (no settle needed).
    pub fn refunds_open(&self, now: i64) -> bool {
        match self.status {
            ProjectStatus::Cancelled | ProjectStatus::Failed => true,
            ProjectStatus::Active => now >= self.deadline && self.total_pledged < self.target,
            ProjectStatus::Succeeded => false,
        }
    }
}

/// Record that a backer's soulbound badge was minted for a project.
/// Address = ["badge", project, backer]; its existence blocks a second mint.
#[account]
#[derive(InitSpace, Debug)]
pub struct Badge {
    pub version: u8,
    pub bump: u8,
    pub project: Pubkey,
    pub backer: Pubkey,
    /// The Metaplex Core asset (the NFT itself).
    pub asset: Pubkey,
    /// Spare room.
    pub reserved: [u8; 32],
}
