//! settle: anyone can run it once the deadline has passed.
//!  - Target hit: the whole vault pays out once. 5% (rounded down) to the
//!    treasury, the rest to the payee, so no dust is left behind.
//!    Payee = the hired contractor (job), the elected contractor
//!    (Community), or the creator (a job that names no contractor).
//!  - Target missed: marked Failed. Backers then claim refunds.
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, TransferChecked};

use crate::{constants::*, error::TownFundError, events::Settled, state::*};

#[derive(Accounts)]
pub struct Settle<'info> {
    /// Anyone. Pays only the transaction fee.
    pub caller: Signer<'info>,

    #[account(
        mut,
        seeds = [PROJECT_SEED, project.creator.as_ref(), &project.project_id.to_le_bytes()],
        bump = project.bump,
    )]
    pub project: Account<'info, Project>,

    #[account(mut, address = project.vault)]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut, seeds = [TREASURY_SEED], bump = treasury.bump)]
    pub treasury: Account<'info, Treasury>,

    #[account(mut, address = treasury.vault)]
    pub treasury_vault: Account<'info, TokenAccount>,

    #[account(address = USDC_MINT @ TownFundError::WrongMint)]
    pub mint: Account<'info, Mint>,

    /// Payee's USDC account. Required on success, ignored on failure.
    /// Checked in the logic below: must be USDC and owned by the payee.
    #[account(mut)]
    pub payee_token: Option<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_settle(ctx: Context<Settle>) -> Result<()> {
    let project = &ctx.accounts.project;

    // ---- Interlocks
    require!(project.status == ProjectStatus::Active, TownFundError::ProjectNotActive);
    let now = Clock::get()?.unix_timestamp;
    require!(now >= project.deadline, TownFundError::DeadlineNotReached);

    // ---- Target missed: mark Failed, refunds open. No money moves.
    if project.total_pledged < project.target {
        let project = &mut ctx.accounts.project;
        project.status = ProjectStatus::Failed;
        emit!(Settled {
            project: project.key(),
            success: false,
            payee: None,
            winner: None,
            payout: 0,
            fee: 0,
        });
        return Ok(());
    }

    // ---- Target hit: who gets paid?
    let payee = project.payee();
    let winner = match project.kind {
        ProjectKind::Community => project.winning_candidate(),
        ProjectKind::Creator => None,
    };
    let payee_token = ctx
        .accounts
        .payee_token
        .as_ref()
        .ok_or(TownFundError::PayeeAccountRequired)?;
    require!(
        payee_token.mint == USDC_MINT && payee_token.owner == payee,
        TownFundError::WrongPayeeAccount
    );

    // ---- 95 / 5 on the whole vault (includes any overfunding).
    let total = ctx.accounts.vault.amount;
    let fee = total
        .checked_mul(FEE_BPS as u64)
        .ok_or(TownFundError::MathOverflow)?
        / 10_000;
    let payout = total.checked_sub(fee).ok_or(TownFundError::MathOverflow)?;

    // The project PDA signs for its vault (it has no private key; only
    // this program can produce this signature).
    let creator = project.creator;
    let id = project.project_id.to_le_bytes();
    let bump = [project.bump];
    let seeds: &[&[u8]] = &[PROJECT_SEED, creator.as_ref(), &id, &bump];
    let signer = &[seeds];

    let payee_info = payee_token.to_account_info();
    let treasury_vault_info = ctx.accounts.treasury_vault.to_account_info();
    for (to, amount) in [(payee_info, payout), (treasury_vault_info, fee)] {
        if amount == 0 {
            continue;
        }
        token::transfer_checked(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                TransferChecked {
                    from: ctx.accounts.vault.to_account_info(),
                    mint: ctx.accounts.mint.to_account_info(),
                    to,
                    authority: ctx.accounts.project.to_account_info(),
                },
                signer,
            ),
            amount,
            USDC_DECIMALS,
        )?;
    }

    // ---- Records
    let treasury = &mut ctx.accounts.treasury;
    treasury.total_fees = treasury
        .total_fees
        .checked_add(fee)
        .ok_or(TownFundError::MathOverflow)?;
    let project = &mut ctx.accounts.project;
    project.status = ProjectStatus::Succeeded;

    emit!(Settled {
        project: project.key(),
        success: true,
        payee: Some(payee),
        winner,
        payout,
        fee,
    });
    Ok(())
}
