//! Machine log: events the website reads to show history (D-d).
use anchor_lang::prelude::*;

use crate::state::ProjectKind;

#[event]
pub struct ProjectCreated {
    pub project: Pubkey,
    pub creator: Pubkey,
    pub project_id: u64,
    pub kind: ProjectKind,
    pub target: u64,
    pub deadline: i64,
    pub candidate_count: u8,
}

#[event]
pub struct TreasuryInitialized {
    pub treasury: Pubkey,
    pub vault: Pubkey,
}

#[event]
pub struct Pledged {
    pub project: Pubkey,
    pub backer: Pubkey,
    pub amount: u64,
    pub backer_total: u64,
    pub project_total: u64,
    pub vote: Option<u8>,
}

#[event]
pub struct Tipped {
    pub project: Pubkey,
    pub tipper: Pubkey,
    pub creator: Pubkey,
    pub amount: u64,
}

#[event]
pub struct Settled {
    pub project: Pubkey,
    /// true = target hit and paid out; false = target missed (refunds open).
    pub success: bool,
    /// Who was paid (None if the project failed).
    pub payee: Option<Pubkey>,
    /// Winning candidate number (Community success only).
    pub winner: Option<u8>,
    pub payout: u64,
    pub fee: u64,
}

#[event]
pub struct ProjectCancelled {
    pub project: Pubkey,
    pub total_pledged: u64,
}

#[event]
pub struct Refunded {
    pub project: Pubkey,
    pub backer: Pubkey,
    pub amount: u64,
}
