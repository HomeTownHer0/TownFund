//! Soulbound backer badge (Metaplex Core NFT), minted after success.
//! Runs against the real Metaplex Core program dumped from Devnet.
mod common;

use {
    anchor_lang::prelude::Pubkey,
    common::*,
    mpl_core::{accounts::BaseAssetV1, types::UpdateAuthority},
    solana_keypair::Keypair,
    solana_signer::Signer,
    townfund::error::TownFundError as E,
};

struct World {
    bench: Bench,
    creator: Keypair,
    project: Pubkey,
}

/// Treasury + a Creator-only project (target 10 USDC, 7 days).
fn world() -> World {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    bench.send(initialize_treasury_ix(&creator.pubkey()), &[&creator]).unwrap();
    let p = NewProject { target: 10 * USDC, ..Default::default() };
    bench.send(initialize_project_ix(&creator.pubkey(), &p), &[&creator]).unwrap();
    let project = project_address(&creator.pubkey(), 1);
    World { bench, creator, project }
}

impl World {
    fn backer_pledges(&mut self, usdc: u64) -> Keypair {
        let k = self.bench.wallet();
        let owner = k.pubkey();
        let token = self.bench.usdc_account(&owner, usdc * USDC);
        let ix = pledge_ix(&owner, &self.project, &token, usdc * USDC, None);
        self.bench.send(ix, &[&k]).unwrap();
        k
    }
    fn settle_success(&mut self) {
        self.bench.warp(7 * DAY);
        let creator_owner = self.creator.pubkey();
        let creator_token = self.bench.usdc_account(&creator_owner, 0);
        let caller = self.bench.wallet();
        let ix = settle_ix(&caller.pubkey(), &self.project, Some(creator_token));
        self.bench.send(ix, &[&caller]).unwrap();
    }
    /// Mint a badge for `backer`, paid and triggered by `payer`. Returns the NFT address.
    fn mint_badge(&mut self, payer: &Keypair, backer: &Pubkey) -> Result<Pubkey, Fault> {
        let asset = Keypair::new();
        let ix = mint_badge_ix(&payer.pubkey(), &self.project, backer, &asset.pubkey());
        self.bench.send(ix, &[payer, &asset]).map(|_| asset.pubkey())
    }
    fn nft(&self, asset: &Pubkey) -> BaseAssetV1 {
        let acct = self.bench.svm.get_account(asset).expect("NFT missing");
        assert_eq!(acct.owner, mpl_core::ID, "must be a Metaplex Core asset");
        BaseAssetV1::from_bytes(&acct.data).unwrap()
    }
}

fn assert_fault(res: Result<Pubkey, Fault>, fault: E) {
    let err = res.expect_err("should be refused");
    assert_eq!(err.code, Some(code(fault)), "logs: {:#?}", err.logs);
}

#[test]
fn backer_gets_soulbound_nft_after_success() {
    let mut w = world();
    let backer = w.backer_pledges(10);
    w.settle_success();

    let asset = w.mint_badge(&backer, &backer.pubkey()).unwrap();

    let nft = w.nft(&asset);
    assert_eq!(nft.owner, backer.pubkey(), "NFT is in the backer's wallet");
    assert_eq!(nft.name, "TownFund Backer: Trash clean up Buffalo Bayou Park");
    assert_eq!(nft.uri, "https://github.com/HomeTownHer0/townfund");
    assert_eq!(
        nft.update_authority,
        UpdateAuthority::Address(badge_authority_address()),
        "only TownFund's never-signing PDA could edit it"
    );
    let record = w.bench.svm.get_account(&badge_address(&w.project, &backer.pubkey()));
    assert!(record.is_some(), "badge record exists");
}

#[test]
fn badge_cannot_be_transferred() {
    let mut w = world();
    let backer = w.backer_pledges(10);
    w.settle_success();
    let asset = w.mint_badge(&backer, &backer.pubkey()).unwrap();

    // The owner tries to send the badge to someone else.
    let someone = Pubkey::new_unique();
    let ix = mpl_core::instructions::TransferV1Builder::new()
        .asset(asset)
        .payer(backer.pubkey())
        .authority(Some(backer.pubkey()))
        .new_owner(someone)
        .instruction();
    assert!(w.bench.send(ix, &[&backer]).is_err(), "soulbound: transfer must fail");
    assert_eq!(w.nft(&asset).owner, backer.pubkey(), "still with the backer");
}

#[test]
fn anyone_can_trigger_but_badge_always_goes_to_the_backer() {
    let mut w = world();
    let backer = w.backer_pledges(10);
    w.settle_success();
    let stranger = w.bench.wallet();
    let asset = w.mint_badge(&stranger, &backer.pubkey()).unwrap();
    assert_eq!(w.nft(&asset).owner, backer.pubkey());
}

#[test]
fn every_backer_gets_their_own_badge() {
    let mut w = world();
    let a = w.backer_pledges(6);
    let b = w.backer_pledges(6);
    w.settle_success();
    let na = w.mint_badge(&a, &a.pubkey()).unwrap();
    let nb = w.mint_badge(&b, &b.pubkey()).unwrap();
    assert_ne!(na, nb);
    assert_eq!(w.nft(&na).owner, a.pubkey());
    assert_eq!(w.nft(&nb).owner, b.pubkey());
}

#[test]
fn no_second_badge_for_the_same_backer() {
    let mut w = world();
    let backer = w.backer_pledges(10);
    w.settle_success();
    w.mint_badge(&backer, &backer.pubkey()).unwrap();
    assert!(w.mint_badge(&backer, &backer.pubkey()).is_err(), "one badge per backer per project");
}

#[test]
fn no_badge_before_settle() {
    let mut w = world();
    let backer = w.backer_pledges(10);
    assert_fault(w.mint_badge(&backer, &backer.pubkey()), E::ProjectNotSucceeded);
    w.bench.warp(7 * DAY); // deadline passed but not settled yet
    assert_fault(w.mint_badge(&backer, &backer.pubkey()), E::ProjectNotSucceeded);
}

#[test]
fn no_badge_for_a_failed_project() {
    let mut w = world();
    let backer = w.backer_pledges(5); // under the 10 USDC target
    w.bench.warp(7 * DAY);
    let caller = w.bench.wallet();
    let ix = settle_ix(&caller.pubkey(), &w.project, None);
    w.bench.send(ix, &[&caller]).unwrap(); // -> Failed
    assert_fault(w.mint_badge(&backer, &backer.pubkey()), E::ProjectNotSucceeded);
}

#[test]
fn no_badge_for_someone_who_never_pledged() {
    let mut w = world();
    w.backer_pledges(10);
    w.settle_success();
    let freeloader = w.bench.wallet();
    assert!(w.mint_badge(&freeloader, &freeloader.pubkey()).is_err());
}
