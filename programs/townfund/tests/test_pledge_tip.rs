//! Slice 2: treasury, pledge, tip.
mod common;

use {
    anchor_lang::prelude::Pubkey,
    common::*,
    solana_keypair::Keypair,
    solana_signer::Signer,
    townfund::{error::TownFundError as E, ProjectKind, USDC_MINT},
};

/// A bench with one live project. Returns (bench, creator, project address).
fn with_project(p: NewProject) -> (Bench, Keypair, Pubkey) {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    let ix = initialize_project_ix(&creator.pubkey(), &p);
    bench.send(ix, &[&creator]).unwrap();
    let project = project_address(&creator.pubkey(), p.project_id);
    (bench, creator, project)
}

fn community() -> NewProject {
    NewProject {
        kind: ProjectKind::Community,
        candidates: vec![Pubkey::new_unique(), Pubkey::new_unique(), Pubkey::new_unique()],
        ..Default::default()
    }
}

/// A backer wallet holding `usdc` whole USDC. Returns (keypair, token account).
fn backer(bench: &mut Bench, usdc: u64) -> (Keypair, Pubkey) {
    let k = bench.wallet();
    let owner = k.pubkey();
    let token = bench.usdc_account(&owner, usdc * USDC);
    (k, token)
}

fn pledge(
    bench: &mut Bench,
    who: &(Keypair, Pubkey),
    project: &Pubkey,
    amount: u64,
    vote: Option<u8>,
) -> Result<(), Fault> {
    let ix = pledge_ix(&who.0.pubkey(), project, &who.1, amount, vote);
    bench.send(ix, &[&who.0])
}

fn assert_fault(res: Result<(), Fault>, fault: E) {
    let err = res.expect_err("should be refused");
    assert_eq!(err.code, Some(code(fault)), "logs: {:#?}", err.logs);
}

// --------------------------------------------------------------- treasury

#[test]
fn treasury_is_created_once_with_no_door() {
    let mut bench = Bench::new();
    let payer = bench.wallet();
    bench.send(initialize_treasury_ix(&payer.pubkey()), &[&payer]).unwrap();

    let t = bench.treasury();
    assert_eq!(t.version, 1);
    assert_eq!(t.mint, USDC_MINT);
    assert_eq!(t.vault, treasury_vault_address());
    assert_eq!(t.door_authority, None, "V1: nobody can spend treasury funds");
    assert_eq!(t.total_fees, 0);
    let (mint, owner, amount) = bench.token_account(t.vault);
    assert_eq!((mint, owner, amount), (USDC_MINT, treasury_address(), 0));

    // A second call (by anyone) is refused.
    let other = bench.wallet();
    assert!(bench.send(initialize_treasury_ix(&other.pubkey()), &[&other]).is_err());
}

// ----------------------------------------------------------------- pledge

#[test]
fn first_pledge_moves_usdc_into_vault_and_creates_receipt() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let b = backer(&mut bench, 50);
    pledge(&mut bench, &b, &project, 20 * USDC, None).unwrap();

    assert_eq!(bench.balance(b.1), 30 * USDC);
    assert_eq!(bench.balance(vault_address(&project)), 20 * USDC);
    let p = bench.project(project);
    assert_eq!(p.total_pledged, 20 * USDC);
    assert_eq!(p.backer_count, 1);
    let r = bench.pledge_record(pledge_address(&project, &b.0.pubkey()));
    assert_eq!(r.backer, b.0.pubkey());
    assert_eq!(r.project, project);
    assert_eq!(r.amount, 20 * USDC);
    assert_eq!(r.vote, None);
    assert!(!r.refunded);
}

#[test]
fn top_up_adds_to_same_receipt_without_new_backer() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let b = backer(&mut bench, 50);
    pledge(&mut bench, &b, &project, 5 * USDC, None).unwrap();
    pledge(&mut bench, &b, &project, 7 * USDC, None).unwrap();

    let p = bench.project(project);
    assert_eq!(p.total_pledged, 12 * USDC);
    assert_eq!(p.backer_count, 1);
    assert_eq!(bench.pledge_record(pledge_address(&project, &b.0.pubkey())).amount, 12 * USDC);
}

#[test]
fn many_backers_and_overfunding_are_allowed() {
    let (mut bench, _, project) = with_project(NewProject::default()); // target 100
    for _ in 0..3 {
        let b = backer(&mut bench, 60);
        pledge(&mut bench, &b, &project, 60 * USDC, None).unwrap();
    }
    let p = bench.project(project);
    assert_eq!(p.backer_count, 3);
    assert_eq!(p.total_pledged, 180 * USDC);
    assert_eq!(bench.balance(vault_address(&project)), 180 * USDC);
}

#[test]
fn exactly_1_usdc_is_accepted() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let b = backer(&mut bench, 5);
    pledge(&mut bench, &b, &project, USDC, None).unwrap();
    pledge(&mut bench, &b, &project, USDC, None).unwrap(); // top-up too
}

#[test]
fn rejects_pledge_under_1_usdc_including_top_ups() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let b = backer(&mut bench, 5);
    assert_fault(pledge(&mut bench, &b, &project, USDC - 1, None), E::PledgeTooSmall);
    pledge(&mut bench, &b, &project, USDC, None).unwrap();
    assert_fault(pledge(&mut bench, &b, &project, USDC - 1, None), E::PledgeTooSmall);
}

#[test]
fn pledges_close_at_the_deadline() {
    let (mut bench, _, project) = with_project(NewProject::default()); // 7 days
    let b = backer(&mut bench, 10);
    bench.warp(7 * DAY - 1);
    pledge(&mut bench, &b, &project, USDC, None).unwrap(); // last second: ok
    bench.warp(1);
    assert_fault(pledge(&mut bench, &b, &project, USDC, None), E::DeadlinePassed);
}

#[test]
fn rejects_pledge_with_more_usdc_than_backer_has() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let b = backer(&mut bench, 5);
    assert!(pledge(&mut bench, &b, &project, 6 * USDC, None).is_err());
    // All or nothing: no receipt, no totals.
    assert!(bench.svm.get_account(&pledge_address(&project, &b.0.pubkey())).is_none());
    assert_eq!(bench.project(project).total_pledged, 0);
    assert_eq!(bench.balance(b.1), 5 * USDC);
}

#[test]
fn rejects_spending_someone_elses_usdc_account() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let victim = backer(&mut bench, 50);
    let thief = bench.wallet();
    let ix = pledge_ix(&thief.pubkey(), &project, &victim.1, 10 * USDC, None);
    assert!(bench.send(ix, &[&thief]).is_err());
    assert_eq!(bench.balance(victim.1), 50 * USDC);
}

#[test]
fn rejects_sending_pledge_to_a_different_vault() {
    let (mut bench, _, project) = with_project(NewProject::default());
    // A second project's vault exists, but this pledge names project #1.
    let other_creator = bench.wallet();
    let ix = initialize_project_ix(&other_creator.pubkey(), &NewProject::default());
    bench.send(ix, &[&other_creator]).unwrap();
    let other_vault = vault_address(&project_address(&other_creator.pubkey(), 1));

    let b = backer(&mut bench, 50);
    let mut ix = pledge_ix(&b.0.pubkey(), &project, &b.1, 10 * USDC, None);
    let vault_meta = ix.accounts.iter_mut().find(|m| m.pubkey == vault_address(&project)).unwrap();
    vault_meta.pubkey = other_vault;
    assert!(bench.send(ix, &[&b.0]).is_err());
}

#[test]
fn rejects_fake_usdc() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let b = bench.wallet();
    let fake_mint = Pubkey::new_unique();
    let owner = b.pubkey();
    bench.put_mint(fake_mint, owner, 6);
    let fake_token = Pubkey::new_unique();
    bench.put_token_account(fake_token, fake_mint, owner, 100 * USDC);
    let ix = pledge_ix(&owner, &project, &fake_token, 10 * USDC, None);
    assert!(bench.send(ix, &[&b]).is_err());
    assert_eq!(bench.balance(vault_address(&project)), 0);
}

// ----------------------------------------------------------------- voting

#[test]
fn community_first_pledge_casts_one_vote() {
    let (mut bench, _, project) = with_project(community());
    let a = backer(&mut bench, 50);
    let b = backer(&mut bench, 50);
    let c = backer(&mut bench, 50);
    pledge(&mut bench, &a, &project, 10 * USDC, Some(2)).unwrap();
    pledge(&mut bench, &b, &project, 1 * USDC, Some(2)).unwrap();
    pledge(&mut bench, &c, &project, 40 * USDC, Some(0)).unwrap();
    // One wallet = one vote, no matter the amount.
    assert_eq!(bench.project(project).votes, [1, 0, 2, 0, 0]);
    assert_eq!(bench.pledge_record(pledge_address(&project, &a.0.pubkey())).vote, Some(2));
}

#[test]
fn community_top_up_keeps_vote_locked() {
    let (mut bench, _, project) = with_project(community());
    let a = backer(&mut bench, 50);
    pledge(&mut bench, &a, &project, 5 * USDC, Some(1)).unwrap();
    pledge(&mut bench, &a, &project, 5 * USDC, None).unwrap(); // top-up, no new vote
    assert_fault(pledge(&mut bench, &a, &project, 5 * USDC, Some(0)), E::VoteNotAllowed);
    assert_eq!(bench.project(project).votes, [0, 1, 0, 0, 0]);
    assert_eq!(bench.pledge_record(pledge_address(&project, &a.0.pubkey())).amount, 10 * USDC);
}

#[test]
fn community_first_pledge_requires_a_vote() {
    let (mut bench, _, project) = with_project(community());
    let a = backer(&mut bench, 50);
    assert_fault(pledge(&mut bench, &a, &project, 5 * USDC, None), E::VoteRequired);
}

#[test]
fn rejects_vote_for_candidate_that_does_not_exist() {
    let (mut bench, _, project) = with_project(community()); // 3 candidates: 0,1,2
    let a = backer(&mut bench, 50);
    assert_fault(pledge(&mut bench, &a, &project, 5 * USDC, Some(3)), E::InvalidVote);
    assert_fault(pledge(&mut bench, &a, &project, 5 * USDC, Some(255)), E::InvalidVote);
}

#[test]
fn rejects_vote_on_creator_project() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let a = backer(&mut bench, 50);
    assert_fault(pledge(&mut bench, &a, &project, 5 * USDC, Some(0)), E::VoteNotAllowed);
}

// -------------------------------------------------------------------- tip

#[test]
fn tip_goes_straight_to_creator_with_no_fee() {
    let (mut bench, creator, project) = with_project(NewProject::default());
    let creator_owner = creator.pubkey();
    let creator_token = bench.usdc_account(&creator_owner, 0);
    let t = backer(&mut bench, 10);
    let ix = tip_ix(&t.0.pubkey(), &project, &t.1, &creator_token, 3 * USDC);
    bench.send(ix, &[&t.0]).unwrap();

    assert_eq!(bench.balance(creator_token), 3 * USDC, "creator gets 100%");
    assert_eq!(bench.balance(t.1), 7 * USDC);
    assert_eq!(bench.balance(vault_address(&project)), 0, "tips never enter the vault");
    assert_eq!(bench.project(project).total_pledged, 0, "tips don't count toward target");
}

#[test]
fn rejects_tip_to_an_account_the_creator_does_not_own() {
    let (mut bench, _, project) = with_project(NewProject::default());
    let t = backer(&mut bench, 10);
    let impostor = Pubkey::new_unique();
    let impostor_token = bench.usdc_account(&impostor, 0);
    let ix = tip_ix(&t.0.pubkey(), &project, &t.1, &impostor_token, 3 * USDC);
    assert!(bench.send(ix, &[&t.0]).is_err());
    assert_eq!(bench.balance(t.1), 10 * USDC);
}

#[test]
fn rejects_zero_tip() {
    let (mut bench, creator, project) = with_project(NewProject::default());
    let creator_owner = creator.pubkey();
    let creator_token = bench.usdc_account(&creator_owner, 0);
    let t = backer(&mut bench, 10);
    let ix = tip_ix(&t.0.pubkey(), &project, &t.1, &creator_token, 0);
    assert_fault(bench.send(ix, &[&t.0]), E::TipZero);
}
