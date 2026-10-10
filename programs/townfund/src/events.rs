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
