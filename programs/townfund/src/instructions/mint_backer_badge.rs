//! mint_backer_badge: after a project succeeds, mint each backer a
//! soulbound NFT (Metaplex Core) as permanent proof they backed it.
//!
//! - Only for projects that settled successfully (money reached the payee).
//! - Only for wallets with a pledge receipt on that project.
//! - One badge per backer per project (the Badge record can't be made twice).
//! - Soulbound: Metaplex Core's Permanent Freeze plugin, frozen at birth,
//!   with no authority that could ever unfreeze it. It can't be transferred.
//! - Anyone may trigger it (and pays the rent); the NFT always goes to the
//!   backer named on the pledge receipt.
use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateV2CpiBuilder,
    types::{PermanentFreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair},
};

use crate::{constants::*, error::TownFundError, events::BadgeMinted, state::*};

#[derive(Accounts)]
pub struct MintBackerBadge<'info> {
    /// Pays rent for the NFT and the Badge record. Anyone.
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [PROJECT_SEED, project.creator.as_ref(), &project.project_id.to_le_bytes()],
        bump = project.bump,
    )]
    pub project: Account<'info, Project>,

    /// The backer's pledge receipt for this project.
    #[account(
        seeds = [PLEDGE_SEED, project.key().as_ref(), backer.key().as_ref()],
        bump = pledge.bump,
    )]
    pub pledge: Account<'info, Pledge>,

    /// CHECK: the wallet that receives the badge. Pinned by the pledge
    /// receipt's seeds above, so it must be the real backer.
    pub backer: UncheckedAccount<'info>,

    /// One per (project, backer). `init` fails if it exists: no double mints.
    #[account(
        init,
        payer = payer,
        space = 8 + Badge::INIT_SPACE,
        seeds = [BADGE_SEED, project.key().as_ref(), backer.key().as_ref()],
        bump
    )]
    pub badge: Account<'info, Badge>,

    /// The new NFT account (a fresh keypair, signs once at creation).
    #[account(mut)]
    pub asset: Signer<'info>,

    /// CHECK: PDA set as the NFT's update authority. Never signs, so the
    /// badge can never be edited.
    #[account(seeds = [BADGE_AUTHORITY_SEED], bump)]
    pub badge_authority: UncheckedAccount<'info>,

    /// CHECK: must be the Metaplex Core program.
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_mint_backer_badge(ctx: Context<MintBackerBadge>) -> Result<()> {
    let project = &ctx.accounts.project;
    require!(
        project.status == ProjectStatus::Succeeded,
        TownFundError::ProjectNotSucceeded
    );

    let name = format!("TownFund Backer: {}", project.title);

    // ---- CPI to Metaplex Core: create the soulbound NFT, owned by the backer
    CreateV2CpiBuilder::new(&ctx.accounts.mpl_core_program.to_account_info())
        .asset(&ctx.accounts.asset.to_account_info())
        .payer(&ctx.accounts.payer.to_account_info())
        .owner(Some(&ctx.accounts.backer.to_account_info()))
        .update_authority(Some(&ctx.accounts.badge_authority.to_account_info()))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .name(name)
        .uri(BADGE_URI.to_string())
        .plugins(vec![PluginAuthorityPair {
            plugin: Plugin::PermanentFreezeDelegate(PermanentFreezeDelegate { frozen: true }),
            authority: Some(PluginAuthority::None),
        }])
        .invoke()?;

    // ---- Record it (blocks a second badge for this backer + project)
    let badge = &mut ctx.accounts.badge;
    badge.set_inner(Badge {
        version: ACCOUNT_VERSION,
        bump: ctx.bumps.badge,
        project: project.key(),
        backer: ctx.accounts.backer.key(),
        asset: ctx.accounts.asset.key(),
        reserved: [0; 32],
    });

    emit!(BadgeMinted {
        project: project.key(),
        backer: badge.backer,
        asset: badge.asset,
    });
    Ok(())
}
