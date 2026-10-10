//! Slice 1: initialize_project. One good run per project kind, then one
//! test per interlock to prove each one trips.
mod common;

use {
    anchor_lang::prelude::Pubkey,
    common::*,
    solana_keypair::Keypair,
    solana_signer::Signer,
    townfund::{error::TownFundError as E, ProjectKind, ProjectStatus, USDC_MINT},
};

fn create(bench: &mut Bench, creator: &Keypair, p: &NewProject) -> Result<(), Fault> {
    let ix = initialize_project_ix(&creator.pubkey(), p);
    bench.send(ix, &[creator])
}

fn expect_fault(p: NewProject, fault: E) {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    let err = create(&mut bench, &creator, &p).expect_err("should be refused");
    assert_eq!(err.code, Some(code(fault)), "logs: {:#?}", err.logs);
}

fn candidates(n: usize) -> Vec<Pubkey> {
    (0..n).map(|_| Pubkey::new_unique()).collect()
}

// ---------------------------------------------------------------- good runs

#[test]
fn creator_project_is_created_with_empty_vault() {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    let p = NewProject::default();
    create(&mut bench, &creator, &p).unwrap();

    let address = project_address(&creator.pubkey(), p.project_id);
    let project = bench.project(address);
    assert_eq!(project.version, 1);
    assert_eq!(project.kind, ProjectKind::Creator);
    assert_eq!(project.status, ProjectStatus::Active);
    assert_eq!(project.creator, creator.pubkey());
    assert_eq!(project.project_id, 1);
    assert_eq!(project.mint, USDC_MINT);
    assert_eq!(project.target, 100 * USDC);
    assert_eq!(project.total_pledged, 0);
    assert_eq!(project.backer_count, 0);
    assert_eq!(project.deadline, project.created_at + 7 * DAY);
    assert_eq!(project.created_at, bench.now());
    assert_eq!(project.title, p.title);
    assert!(project.candidates.is_empty());
    assert_eq!(project.votes, [0; 5]);
    assert_eq!(project.reserved, [0; 64]);

    // Vault: right address, holds USDC, owned by the project PDA, empty.
    let vault = vault_address(&address);
    assert_eq!(project.vault, vault);
    let (mint, owner, amount) = bench.token_account(vault);
    assert_eq!(mint, USDC_MINT);
    assert_eq!(owner, address, "only the program (via the project PDA) controls the vault");
    assert_eq!(amount, 0);
}

#[test]
fn community_project_stores_candidates() {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    let list = candidates(3);
    let p = NewProject { kind: ProjectKind::Community, candidates: list.clone(), ..Default::default() };
    create(&mut bench, &creator, &p).unwrap();

    let project = bench.project(project_address(&creator.pubkey(), 1));
    assert_eq!(project.kind, ProjectKind::Community);
    assert_eq!(project.candidates, list);
}

#[test]
fn community_project_accepts_two_and_five_candidates() {
    for n in [2, 5] {
        let mut bench = Bench::new();
        let creator = bench.wallet();
        let p = NewProject { kind: ProjectKind::Community, candidates: candidates(n), ..Default::default() };
        create(&mut bench, &creator, &p).unwrap();
    }
}

#[test]
fn limits_are_inclusive() {
    // Exactly 10 USDC, exactly 1 day, exactly 60 days, a 64-byte title.
    let mut bench = Bench::new();
    let creator = bench.wallet();
    let cases = [
        NewProject { project_id: 1, target: 10 * USDC, ..Default::default() },
        NewProject { project_id: 2, duration_secs: DAY, ..Default::default() },
        NewProject { project_id: 3, duration_secs: 60 * DAY, ..Default::default() },
        NewProject { project_id: 4, title: "x".repeat(64), ..Default::default() },
    ];
    for p in &cases {
        create(&mut bench, &creator, p).unwrap();
    }
}

#[test]
fn one_creator_can_run_many_projects() {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    for id in 1..=3 {
        create(&mut bench, &creator, &NewProject { project_id: id, ..Default::default() }).unwrap();
    }
}

#[test]
fn two_creators_can_use_the_same_project_id() {
    let mut bench = Bench::new();
    let a = bench.wallet();
    let b = bench.wallet();
    create(&mut bench, &a, &NewProject::default()).unwrap();
    create(&mut bench, &b, &NewProject::default()).unwrap();
}

// ------------------------------------------------------------ interlocks

#[test]
fn rejects_target_below_10_usdc() {
    expect_fault(NewProject { target: 10 * USDC - 1, ..Default::default() }, E::TargetTooLow);
}

#[test]
fn rejects_duration_under_1_day() {
    expect_fault(NewProject { duration_secs: DAY - 1, ..Default::default() }, E::InvalidDuration);
}

#[test]
fn rejects_duration_over_60_days() {
    expect_fault(NewProject { duration_secs: 60 * DAY + 1, ..Default::default() }, E::InvalidDuration);
}

#[test]
fn rejects_zero_and_negative_duration() {
    expect_fault(NewProject { duration_secs: 0, ..Default::default() }, E::InvalidDuration);
    expect_fault(NewProject { duration_secs: -DAY, ..Default::default() }, E::InvalidDuration);
}

#[test]
#[cfg(not(feature = "demo"))]
fn normal_build_refuses_demo_durations() {
    for secs in [10, 20, 600] {
        expect_fault(NewProject { duration_secs: secs, ..Default::default() }, E::InvalidDuration);
    }
}

#[test]
fn rejects_title_over_64_bytes() {
    expect_fault(NewProject { title: "x".repeat(65), ..Default::default() }, E::TitleTooLong);
}

#[test]
fn rejects_candidates_on_creator_project() {
    expect_fault(
        NewProject { candidates: candidates(2), ..Default::default() },
        E::CreatorProjectHasCandidates,
    );
}

#[test]
fn rejects_community_with_too_few_or_too_many_candidates() {
    for n in [0, 1, 6] {
        expect_fault(
            NewProject { kind: ProjectKind::Community, candidates: candidates(n), ..Default::default() },
            E::WrongCandidateCount,
        );
    }
}

#[test]
fn rejects_duplicate_candidate() {
    let c = Pubkey::new_unique();
    expect_fault(
        NewProject {
            kind: ProjectKind::Community,
            candidates: vec![c, Pubkey::new_unique(), c],
            ..Default::default()
        },
        E::DuplicateCandidate,
    );
}

#[test]
fn rejects_empty_address_as_candidate() {
    expect_fault(
        NewProject {
            kind: ProjectKind::Community,
            candidates: vec![Pubkey::new_unique(), Pubkey::default()],
            ..Default::default()
        },
        E::InvalidCandidate,
    );
}

#[test]
fn rejects_any_mint_except_townfund_usdc() {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    let fake = Pubkey::new_unique();
    let authority = creator.pubkey();
    bench.put_mint(fake, authority, 6); // looks exactly like USDC, wrong address
    let err = create(&mut bench, &creator, &NewProject { mint: fake, ..Default::default() })
        .expect_err("fake USDC must be refused");
    assert_eq!(err.code, Some(code(E::WrongMint)));
}

#[test]
fn rejects_reusing_a_project_id() {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    create(&mut bench, &creator, &NewProject::default()).unwrap();
    let again = create(&mut bench, &creator, &NewProject { target: 500 * USDC, ..Default::default() });
    assert!(again.is_err(), "existing project must not be overwritten");
    // Original is untouched.
    let project = bench.project(project_address(&creator.pubkey(), 1));
    assert_eq!(project.target, 100 * USDC);
}

#[test]
fn rejects_unsigned_creator() {
    // Someone else pays and signs, naming `victim` as creator without its signature.
    let mut bench = Bench::new();
    let payer = bench.wallet();
    let victim = Keypair::new();
    let mut ix = initialize_project_ix(&victim.pubkey(), &NewProject::default());
    ix.accounts[0].is_signer = false;
    assert!(bench.send(ix, &[&payer]).is_err());
}

#[test]
#[cfg(feature = "demo")]
fn demo_build_accepts_10_seconds_to_10_minutes() {
    let mut bench = Bench::new();
    let creator = bench.wallet();
    for (id, secs) in [(1, 10), (2, 20), (3, 120), (4, 600)] {
        create(&mut bench, &creator, &NewProject { project_id: id, duration_secs: secs, ..Default::default() })
            .unwrap();
        let project = bench.project(project_address(&creator.pubkey(), id));
        assert_eq!(project.deadline, project.created_at + secs);
    }
    // Just outside the demo window is still refused.
    expect_fault(NewProject { duration_secs: 9, ..Default::default() }, E::InvalidDuration);
    expect_fault(NewProject { duration_secs: 601, ..Default::default() }, E::InvalidDuration);
    // Normal 1-60 day projects still work in the demo build.
    create(&mut bench, &creator, &NewProject { project_id: 5, duration_secs: 7 * DAY, ..Default::default() })
        .unwrap();
}

#[test]
#[cfg(feature = "demo")]
fn demo_build_full_cycle_in_20_seconds() {
    // Create -> pledge -> deadline -> settle 95/5, on a 20-second clock.
    let mut bench = Bench::new();
    let creator = bench.wallet();
    bench.send(initialize_treasury_ix(&creator.pubkey()), &[&creator]).unwrap();
    let p = NewProject { target: 10 * USDC, duration_secs: 20, ..Default::default() };
    create(&mut bench, &creator, &p).unwrap();
    let project = project_address(&creator.pubkey(), 1);

    let backer = bench.wallet();
    let owner = backer.pubkey();
    let backer_token = bench.usdc_account(&owner, 10 * USDC);
    bench.send(pledge_ix(&owner, &project, &backer_token, 10 * USDC, None), &[&backer]).unwrap();

    bench.warp(20);
    let creator_owner = creator.pubkey();
    let creator_token = bench.usdc_account(&creator_owner, 0);
    bench.send(settle_ix(&owner, &project, Some(creator_token)), &[&backer]).unwrap();
    assert_eq!(bench.balance(creator_token), 9_500_000);
    assert_eq!(bench.balance(treasury_vault_address()), 500_000);
}
