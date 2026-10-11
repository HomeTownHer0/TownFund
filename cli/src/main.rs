//! `townfund`: run TownFund on Devnet from the terminal.
//!
//! The Demo Day story in one command:
//!   cargo run -p townfund-cli -- demo
//!
//! Every step prints an Explorer link so the audience can watch it happen.
//! Keys: your main wallet (~/.config/solana/id.json) pays for and funds
//! everything. Demo wallets (creator, backer1, backer2) are created once in
//! ~/.config/solana/townfund-demo/ and reused. No key ever leaves your Mac.

use std::{
    fs,
    path::{Path, PathBuf},
    thread::sleep,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anchor_lang::{
    prelude::Pubkey,
    solana_program::instruction::{AccountMeta, Instruction},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use anyhow::{anyhow, bail, Context, Result};
use clap::{Parser, Subcommand};
use solana_commitment_config::CommitmentConfig;
use solana_keypair::Keypair;
use solana_rpc_client::rpc_client::RpcClient;
use solana_signer::Signer;
use solana_transaction::Transaction;
use townfund::{
    Project, ProjectKind, BADGE_AUTHORITY_SEED, BADGE_SEED, PLEDGE_SEED,
    PROJECT_SEED, TREASURY_SEED, TREASURY_VAULT_SEED, USDC_MINT, VAULT_SEED,
};

const USDC: u64 = 1_000_000;
const TOKEN_PROGRAM: Pubkey = anchor_spl::token::ID;
const ATA_PROGRAM: Pubkey = anchor_spl::associated_token::ID;
const SYSTEM_PROGRAM: Pubkey = anchor_lang::solana_program::system_program::ID;
const DEMO_SOL: u64 = 100_000_000; // 0.1 SOL per demo wallet
const DEVNET: &str = "https://api.devnet.solana.com";

#[derive(Parser)]
#[command(name = "townfund", about = "Run TownFund on Devnet")]
struct Cli {
    /// RPC URL (default: Devnet)
    #[arg(long, global = true, default_value = DEVNET)]
    url: String,
    /// Main wallet: pays fees and funds the demo wallets
    #[arg(long, global = true, default_value = "~/.config/solana/id.json")]
    keypair: String,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run everything: wallets, treasury, success story, failure story
    Demo {
        /// Project length in seconds (demo build accepts 10-600)
        #[arg(long, default_value_t = 20)]
        seconds: i64,
    },
    /// Create (once) and fund the creator / backer1 / backer2 wallets
    SetupWallets,
    /// Create TownFund's treasury (once per program; safe to rerun)
    InitTreasury,
    /// Individual project: pledge + tip -> settle 95/5 -> soulbound badge
    /// (run setup-wallets first)
    DemoSuccess {
        #[arg(long, default_value_t = 20)]
        seconds: i64,
    },
    /// Community project: under target -> fails -> backer refunded 100%
    /// (run setup-wallets first)
    DemoFail {
        #[arg(long, default_value_t = 20)]
        seconds: i64,
    },
    /// Show a project's on-chain record
    Status { project: Pubkey },
    /// Show SOL and USDC balances of all wallets
    Balances,
    /// Send the demo wallets' USDC back to your main wallet (for reruns)
    Recycle,
}

// ===================================================================== app

struct App {
    rpc: RpcClient,
    url: String,
    main: Keypair,
}

impl App {
    fn link(&self, kind: &str, id: &str) -> String {
        let cluster = if self.url == DEVNET {
            "cluster=devnet".to_string()
        } else {
            format!("cluster=custom&customUrl={}", self.url)
        };
        format!("https://explorer.solana.com/{kind}/{id}?{cluster}")
    }

    fn send(&self, label: &str, ixs: &[Instruction], signers: &[&Keypair]) -> Result<()> {
        let payer = signers[0].pubkey();
        let blockhash = self.rpc.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(ixs, Some(&payer), signers, blockhash);
        let sig = self
            .rpc
            .send_and_confirm_transaction(&tx)
            .with_context(|| format!("{label} failed"))?;
        println!("  ✅ {label}");
        println!("     {}", self.link("tx", &sig.to_string()));
        Ok(())
    }

    fn exists(&self, address: &Pubkey) -> bool {
        self.rpc
            .get_account_with_commitment(address, CommitmentConfig::confirmed())
            .map(|r| r.value.is_some())
            .unwrap_or(false)
    }

    fn usdc(&self, owner: &Pubkey) -> u64 {
        self.rpc
            .get_token_account_balance(&ata(owner))
            .ok()
            .and_then(|b| b.amount.parse().ok())
            .unwrap_or(0)
    }

    fn sol(&self, owner: &Pubkey) -> u64 {
        self.rpc.get_balance(owner).unwrap_or(0)
    }

    fn project(&self, address: &Pubkey) -> Result<Project> {
        let acct = self.rpc.get_account(address)?;
        let mut data: &[u8] = &acct.data;
        Ok(Project::try_deserialize(&mut data)?)
    }

    /// Chain clock (what the program checks deadlines against).
    fn chain_time(&self) -> Result<i64> {
        let slot = self.rpc.get_slot()?;
        Ok(self.rpc.get_block_time(slot)?)
    }
}

// ============================================================ addresses

fn pda(seeds: &[&[u8]]) -> Pubkey {
    Pubkey::find_program_address(seeds, &townfund::id()).0
}
fn project_address(creator: &Pubkey, id: u64) -> Pubkey {
    pda(&[PROJECT_SEED, creator.as_ref(), &id.to_le_bytes()])
}
fn vault_address(project: &Pubkey) -> Pubkey {
    pda(&[VAULT_SEED, project.as_ref()])
}
fn pledge_address(project: &Pubkey, backer: &Pubkey) -> Pubkey {
    pda(&[PLEDGE_SEED, project.as_ref(), backer.as_ref()])
}
fn badge_address(project: &Pubkey, backer: &Pubkey) -> Pubkey {
    pda(&[BADGE_SEED, project.as_ref(), backer.as_ref()])
}
fn treasury_address() -> Pubkey {
    pda(&[TREASURY_SEED])
}
fn treasury_vault_address() -> Pubkey {
    pda(&[TREASURY_VAULT_SEED])
}
fn badge_authority_address() -> Pubkey {
    pda(&[BADGE_AUTHORITY_SEED])
}
/// The owner's standard (associated) USDC account.
fn ata(owner: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), TOKEN_PROGRAM.as_ref(), USDC_MINT.as_ref()],
        &ATA_PROGRAM,
    )
    .0
}

// ================================================= standard instructions

/// Create `owner`'s USDC account if it doesn't exist yet (no-op otherwise).
fn create_ata_ix(payer: &Pubkey, owner: &Pubkey) -> Instruction {
    Instruction {
        program_id: ATA_PROGRAM,
        accounts: vec![
            AccountMeta::new(*payer, true),
            AccountMeta::new(ata(owner), false),
            AccountMeta::new_readonly(*owner, false),
            AccountMeta::new_readonly(USDC_MINT, false),
            AccountMeta::new_readonly(SYSTEM_PROGRAM, false),
            AccountMeta::new_readonly(TOKEN_PROGRAM, false),
        ],
        data: vec![1], // CreateIdempotent
    }
}

fn sol_transfer_ix(from: &Pubkey, to: &Pubkey, lamports: u64) -> Instruction {
    let mut data = 2u32.to_le_bytes().to_vec(); // System: Transfer
    data.extend_from_slice(&lamports.to_le_bytes());
    Instruction {
        program_id: SYSTEM_PROGRAM,
        accounts: vec![AccountMeta::new(*from, true), AccountMeta::new(*to, false)],
        data,
    }
}

fn usdc_transfer_ix(from_owner: &Pubkey, to_owner: &Pubkey, amount: u64) -> Instruction {
    let mut data = vec![12u8]; // Token: TransferChecked
    data.extend_from_slice(&amount.to_le_bytes());
    data.push(6);
    Instruction {
        program_id: TOKEN_PROGRAM,
        accounts: vec![
            AccountMeta::new(ata(from_owner), false),
            AccountMeta::new_readonly(USDC_MINT, false),
            AccountMeta::new(ata(to_owner), false),
            AccountMeta::new_readonly(*from_owner, true),
        ],
        data,
    }
}

// ================================================ TownFund instructions

fn ix(data: Vec<u8>, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction { program_id: townfund::id(), accounts, data }
}

fn init_treasury_ix(payer: &Pubkey) -> Instruction {
    ix(
        townfund::instruction::InitializeTreasury {}.data(),
        townfund::accounts::InitializeTreasury {
            payer: *payer,
            treasury: treasury_address(),
            mint: USDC_MINT,
            treasury_vault: treasury_vault_address(),
            token_program: TOKEN_PROGRAM,
            system_program: SYSTEM_PROGRAM,
        }
        .to_account_metas(None),
    )
}

#[allow(clippy::too_many_arguments)]
fn create_project_ix(
    creator: &Pubkey,
    project_id: u64,
    kind: ProjectKind,
    target: u64,
    duration_secs: i64,
    title: &str,
    candidates: Vec<Pubkey>,
) -> Instruction {
    let project = project_address(creator, project_id);
    ix(
        townfund::instruction::InitializeProject {
            project_id,
            kind,
            target,
            duration_secs,
            title: title.to_string(),
            candidates,
        }
        .data(),
        townfund::accounts::InitializeProject {
            creator: *creator,
            project,
            mint: USDC_MINT,
            vault: vault_address(&project),
            token_program: TOKEN_PROGRAM,
            system_program: SYSTEM_PROGRAM,
        }
        .to_account_metas(None),
    )
}

fn pledge_ix(backer: &Pubkey, project: &Pubkey, amount: u64, vote: Option<u8>) -> Instruction {
    ix(
        townfund::instruction::Pledge { amount, vote }.data(),
        townfund::accounts::MakePledge {
            backer: *backer,
            project: *project,
            pledge: pledge_address(project, backer),
            mint: USDC_MINT,
            backer_token: ata(backer),
            vault: vault_address(project),
            token_program: TOKEN_PROGRAM,
            system_program: SYSTEM_PROGRAM,
        }
        .to_account_metas(None),
    )
}

fn tip_ix(tipper: &Pubkey, project: &Pubkey, creator: &Pubkey, amount: u64) -> Instruction {
    ix(
        townfund::instruction::Tip { amount }.data(),
        townfund::accounts::Tip {
            tipper: *tipper,
            project: *project,
            mint: USDC_MINT,
            tipper_token: ata(tipper),
            creator_token: ata(creator),
            token_program: TOKEN_PROGRAM,
        }
        .to_account_metas(None),
    )
}

fn settle_ix(caller: &Pubkey, project: &Pubkey, payee: Option<&Pubkey>) -> Instruction {
    ix(
        townfund::instruction::Settle {}.data(),
        townfund::accounts::Settle {
            caller: *caller,
            project: *project,
            vault: vault_address(project),
            treasury: treasury_address(),
            treasury_vault: treasury_vault_address(),
            mint: USDC_MINT,
            payee_token: payee.map(ata),
            token_program: TOKEN_PROGRAM,
        }
        .to_account_metas(None),
    )
}

fn refund_ix(backer: &Pubkey, project: &Pubkey) -> Instruction {
    ix(
        townfund::instruction::Refund {}.data(),
        townfund::accounts::Refund {
            backer: *backer,
            project: *project,
            pledge: pledge_address(project, backer),
            vault: vault_address(project),
            mint: USDC_MINT,
            backer_token: ata(backer),
            token_program: TOKEN_PROGRAM,
        }
        .to_account_metas(None),
    )
}

fn badge_ix(payer: &Pubkey, project: &Pubkey, backer: &Pubkey, asset: &Pubkey) -> Instruction {
    ix(
        townfund::instruction::MintBackerBadge {}.data(),
        townfund::accounts::MintBackerBadge {
            payer: *payer,
            project: *project,
            pledge: pledge_address(project, backer),
            backer: *backer,
            badge: badge_address(project, backer),
            asset: *asset,
            badge_authority: badge_authority_address(),
            mpl_core_program: mpl_core::ID,
            system_program: SYSTEM_PROGRAM,
        }
        .to_account_metas(None),
    )
}

// ================================================================= keys

fn expand(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(rest),
        None => PathBuf::from(path),
    }
}

fn read_keypair(path: &Path) -> Result<Keypair> {
    let text = fs::read_to_string(path).with_context(|| format!("can't read {}", path.display()))?;
    let bytes: Vec<u8> = serde_json::from_str(&text)?;
    Keypair::try_from(bytes.as_slice()).map_err(|e| anyhow!("bad keypair {}: {e}", path.display()))
}

/// Load a demo wallet, creating it (once) if missing. File mode 600.
fn demo_wallet(name: &str) -> Result<Keypair> {
    let dir = expand("~/.config/solana/townfund-demo");
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{name}.json"));
    if path.exists() {
        return read_keypair(&path);
    }
    let k = Keypair::new();
    fs::write(&path, serde_json::to_string(&k.to_bytes().to_vec())?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    }
    println!("  🔑 created {name} wallet: {}", k.pubkey());
    Ok(k)
}

struct Wallets {
    creator: Keypair,
    backer1: Keypair,
    backer2: Keypair,
}

impl Wallets {
    fn named(&self) -> [(&'static str, &Keypair); 3] {
        [("creator", &self.creator), ("backer1", &self.backer1), ("backer2", &self.backer2)]
    }
}

fn wallets() -> Result<Wallets> {
    let w = Wallets {
        creator: demo_wallet("creator")?,
        backer1: demo_wallet("backer1")?,
        backer2: demo_wallet("backer2")?,
    };
    write_public_addresses(&w);
    Ok(w)
}

/// Save the demo wallets' PUBLIC addresses (never keys) to
/// demo-wallets.json in the repo folder, so the live page can be pointed
/// at them. The file is git-ignored.
fn write_public_addresses(w: &Wallets) {
    let json = serde_json::json!({
        "creator": w.creator.pubkey().to_string(),
        "backer1": w.backer1.pubkey().to_string(),
        "backer2": w.backer2.pubkey().to_string(),
    });
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../demo-wallets.json");
    let _ = fs::write(path, serde_json::to_string_pretty(&json).unwrap_or_default() + "\n");
}

fn fmt_usdc(base: u64) -> String {
    let (whole, frac) = (base / USDC, base % USDC);
    if frac == 0 {
        format!("{whole} USDC")
    } else {
        format!("{whole}.{} USDC", format!("{frac:06}").trim_end_matches('0'))
    }
}

fn fmt_sol(lamports: u64) -> String {
    format!("{:.4} SOL", lamports as f64 / 1e9)
}

// =============================================================== stories

fn setup_wallets(app: &App) -> Result<Wallets> {
    println!("\n━━ Demo wallets ━━");
    let w = wallets()?;
    let main = app.main.pubkey();
    // Each backer pledges and tips 1 USDC: backer1 10 + 1, backer2 5 + 1.
    let want = [(&w.creator, 0u64), (&w.backer1, 11 * USDC), (&w.backer2, 6 * USDC)];

    let mut ixs = vec![];
    let mut usdc_needed = 0;
    for (k, usdc) in want {
        let who = k.pubkey();
        if app.sol(&who) < DEMO_SOL / 2 {
            ixs.push(sol_transfer_ix(&main, &who, DEMO_SOL));
        }
        ixs.push(create_ata_ix(&main, &who));
        let have = app.usdc(&who);
        if have < usdc {
            ixs.push(usdc_transfer_ix(&main, &who, usdc - have));
            usdc_needed += usdc - have;
        }
    }
    let main_usdc = app.usdc(&main);
    if main_usdc < usdc_needed {
        bail!(
            "main wallet has {} but the demo wallets need {} more.\n\
             Get Devnet USDC at https://faucet.circle.com (Solana Devnet) for {}",
            fmt_usdc(main_usdc),
            fmt_usdc(usdc_needed),
            main
        );
    }
    app.send("Fund demo wallets (SOL for fees, USDC for pledges and tips)", &ixs, &[&app.main])?;
    for (name, k) in w.named() {
        let who = k.pubkey();
        println!("  {name:8} {who}  {} | {}", fmt_sol(app.sol(&who)), fmt_usdc(app.usdc(&who)));
    }
    Ok(w)
}

fn init_treasury(app: &App) -> Result<()> {
    println!("\n━━ Treasury ━━");
    if app.exists(&treasury_address()) {
        println!("  ✅ already exists: {}", treasury_address());
        return Ok(());
    }
    app.send("Create TownFund treasury", &[init_treasury_ix(&app.main.pubkey())], &[&app.main])
}

fn new_project_id() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

/// Wait for the on-chain clock to pass the deadline, with a countdown.
fn wait_for_deadline(app: &App, deadline: i64) -> Result<()> {
    let started = Instant::now();
    loop {
        let now = app.chain_time()?;
        if now >= deadline {
            println!("\r  ⏰ deadline reached (chain time)           ");
            return Ok(());
        }
        print!("\r  ⏳ {:>3}s until the deadline…   ", deadline - now);
        use std::io::Write;
        std::io::stdout().flush().ok();
        if started.elapsed() > Duration::from_secs(900) {
            bail!("timed out waiting for the deadline");
        }
        sleep(Duration::from_secs(2));
    }
}

fn demo_success(app: &App, w: &Wallets, seconds: i64) -> Result<()> {
    println!("\n━━ Story 1: Individual project SUCCEEDS ━━");
    let creator = w.creator.pubkey();
    let backer = w.backer1.pubkey();
    let id = new_project_id();
    let project = project_address(&creator, id);
    let title = "Pothole on Westheimer";

    app.send(
        &format!("Creator opens \"{title}\": target 10 USDC, {seconds}s"),
        &[create_project_ix(&creator, id, ProjectKind::Creator, 10 * USDC, seconds, title, vec![])],
        &[&w.creator],
    )?;
    println!("     project: {}", app.link("address", &project.to_string()));
    let deadline = app.project(&project)?.deadline;

    app.send("Backer1 pledges 10 USDC → vault (escrow)", &[pledge_ix(&backer, &project, 10 * USDC, None)], &[&w.backer1])?;
    let vault = vault_address(&project);
    println!("     vault now holds {}: {}", fmt_usdc(app.usdc_at(&vault)), app.link("address", &vault.to_string()));
    app.send("Backer1 tips the creator 1 USDC (direct, no fee)", &[tip_ix(&backer, &project, &creator, USDC)], &[&w.backer1])?;

    wait_for_deadline(app, deadline)?;
    let creator_before = app.usdc(&creator);
    let treasury_before = app.usdc_at(&treasury_vault_address());
    settle_retrying(app, &project, Some(&creator), "Settle: 95% → creator, 5% → treasury")?;
    println!(
        "     creator +{} | treasury +{} | vault now {}",
        fmt_usdc(app.usdc(&creator) - creator_before),
        fmt_usdc(app.usdc_at(&treasury_vault_address()) - treasury_before),
        fmt_usdc(app.usdc_at(&vault)),
    );

    let asset = Keypair::new();
    app.send(
        "Mint soulbound backer badge → backer1",
        &[badge_ix(&app.main.pubkey(), &project, &backer, &asset.pubkey())],
        &[&app.main, &asset],
    )?;
    println!("     badge NFT: {}", app.link("address", &asset.pubkey().to_string()));
    let p = app.project(&project)?;
    println!("  🎉 status: {:?}", p.status);
    Ok(())
}

fn demo_fail(app: &App, w: &Wallets, seconds: i64) -> Result<()> {
    println!("\n━━ Story 2: Community project FAILS → refund ━━");
    let creator = w.creator.pubkey();
    let backer = w.backer2.pubkey();
    let id = new_project_id() + 1;
    let project = project_address(&creator, id);
    let title = "Fix Local Park";
    let candidates = vec![Keypair::new().pubkey(), Keypair::new().pubkey()];

    app.send(
        &format!("Creator opens \"{title}\": target 20 USDC, 2 contractors, {seconds}s"),
        &[create_project_ix(&creator, id, ProjectKind::Community, 20 * USDC, seconds, title, candidates)],
        &[&w.creator],
    )?;
    println!("     project: {}", app.link("address", &project.to_string()));
    let deadline = app.project(&project)?.deadline;

    let before = app.usdc(&backer);
    app.send(
        "Backer2 pledges 5 USDC and votes for contractor #0",
        &[pledge_ix(&backer, &project, 5 * USDC, Some(0))],
        &[&w.backer2],
    )?;
    println!("     backer2: {} → {}", fmt_usdc(before), fmt_usdc(app.usdc(&backer)));
    app.send("Backer2 tips the creator 1 USDC (tips are not refundable)", &[tip_ix(&backer, &project, &creator, USDC)], &[&w.backer2])?;

    wait_for_deadline(app, deadline)?;
    settle_retrying(app, &project, None, "Settle: 5 of 20 USDC → project FAILED")?;
    app.send("Backer2 claims refund (100%, no fee)", &[refund_ix(&backer, &project)], &[&w.backer2])?;
    println!("     backer2 got the 5 USDC pledge back; the 1 USDC tip stays with the creator");
    let p = app.project(&project)?;
    println!("  ↩️  status: {:?}", p.status);
    Ok(())
}

/// Devnet's clock can trail wall time by a few seconds; retry briefly if
/// the program says the deadline hasn't been reached yet.
fn settle_retrying(app: &App, project: &Pubkey, payee: Option<&Pubkey>, label: &str) -> Result<()> {
    for attempt in 1..=10 {
        match app.send(label, &[settle_ix(&app.main.pubkey(), project, payee)], &[&app.main]) {
            Ok(()) => return Ok(()),
            Err(e) if attempt < 10 && is_deadline_not_reached(&e) => {
                sleep(Duration::from_secs(2));
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}

/// DeadlineNotReached is TownFund fault 6016 (0x1780).
fn is_deadline_not_reached(e: &anyhow::Error) -> bool {
    let text = format!("{e:?}");
    text.contains("DeadlineNotReached") || text.contains("0x1780")
}

fn status(app: &App, address: &Pubkey) -> Result<()> {
    let p = app.project(address)?;
    println!("Project   {address}");
    println!("  title    {}", p.title);
    println!("  kind     {:?}", p.kind);
    println!("  status   {:?}", p.status);
    println!("  creator  {}", p.creator);
    println!("  target   {}", fmt_usdc(p.target));
    println!("  pledged  {} from {} backer(s)", fmt_usdc(p.total_pledged), p.backer_count);
    println!("  vault    {} ({})", p.vault, fmt_usdc(app.usdc_at(&p.vault)));
    println!("  deadline {}", p.deadline);
    if p.kind == ProjectKind::Community {
        for (i, c) in p.candidates.iter().enumerate() {
            println!("  #{i} {c}  votes: {}", p.votes[i]);
        }
    }
    Ok(())
}

fn balances(app: &App) -> Result<()> {
    let w = wallets()?;
    for (name, who) in [
        ("main", app.main.pubkey()),
        ("creator", w.creator.pubkey()),
        ("backer1", w.backer1.pubkey()),
        ("backer2", w.backer2.pubkey()),
    ] {
        println!("{name:8} {who}  {} | {}", fmt_sol(app.sol(&who)), fmt_usdc(app.usdc(&who)));
    }
    println!(
        "treasury {}  {}",
        treasury_vault_address(),
        fmt_usdc(app.usdc_at(&treasury_vault_address()))
    );
    Ok(())
}

fn recycle(app: &App) -> Result<()> {
    println!("\n━━ Recycle demo USDC → main wallet ━━");
    let w = wallets()?;
    let main = app.main.pubkey();
    for (name, k) in w.named() {
        let amount = app.usdc(&k.pubkey());
        if amount == 0 {
            println!("  {name}: nothing to send");
            continue;
        }
        // Main wallet pays the fee; the demo wallet signs for its USDC.
        app.send(
            &format!("{name} returns {}", fmt_usdc(amount)),
            &[usdc_transfer_ix(&k.pubkey(), &main, amount)],
            &[&app.main, k],
        )?;
    }
    // Earlier versions had a "supporter" tipping wallet; empty it if present.
    let old = expand("~/.config/solana/townfund-demo/supporter.json");
    if old.exists() {
        let k = read_keypair(&old)?;
        let amount = app.usdc(&k.pubkey());
        if amount > 0 {
            app.send(
                &format!("old supporter wallet returns {}", fmt_usdc(amount)),
                &[usdc_transfer_ix(&k.pubkey(), &main, amount)],
                &[&app.main, &k],
            )?;
        }
    }
    println!("  main wallet now has {}", fmt_usdc(app.usdc(&main)));
    Ok(())
}

impl App {
    /// USDC in a specific token account (vault, treasury vault).
    fn usdc_at(&self, token_account: &Pubkey) -> u64 {
        self.rpc
            .get_token_account_balance(token_account)
            .ok()
            .and_then(|b| b.amount.parse().ok())
            .unwrap_or(0)
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let main = read_keypair(&expand(&cli.keypair))?;
    let app = App {
        rpc: RpcClient::new_with_commitment(cli.url.clone(), CommitmentConfig::confirmed()),
        url: cli.url.clone(),
        main,
    };
    println!("TownFund program {}", townfund::id());
    println!("Main wallet      {}", app.main.pubkey());

    match cli.cmd {
        Cmd::Demo { seconds } => {
            let w = setup_wallets(&app)?;
            init_treasury(&app)?;
            demo_success(&app, &w, seconds)?;
            demo_fail(&app, &w, seconds)?;
            println!("\n✅ Demo complete.");
        }
        Cmd::SetupWallets => {
            setup_wallets(&app)?;
        }
        Cmd::InitTreasury => init_treasury(&app)?,
        Cmd::DemoSuccess { seconds } => {
            let w = wallets()?;
            init_treasury(&app)?;
            demo_success(&app, &w, seconds)?;
        }
        Cmd::DemoFail { seconds } => {
            let w = wallets()?;
            init_treasury(&app)?;
            demo_fail(&app, &w, seconds)?;
        }
        Cmd::Status { project } => status(&app, &project)?,
        Cmd::Balances => balances(&app)?,
        Cmd::Recycle => recycle(&app)?,
    }
    Ok(())
}
