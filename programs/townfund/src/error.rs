//! Fault code list: every reason an instruction can be refused.
//! On-chain codes start at 6000 (first entry = 6000, next = 6001, ...).
//! Only ever ADD new faults at the bottom, so existing codes never change.
use anchor_lang::prelude::*;

#[error_code]
pub enum TownFundError {
    #[msg("Target must be at least 10 USDC")]
    TargetTooLow,
    #[msg("Duration must be between 1 and 60 days")]
    InvalidDuration,
    #[msg("Title is longer than 64 bytes")]
    TitleTooLong,
    #[msg("A job can name at most one contractor")]
    CreatorProjectHasCandidates,
    #[msg("A Community project needs 2 to 5 contractor candidates")]
    WrongCandidateCount,
    #[msg("The same candidate is listed twice")]
    DuplicateCandidate,
    #[msg("Candidate address is invalid")]
    InvalidCandidate,
    #[msg("Only the TownFund USDC mint is accepted")]
    WrongMint,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Project is not taking pledges")]
    ProjectNotActive,
    #[msg("The deadline has passed")]
    DeadlinePassed,
    #[msg("Pledge must be at least 1 USDC")]
    PledgeTooSmall,
    #[msg("Community projects need a vote on the first pledge")]
    VoteRequired,
    #[msg("Vote not allowed: Creator-only project, or vote already cast")]
    VoteNotAllowed,
    #[msg("Vote is not a valid candidate number")]
    InvalidVote,
    #[msg("Tip must be more than zero")]
    TipZero,
    #[msg("The deadline has not been reached yet")]
    DeadlineNotReached,
    #[msg("Payee's USDC account is required to pay out a successful project")]
    PayeeAccountRequired,
    #[msg("That USDC account does not belong to the payee")]
    WrongPayeeAccount,
    #[msg("Refunds are only open for failed or cancelled projects")]
    RefundsNotOpen,
    #[msg("Badges are only minted for projects that succeeded")]
    ProjectNotSucceeded,
}
