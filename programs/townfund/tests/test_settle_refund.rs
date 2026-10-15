//! Slice 3: settle, refund, cancel_project. Includes the two Demo Day
//! "must" scenarios end to end.
mod common;

use {
    anchor_lang::prelude::Pubkey,
    common::*,
    solana_keypair::Keypair,
    solana_signer::Signer,
    townfund::{error::TownFundError as E, ProjectKind, ProjectStatus},
};

struct World {
    bench: Bench,
    creator: Keypair,
    project: Pubkey,
}

/// Treasury + one project, ready for pledges.
fn world(p: NewProject) -> World {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    bench.send(initialize_treasury_ix(&creator.pubkey()), &[&creator]).unwrap();
    bench.send(initialize_project_ix(&creator.pubkey(), &p), &[&creator]).unwrap();
    let project = project_address(&creator.pubkey(), p.project_id);
    World { bench, creator, project }
}

impl World {
    fn backer(&mut self, usdc: u64) -> (Keypair, Pubkey) {
        let k = self.bench.wallet();
        let owner = k.pubkey();
        let token = self.bench.usdc_account(&owner, usdc * USDC);
        (k, token)
    }
    fn pledge(&mut self, b: &(Keypair, Pubkey), amount: u64, vote: Option<u8>) {
        let ix = pledge_ix(&b.0.pubkey(), &self.project, &b.1, amount, vote);
        self.bench.send(ix, &[&b.0]).unwrap();
    }
    fn settle(&mut self, payee_token: Option<Pubkey>) -> Result<(), Fault> {
        let caller = self.bench.wallet(); // anyone can settle
        let ix = settle_ix(&caller.pubkey(), &self.project, payee_token);
        self.bench.send(ix, &[&caller])
    }
    fn refund(&mut self, b: &(Keypair, Pubkey)) -> Result<(), Fault> {
        let ix = refund_ix(&b.0.pubkey(), &self.project, &b.1);
        self.bench.send(ix, &[&b.0])
    }
    fn cancel(&mut self, who: &Keypair) -> Result<(), Fault> {
        let ix = cancel_ix(&who.pubkey(), &self.project);
        self.bench.send(ix, &[who])
    }
    fn past_deadline(&mut self) {
        self.bench.warp(7 * DAY);
    }
    fn vault(&self) -> u64 {
        self.bench.balance(vault_address(&self.project))
    }
    fn treasury_balance(&self) -> u64 {
        self.bench.balance(treasury_vault_address())
    }
    fn status(&self) -> ProjectStatus {
        self.bench.project(self.project).status
    }
}

fn assert_fault(res: Result<(), Fault>, fault: E) {
    let err = res.expect_err("should be refused");
    assert_eq!(err.code, Some(code(fault)), "logs: {:#?}", err.logs);
}

// ======================================================= Demo Day "must" #1
#[test]
fn demo_creator_project_succeeds_overfunded_with_tip_and_settles_95_5() {
    let mut w = world(NewProject::default()); // target 100 USDC, 7 days
    let creator_owner = w.creator.pubkey();
    let creator_token = w.bench.usdc_account(&creator_owner, 0);

    let alice = w.backer(100);
    let bob = w.backer(100);
    w.pledge(&alice, 60 * USDC, None);
    w.pledge(&bob, 50 * USDC, None); // 110 total: overfunded

    // A tip goes straight to the creator.
    let fan = w.backer(10);
    let ix = tip_ix(&fan.0.pubkey(), &w.project, &fan.1, &creator_token, 5 * USDC);
    w.bench.send(ix, &[&fan.0]).unwrap();
    assert_eq!(w.bench.balance(creator_token), 5 * USDC);

    w.past_deadline();
    w.settle(Some(creator_token)).unwrap();

    // 110 * 5% = 5.5 to treasury; 104.5 to creator (+ the 5 tip).
    assert_eq!(w.treasury_balance(), 5_500_000);
    assert_eq!(w.bench.balance(creator_token), 5 * USDC + 104_500_000);
    assert_eq!(w.vault(), 0, "whole vault paid out, no dust");
    assert_eq!(w.status(), ProjectStatus::Succeeded);
    assert_eq!(w.bench.treasury().total_fees, 5_500_000);
}

// ======================================================= Demo Day "must" #2
#[test]
fn demo_project_fails_and_every_backer_gets_100_percent_back() {
    let mut w = world(NewProject::default()); // target 100
    let alice = w.backer(100);
    let bob = w.backer(100);
    w.pledge(&alice, 30 * USDC, None);
    w.pledge(&bob, 20 * USDC, None); // 50 < 100: will fail

    w.past_deadline();
    w.settle(None).unwrap(); // no payee needed when failing
    assert_eq!(w.status(), ProjectStatus::Failed);
    assert_eq!(w.treasury_balance(), 0, "no fee on failure");

    w.refund(&alice).unwrap();
    w.refund(&bob).unwrap();
    assert_eq!(w.bench.balance(alice.1), 100 * USDC);
    assert_eq!(w.bench.balance(bob.1), 100 * USDC);
    assert_eq!(w.vault(), 0);
    // Receipts closed, rent returned.
    assert!(w.bench.svm.get_account(&pledge_address(&w.project, &alice.0.pubkey())).is_none());
}

// =================================================================== settle

#[test]
fn fee_is_rounded_down_and_nothing_is_left_behind() {
    let mut w = world(NewProject { target: 10 * USDC, ..Default::default() });
    let creator_owner = w.creator.pubkey();
    let creator_token = w.bench.usdc_account(&creator_owner, 0);
    let a = w.backer(50);
    w.pledge(&a, 33_333_333, None); // 33.333333 USDC
    w.past_deadline();
    w.settle(Some(creator_token)).unwrap();
    assert_eq!(w.treasury_balance(), 1_666_666); // floor(33_333_333 * 5%)
    assert_eq!(w.bench.balance(creator_token), 31_666_667);
    assert_eq!(w.vault(), 0);
}

#[test]
fn exactly_hitting_target_succeeds() {
    let mut w = world(NewProject::default());
    let creator_owner = w.creator.pubkey();
    let creator_token = w.bench.usdc_account(&creator_owner, 0);
    let a = w.backer(100);
    w.pledge(&a, 100 * USDC, None);
    w.past_deadline();
    w.settle(Some(creator_token)).unwrap();
    assert_eq!(w.status(), ProjectStatus::Succeeded);
}

#[test]
fn cannot_settle_before_deadline() {
    let mut w = world(NewProject::default());
    let a = w.backer(200);
    w.pledge(&a, 150 * USDC, None);
    w.bench.warp(7 * DAY - 1);
    assert_fault(w.settle(None), E::DeadlineNotReached);
}

#[test]
fn cannot_settle_twice() {
    let mut w = world(NewProject::default());
    w.past_deadline();
    w.settle(None).unwrap();
    assert_fault(w.settle(None), E::ProjectNotActive);
}

#[test]
fn success_requires_the_payees_own_usdc_account() {
    let mut w = world(NewProject::default());
    let a = w.backer(100);
    w.pledge(&a, 100 * USDC, None);
    w.past_deadline();

    assert_fault(w.settle(None), E::PayeeAccountRequired);
    // An attacker names their own account as payee: refused.
    let attacker = Pubkey::new_unique();
    let attacker_token = w.bench.usdc_account(&attacker, 0);
    assert_fault(w.settle(Some(attacker_token)), E::WrongPayeeAccount);
    assert_eq!(w.vault(), 100 * USDC, "nothing moved");
    assert_eq!(w.status(), ProjectStatus::Active);
}

#[test]
fn community_success_pays_elected_contractor_not_creator() {
    let candidates = vec![Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique()];
    let mut w = world(NewProject {
        kind: ProjectKind::Community,
        candidates: candidates.clone(),
        ..Default::default()
    });
    let a = w.backer(100);
    let b = w.backer(100);
    let c = w.backer(100);
    w.pledge(&a, 80 * USDC, Some(2)); // big pledge, still one vote
    w.pledge(&b, 10 * USDC, Some(1));
    w.pledge(&c, 10 * USDC, Some(1)); // candidate 1 wins 2-1
    w.past_deadline();

    let creator_owner = w.creator.pubkey();
    let creator_token = w.bench.usdc_account(&creator_owner, 0);
    assert_fault(w.settle(Some(creator_token)), E::WrongPayeeAccount);

    let winner_token = w.bench.usdc_account(&candidates[1], 0);
    w.settle(Some(winner_token)).unwrap();
    assert_eq!(w.bench.balance(winner_token), 95 * USDC);
    assert_eq!(w.treasury_balance(), 5 * USDC);
    assert_eq!(w.bench.balance(creator_token), 0, "creator gets nothing");
}

#[test]
fn community_tie_goes_to_lowest_candidate_number() {
    let candidates = vec![Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique()];
    let mut w = world(NewProject {
        kind: ProjectKind::Community,
        candidates: candidates.clone(),
        ..Default::default()
    });
    let a = w.backer(100);
    let b = w.backer(100);
    w.pledge(&a, 50 * USDC, Some(2));
    w.pledge(&b, 50 * USDC, Some(1)); // 1-1 tie between #1 and #2
    w.past_deadline();
    let winner_token = w.bench.usdc_account(&candidates[1], 0);
    w.settle(Some(winner_token)).unwrap();
    assert_eq!(w.bench.balance(winner_token), 95 * USDC);
}

// =================================================================== refund

#[test]
fn refund_works_after_missed_deadline_without_settle() {
    let mut w = world(NewProject::default());
    let a = w.backer(50);
    w.pledge(&a, 40 * USDC, None);
    w.past_deadline();
    w.refund(&a).unwrap(); // nobody ran settle; still allowed
    assert_eq!(w.bench.balance(a.1), 50 * USDC);
}

#[test]
fn cannot_refund_twice() {
    let mut w = world(NewProject::default());
    let a = w.backer(50);
    w.pledge(&a, 40 * USDC, None);
    w.past_deadline();
    w.refund(&a).unwrap();
    assert!(w.refund(&a).is_err(), "receipt is closed");
    assert_eq!(w.bench.balance(a.1), 50 * USDC);
}

#[test]
fn no_refunds_while_active_or_after_success() {
    let mut w = world(NewProject::default());
    let creator_owner = w.creator.pubkey();
    let creator_token = w.bench.usdc_account(&creator_owner, 0);
    let a = w.backer(150);
    w.pledge(&a, 120 * USDC, None);
    assert_fault(w.refund(&a), E::RefundsNotOpen); // still active

    w.past_deadline();
    assert_fault(w.refund(&a), E::RefundsNotOpen); // target hit, not settled
    w.settle(Some(creator_token)).unwrap();
    assert_fault(w.refund(&a), E::RefundsNotOpen); // paid out
}

#[test]
fn cannot_claim_someone_elses_refund() {
    let mut w = world(NewProject::default());
    let victim = w.backer(50);
    w.pledge(&victim, 40 * USDC, None);
    w.past_deadline();
    let thief = w.backer(0);
    // Thief signs and points at their own token account: their receipt
    // address doesn't exist, so it fails.
    assert!(w.refund(&thief).is_err());
    // Thief tries to receive the victim's refund into the thief's account.
    let mut ix = refund_ix(&victim.0.pubkey(), &w.project, &thief.1);
    ix.accounts[0].is_signer = false;
    assert!(w.bench.send(ix, &[&thief.0]).is_err());
    assert_eq!(w.vault(), 40 * USDC);
}

// =================================================================== cancel

#[test]
fn creator_cancels_and_backers_refund_immediately() {
    let mut w = world(NewProject::default());
    let a = w.backer(50);
    w.pledge(&a, 30 * USDC, None);
    let creator = w.creator.insecure_clone();
    w.cancel(&creator).unwrap();
    assert_eq!(w.status(), ProjectStatus::Cancelled);

    w.refund(&a).unwrap(); // no waiting for the deadline
    assert_eq!(w.bench.balance(a.1), 50 * USDC);

    let b = w.backer(50);
    let ix = pledge_ix(&b.0.pubkey(), &w.project, &b.1, 5 * USDC, None);
    assert_fault(w.bench.send(ix, &[&b.0]), E::ProjectNotActive);
    w.past_deadline();
    assert_fault(w.settle(None), E::ProjectNotActive);
}

#[test]
fn only_the_creator_can_cancel() {
    let mut w = world(NewProject::default());
    let stranger = w.bench.wallet();
    assert!(w.cancel(&stranger).is_err());
    assert_eq!(w.status(), ProjectStatus::Active);
}

#[test]
fn cannot_cancel_after_deadline() {
    let mut w = world(NewProject::default());
    w.past_deadline();
    let creator = w.creator.insecure_clone();
    assert_fault(w.cancel(&creator), E::DeadlinePassed);
}
