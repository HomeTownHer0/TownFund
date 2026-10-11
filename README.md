# TownFund

[![build-and-test](https://github.com/HomeTownHer0/townfund/actions/workflows/build.yml/badge.svg)](https://github.com/HomeTownHer0/townfund/actions/workflows/build.yml)

A Solana program for community fundraising: the first building block of a
community job board. Backers pledge USDC to a project. If it hits its target
by the deadline, the money pays out (95% to the payee, 5% to the TownFund
treasury). If it misses, every backer gets a full refund. The program has no
instruction that moves escrowed money any other way (see the upgrade-authority
caveat under *Known limitations*).

Built by Christian Arceo ([@HomeTownHer0](https://github.com/HomeTownHer0))
in Solana School Fall 2026

> **Status:** V1 program complete (62 tests), live on Devnet, and the full
> demo has run successfully on Devnet (Oct 10, 2026).
> Every revision is recorded in [CHANGELOG.md](CHANGELOG.md).

## How it works (V1, "Creators + Backers")

| Project kind | Who gets paid on success |
|---|---|
| **Creator-only** | The creator |
| **Community** | A contractor elected by backers (2–5 candidates, one wallet = one vote) |

- **Pledge:** at least 1 USDC (top-ups too). Target at least 10 USDC.
  Duration 1–60 days (the Devnet demo build also allows 10 seconds–10 minutes).
- **Vote** (Community only): cast with a backer's first pledge, one wallet =
  one vote, locked for top-ups. Most votes wins; a tie goes to the lowest number.
- **Settle** (anyone can trigger it after the deadline):
  - Target hit → the whole vault pays out once: **95% payee / 5% treasury**
    (fee rounded down, nothing left behind). Overfunding allowed.
  - Target missed → marked Failed; each backer claims a **100% refund**, no fee.
    Refunds also open without anyone running settle.
- **Cancel:** the creator can cancel before the deadline; refunds open at once.
- **Tips** go straight to the creator's USDC account. No fee, not refundable,
  never enter the vault, don't count toward the target.
- **Treasury** is owned by the program (a PDA, no private key). No withdrawals in V1.
- **Backer badge:** after success, each backer can receive a soulbound NFT
  (see below).

## For reviewers

### Watch it live

**[hometownher0.github.io/townfund](https://hometownher0.github.io/townfund/)**: a
read-only page that shows every wallet, both project vaults and the town
treasury straight from Devnet, updating every few seconds. Open it, then run
the demo (below): coins move along the money map, balances count up and down,
and each project's steps tick off as they happen. Nothing to install; it
can't move money.

### Verify it on Devnet (2 minutes)

Program: [`FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv`](https://explorer.solana.com/address/FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv?cluster=devnet)

First full demo run, Oct 10, 2026. Every link is a real Devnet transaction or account:

| Step | Link | What to look for |
|---|---|---|
| Pledge 10 USDC into escrow | [tx](https://explorer.solana.com/tx/35H91cbvVv9VY4fm75NNn1viwBJUXdVoz9jR9tSHvxHjRE4fncKW1tWoxbDRArGtV5NwUQ8Jn4zh4S3SqeNB6gfh?cluster=devnet) | USDC moves backer → vault (a PDA-owned token account) |
| Settle (success) | [tx](https://explorer.solana.com/tx/xcfjRbyLZJcXTM65wvRfKT8QBwyLSZXYVy1T3vUeGE8caNc7sx7UU6sg3HmQoRMEesyqkKuwJB8kJhhYCybL2xW?cluster=devnet) | 9.5 USDC → creator, 0.5 USDC → treasury, vault → 0 |
| Soulbound badge | [NFT](https://explorer.solana.com/address/3M9ngUUCz1iBF7zcQ8RcPikL9xUpikU9nF9aWsmdSN4o?cluster=devnet) | Metaplex Core asset owned by the backer, Permanent Freeze plugin |
| Settle (failure) | [tx](https://explorer.solana.com/tx/nLNkdzxMRK6uh4goqng6i3rtZPdwMVsbtDQTW9RHXroLs2nBHzw7F7jgt2k85U92VFe4uXK4VFZBnoUjv2WdigT?cluster=devnet) | 5 of 20 USDC raised → status Failed, no money moves |
| Refund | [tx](https://explorer.solana.com/tx/4dx7i3uEkWAaaYj2Fe6j8ZxQ4p3BQu8MY6FyRPgsXMVmiWRT6iQ7HgbGBqf3Wo4BY2xD1ifxD2SfRabLP51BekGm?cluster=devnet) | 5 USDC back to the backer; pledge receipt closed, rent returned |

### Instructions

| Instruction | Who can call | Key checks | Money movement |
|---|---|---|---|
| `initialize_project` | Creator (signs) | Target ≥ 10 USDC, duration in range, title ≤ 64 bytes, Creator-only has no candidates, Community has 2–5 distinct non-empty candidates, mint = USDC | Creates Project PDA + empty vault |
| `initialize_treasury` | Anyone, once | Mint = USDC; nothing configurable, so calling first grants no power | Creates treasury PDA + fee vault |
| `pledge` | Backer (signs) | Project Active, before deadline, ≥ 1 USDC, vote rules, source account owned by backer, destination = this project's vault | Backer → vault |
| `tip` | Anyone (signs) | > 0, destination owned by the project's creator | Tipper → creator |
| `settle` | Anyone | Active, deadline passed; on success the payee account must be USDC owned by the payee | Vault → payee 95% + treasury 5%, or none on failure |
| `cancel_project` | Creator only | Active, before deadline | None |
| `refund` | Backer (signs) | Refunds open (Failed, Cancelled, or past deadline under target); receipt exists | Vault → backer 100%; receipt closed |
| `mint_backer_badge` | Anyone (pays rent) | Project Succeeded, backer has a receipt, no badge yet | Mints a Metaplex Core NFT to the backer |

### Accounts (all program-owned, addressed by seeds)

| Account | Seeds | Holds |
|---|---|---|
| Project | `["project", creator, project_id]` | Terms, totals, status, candidates, votes |
| Vault | `["vault", project]` | Escrowed USDC. Authority = the project PDA (no private key) |
| Pledge receipt | `["pledge", project, backer]` | Backer's total and vote |
| Treasury / fee vault | `["treasury"]` / `["treasury_vault"]` | Fees; `door_authority` reserved, always empty in V1 |
| Badge record | `["badge", project, backer]` | Which NFT a backer received; blocks a second mint |
| Badge authority | `["badge_authority"]` | Update authority of every badge; never signs, so badges can't be edited |

Every stored account carries a version byte and spare bytes, so later
versions can add fields without breaking existing data. Every money movement
emits an event (`Pledged`, `Tipped`, `Settled`, `Refunded`, ...).

### Security properties, and the test that proves each

| Property | Test |
|---|---|
| Only Circle Devnet USDC accepted | `rejects_any_mint_except_townfund_usdc`, `rejects_fake_usdc` |
| Pledges can only go into this project's vault | `rejects_sending_pledge_to_a_different_vault` |
| Nobody can spend someone else's USDC | `rejects_spending_someone_elses_usdc_account` |
| Payout only to the real payee | `success_requires_the_payees_own_usdc_account`, `community_success_pays_elected_contractor_not_creator` |
| 95/5 exact, rounded down, nothing left in the vault | `fee_is_rounded_down_and_nothing_is_left_behind` |
| No double settle, refund or badge | `cannot_settle_twice`, `cannot_refund_twice`, `no_second_badge_for_the_same_backer` |
| Refunds only to the backer who pledged | `cannot_claim_someone_elses_refund` |
| No refunds once a project succeeds | `no_refunds_while_active_or_after_success` |
| One wallet = one vote, locked after the first pledge | `community_first_pledge_casts_one_vote`, `community_top_up_keeps_vote_locked` |
| Badge is soulbound | `badge_cannot_be_transferred` |
| Failed transactions change nothing | `rejects_pledge_with_more_usdc_than_backer_has` |

All math is checked (overflow faults instead of wrapping), and release builds
keep overflow checks on. Tests run on LiteSVM against the compiled program and
the real Metaplex Core program dumped from Devnet. Several properties were also
mutation-checked: deliberately breaking the rule made its test fail.

### Code map

| Path | What it is |
|---|---|
| `programs/townfund/src/lib.rs` | Entry point: the list of instructions |
| `programs/townfund/src/instructions/` | One file per instruction: account checks, then logic |
| `programs/townfund/src/state.rs` | Stored account layouts |
| `programs/townfund/src/constants.rs` | Limits, seeds, USDC mint |
| `programs/townfund/src/error.rs` | Every reason an instruction is refused |
| `programs/townfund/tests/` | 62 tests (63 in the demo build) |
| `cli/src/main.rs` | `townfund` command-line tool used for the demo |
| `site/` | Live status page (published to GitHub Pages) |

## Known limitations (roadmap, not bugs)

1. A Community contractor is paid at settle, before the work is done. Milestones fix this.
2. One wallet = one vote, so votes can be bought with extra wallets.
3. Treasury funds stay locked until a "door" (multisig or community vote) is built.
4. **Upgrade authority:** on Devnet the owner's wallet can replace the program,
   which means it could change the rules for escrowed funds. Before real money:
   a multisig or a revoked authority.
5. Project and vault rent is not reclaimed after a project ends (~0.006 SOL per project).
6. USDC sent straight to a vault (not through `pledge`) is not tracked by any
   receipt: on success it pays out with the vault, on failure it stays stuck.
7. Tips on a Community project go to the creator, not the elected contractor.

## Build plan

| Step | What | Status |
|---|---|---|
| Scaffold | Anchor 1.2 workspace, keypair-safe `.gitignore`, passing test, GitHub CI check | ✅ |
| Slice 1 | `initialize_project`, Project account, vault | ✅ |
| Slice 2 | Treasury, `pledge` (+ Community vote), `tip` | ✅ |
| Slice 3 | `settle` (95/5), `refund`, `cancel_project` | ✅ |
| Badge | Soulbound backer NFT (Metaplex Core) after success | ✅ |
| Devnet | Deployed (demo build): [`FdaMK3…PXrv`](https://explorer.solana.com/address/FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv?cluster=devnet) | ✅ |
| Demo tool | `townfund` CLI: full Demo Day story with Explorer links | ✅ |
| Devnet run | Full demo on Devnet (links above) | ✅ |
| Live page | Read-only status page on GitHub Pages | ✅ |
| Demo Day | October 16, 2026 | ⏳ |
| Roadmap | Treasury door, milestones, website | after Oct 16 |

## Deployment

| | |
|---|---|
| Cluster | Devnet only |
| Program ID | `FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv` |
| Build | Demo build (`./scripts/build-devnet.sh`) |
| Upgrade authority | `7cpjF6175EPGZ3gopVLn94h5iuUFvd2VZ9Xq9QKzxU21` (owner's wallet) |
| USDC | Circle Devnet USDC `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` ([faucet](https://faucet.circle.com)) |
| Metaplex Core | `CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d` |

## Build and test

Requires Solana CLI 4.1.2, Anchor 1.2.0, Rust ≥ 1.89.

```bash
./scripts/build.sh          # normal build + 62 tests
./scripts/build-devnet.sh   # demo build (10 s–10 min projects allowed) + 63 tests
```

Both build the program for SBPF v0 with platform-tools v1.54 and run the tests
on LiteSVM (an in-memory Solana chain). GitHub Actions runs both on every push.

## Run the demo (Devnet)

```bash
cargo run -p townfund-cli -- demo --seconds 20
```

One command runs the Demo Day story against the live program and prints an
Explorer link for every step:

1. Creates (once) and funds three demo wallets in `~/.config/solana/townfund-demo/`
   (creator, backer1, backer2) from your main wallet: SOL for fees, USDC for pledges
2. **"Pothole on Westheimer"** (Creator-only) succeeds: pledge 10 USDC into
   escrow, tip 1 USDC, deadline passes, settle 95/5, soulbound badge to the backer
3. **"Fix Local Park"** (Community) fails: pledge 5 of 20 USDC, deadline
   passes, settle marks it Failed, backer refunded 100%

Other commands: `setup-wallets`, `init-treasury`, `demo-success`, `demo-fail`,
`status <project>`, `balances`, `recycle` (sends demo USDC back to your main
wallet for another rehearsal). The first run needs ~16 USDC in the main wallet;
after `recycle`, each run only uses up the 0.5 USDC treasury fee.

## Backer badge

After a project succeeds, `mint_backer_badge` gives each backer a
**soulbound NFT** (Metaplex Core) named "TownFund Backer: \<project title\>".
Core's Permanent Freeze plugin freezes it at creation with no authority that
could unfreeze it, so it can never be transferred or sold. One per backer per
project; never for failed projects.

## Safety

- **Never commit a keypair.** `.gitignore` blocks `*-keypair.json`, `id.json`, `.env`.
  Demo wallet keys live outside the repo in `~/.config/solana/townfund-demo/`.
- Devnet only. No mainnet, no real money.
