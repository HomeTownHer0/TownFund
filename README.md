# TownFund

[![build-and-test](https://github.com/HomeTownHer0/TownFund/actions/workflows/build.yml/badge.svg)](https://github.com/HomeTownHer0/TownFund/actions/workflows/build.yml)

TownFund is a Solana-native platform that replaces outdated temp-agency
friction with USDC escrow and digital approvals, so workers can get paid the
same day they finish short-term jobs. It also lets communities crowdfund
local needs and routes the money directly to workers, with a 5% fee going
to a town treasury that will sit in a Jupiter USDC yield vault. It's
intended to be built as a compliant US marketplace that uses Solana for
fast, low-cost settlement rather than pure decentralization.

Built by Christian Arceo ([@HomeTownHer0](https://github.com/HomeTownHer0))
for Solana School, Fall 2026.

## What V1 does

V1 is the money layer everything else sits on: the escrow. People pledge
USDC to a project, and the money sits in an escrow vault until the
deadline. If the project hits its target, the vault pays out: 95% to
whoever does the work, 5% to the town treasury. If it misses, everyone gets
100% of their money back. Nobody holds the money in between, me included.
The program has no instruction that moves escrowed money any other way (the
one caveat is the upgrade authority, covered under known limitations).

**Status:** V1 is done. The program has 62 passing tests, it's deployed on
Devnet, and the full demo runs on Devnet from a button on the
[live page](https://hometownher0.github.io/TownFund/), resetting itself
after each run (Oct 11, 2026).

### Built now vs. next

| | Status |
|---|---|
| USDC escrow on Solana | Built, live on Devnet |
| Communities crowdfund local needs | Built (Creator-only and Community projects) |
| Money goes straight to the worker | Built: a Community project pays the contractor the backers picked |
| 5% fee to a town treasury | Built (the 95/5 split) |
| Treasury earns yield in a Jupiter USDC vault | Next. Fees sit in the treasury vault for now, and the program already saves room for the "door" this needs |
| Short-term jobs with digital approval and same-day pay | Next. This is the milestones step: the job is escrowed, the hirer approves, the worker gets paid |
| Compliant US marketplace | Long-term. That's a business and legal track (identity checks, money-transmission rules), not code |

## How it works

There are two kinds of projects:

- **Creator-only:** the person who started the project gets paid.
- **Community:** backers vote on who does the job.

The rules:

- A pledge is at least 1 USDC, and you can add more later. A project's
  target is at least 10 USDC and it runs 1 to 60 days. (The Devnet demo
  build also allows 10 seconds to 10 minutes, so a whole project fits in a
  live demo.)
- After the deadline, anyone can trigger **settle**. If the target was hit,
  the vault pays out once, 95% to the payee and 5% to the treasury. The fee
  rounds down so nothing is left in the vault, and going over the target is
  fine. If the target was missed, the project is marked Failed and each
  backer can claim a full refund with no fee. Refunds open even if nobody
  runs settle.
- The creator can **cancel** before the deadline, and refunds open right away.
- **Tips** go straight to the creator. No fee, no refunds, and they don't
  count toward the target.
- The **treasury** belongs to the program itself (a PDA, so there's no
  private key for anyone to hold). In V1 there's no way to withdraw from it.
- After a project succeeds, each backer can get a **soulbound badge**, an
  NFT that can never be sold or transferred.

If you work around machines, think of the vault as a cell with an
interlocked door. The program is the safety PLC: the door only opens for
"target met after the deadline" (pay out 95/5) or "target missed or
cancelled" (refund). There's no override button, and every cycle is logged
on-chain.

## For reviewers

### Watch it live

**[hometownher0.github.io/TownFund](https://hometownher0.github.io/TownFund/)**

The live page reads straight from Devnet every few seconds and shows every
wallet, both project vaults, and the town treasury. When the demo runs,
coins travel along the money map and each box updates when its coin lands.
Under the map, the TownFund program log prints each transaction as it comes
in: who paid whom, then the program's own log lines the way Solana recorded
them. Nothing to install, and just looking at the page can't move any money.

Two things on the map that might need explaining: **TIP** isn't an account.
On-chain, a tip goes straight from the backer to the creator, but the coin
passes through TIP so you can see who tipped whom, and TIP shows the total
for that run. The ★ next to a backer counts their badges and links to the
newest one on Explorer.

### Check it on Devnet yourself

Program: [`FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv`](https://explorer.solana.com/address/FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv?cluster=devnet)

These are real transactions from the demo run on Oct 11, 2026, where
"Pothole on Westheimer" succeeds and "Fix Local Park" fails:

| Step | Link | What you'll see |
|---|---|---|
| Pledge 10 USDC into escrow | [tx](https://explorer.solana.com/tx/3GE2pcWW8V9k5LRU9U3oDhuJRB8gwH67n8JX2WKRAsBK2byoguxM4dHZKis8pvESsMMowkY7fxKefVpmJ9UxbW97?cluster=devnet) | USDC moves from the backer to the vault |
| Settle (success) | [tx](https://explorer.solana.com/tx/4ic5eQf3emsp729XR8AXr9L1M63197BXJDXChkGag5LJREhy55SUU2QbevCLB2BuKyivDcobC5WBC1fFdvr7sQ8n?cluster=devnet) | 9.5 USDC to the creator, 0.5 to the treasury, vault empty |
| Soulbound badge | [NFT](https://explorer.solana.com/address/EKjbBup2erEZbWqFxRcG4FXF9wdYqav5aDAbWNoFFsNZ?cluster=devnet) | "TownFund Backer: Pothole on Westheimer", owned by the backer, frozen for good |
| Settle (failure) | [tx](https://explorer.solana.com/tx/45GYQyAcMyEgwygw1CPmLcrBdmcT2EPvkPPdcTahxL6TfqziBSAyVokNYk9ubMQf5hxRQxgvPMkaQi3754pNCQWS?cluster=devnet) | Only 5 of 20 USDC raised, so it's marked Failed and no money moves |
| Refund | [tx](https://explorer.solana.com/tx/66uuFZTt4b2AmRQ78jLpzmTK3FqJTuQ81w1wjva5TpjTym2GeKVUuuQTR4mqr7uHHKZpVa2Yr9MLJndFEh4HHH4s?cluster=devnet) | 5 USDC back to the backer, and the pledge receipt is closed |

### The instructions

The program has eight:

- `initialize_project`: the creator opens a project and its empty vault.
  It checks the target, the duration, the title (64 bytes max) and the
  contractor list, and only accepts USDC.
- `initialize_treasury`: creates the treasury and its fee vault. Anyone can
  call it, once, but there's nothing to configure, so being first doesn't
  give you any control.
- `pledge`: a backer moves USDC into the vault. It has to be before the
  deadline, from the backer's own account, and into this project's vault,
  not some other one.
- `tip`: anyone sends USDC straight to the creator.
- `settle`: anyone, after the deadline. Pays 95/5 to the real payee, or
  marks the project Failed.
- `cancel_project`: only the creator, only before the deadline.
- `refund`: a backer takes back 100% once refunds are open. The receipt is
  closed, so it can't be claimed twice.
- `mint_backer_badge`: anyone can pay to mint a backer's badge after a
  success. One per backer per project.

### Accounts

Every account is a PDA, an address the program works out from fixed seeds.
There's no private key behind any of them, so only the program's rules can
move what's inside.

- **Project** `["project", creator, project_id]`: the terms, totals,
  status, candidates and votes.
- **Vault** `["vault", project]`: the escrowed USDC. The project account
  controls it.
- **Pledge receipt** `["pledge", project, backer]`: how much one backer put
  in, and their vote.
- **Treasury** `["treasury"]` and **fee vault** `["treasury_vault"]`: the
  5% fees. There's a `door_authority` field saved for later; it's always
  empty in V1.
- **Badge record** `["badge", project, backer]`: which NFT a backer got.
  Its existence blocks a second one.
- **Badge authority** `["badge_authority"]`: the update authority on every
  badge. It never signs anything, so badges can't be edited.

Each account also has a version number and some spare space, so later
versions can add fields without breaking old data. Every money movement
emits an event (`Pledged`, `Tipped`, `Settled`, `Refunded`, and so on).

### How I know it's safe

Each rule has a test that tries to break it:

- Only Circle's Devnet USDC is accepted: `rejects_any_mint_except_townfund_usdc`, `rejects_fake_usdc`
- A pledge can only go into its own project's vault: `rejects_sending_pledge_to_a_different_vault`
- You can't spend someone else's USDC: `rejects_spending_someone_elses_usdc_account`
- Only the real payee gets paid: `success_requires_the_payees_own_usdc_account`, `community_success_pays_elected_contractor_not_creator`
- The 95/5 split is exact and leaves nothing behind: `fee_is_rounded_down_and_nothing_is_left_behind`
- No settling, refunding or minting a badge twice: `cannot_settle_twice`, `cannot_refund_twice`, `no_second_badge_for_the_same_backer`
- Refunds only go to the backer who pledged: `cannot_claim_someone_elses_refund`
- No refunds once a project succeeds: `no_refunds_while_active_or_after_success`
- One wallet, one vote, locked after the first pledge: `community_first_pledge_casts_one_vote`, `community_top_up_keeps_vote_locked`
- The badge can't be transferred: `badge_cannot_be_transferred`
- A failed transaction changes nothing: `rejects_pledge_with_more_usdc_than_backer_has`

All the math is checked, so an overflow stops the transaction instead of
wrapping around, and release builds keep those checks on. The tests run on
LiteSVM against the compiled program, with the real Metaplex Core program
copied from Devnet. For several of the rules I also broke the rule on
purpose in the code to make sure its test would catch it.

### Where things are

- `programs/townfund/src/lib.rs`: the entry point, the list of instructions
- `programs/townfund/src/instructions/`: one file per instruction, account checks first, then the logic
- `programs/townfund/src/state.rs`: what each stored account looks like
- `programs/townfund/src/constants.rs`: limits, seeds, the USDC mint
- `programs/townfund/src/error.rs`: every reason an instruction can be refused
- `programs/townfund/tests/`: the 62 tests (63 in the demo build)
- `cli/src/main.rs`: the `townfund` command-line tool: the demo, the reset, and the local runner behind the Run demo button
- `site/`: the live page, published to GitHub Pages

## Known limitations

These are the next things to fix, not hidden bugs:

1. A Community contractor gets paid at settle, before the work is done.
   Milestone payments will fix that.
2. One wallet = one vote, so someone could buy votes with extra wallets.
3. Treasury money stays locked until there's a "door" to spend it, like a
   multisig or a community vote.
4. On Devnet my wallet is the upgrade authority, so I could replace the
   program and change the rules for money in escrow. Before any real money,
   that has to move to a multisig or be revoked.
5. The rent for a project and its vault (about 0.006 SOL) isn't returned
   after the project ends.
6. USDC sent straight to a vault without using `pledge` isn't tracked by a
   receipt. On success it pays out with everything else; on failure it's stuck.
7. Tips on a Community project go to the creator, not the elected contractor.

## Progress

- [x] Anchor 1.2 workspace, keypair-safe `.gitignore`, GitHub test check
- [x] `initialize_project`, the project account and its vault
- [x] Treasury, `pledge` with Community voting, `tip`
- [x] `settle` (95/5), `refund`, `cancel_project`
- [x] Soulbound backer badge with Metaplex Core
- [x] Deployed to Devnet (demo build): [`FdaMK3…PXrv`](https://explorer.solana.com/address/FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv?cluster=devnet)
- [x] `townfund` CLI that runs the full Demo Day story with Explorer links
- [x] Full demo run on Devnet (links above)
- [x] Live status page on GitHub Pages
- [x] Run demo button that starts the demo from the page and resets after each run
- [ ] Demo Day
- [ ] After that: the treasury door and Jupiter yield vault, short-term jobs with approvals and same-day pay, and the website

## Deployment

- **Cluster:** Devnet only
- **Program ID:** `FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv`
- **Build:** demo build (`./scripts/build-devnet.sh`)
- **Upgrade authority:** `7cpjF6175EPGZ3gopVLn94h5iuUFvd2VZ9Xq9QKzxU21` (my wallet)
- **USDC:** Circle Devnet USDC `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` ([faucet](https://faucet.circle.com))
- **Metaplex Core:** `CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d`

## Build and test

You'll need Solana CLI 4.1.2, Anchor 1.2.0 and Rust 1.89 or newer.

```bash
./scripts/build.sh          # normal build + 62 tests
./scripts/build-devnet.sh   # demo build (allows 10 s to 10 min projects) + 63 tests
```

Both build the program for SBPF v0 with platform-tools v1.54 and run the
tests on LiteSVM, an in-memory Solana chain. GitHub runs both on every push.

## Running the demo

```bash
cargo run -p townfund-cli -- demo
```

That one command runs the whole Demo Day story against the live program and
prints an Explorer link for every step:

1. It funds four demo wallets (Creator 1, Creator 2, Backer 1, Backer 2)
   from my main wallet: SOL for fees, USDC for pledges and tips.
2. **"Pothole on Westheimer"** (Creator-only, opened by Creator 1)
   succeeds. Backer 1 pledges 10 USDC and tips Creator 1 1 USDC. After the
   deadline, settle pays out 95/5 and Backer 1 gets a soulbound badge.
3. **"Fix Local Park"** (Community, opened by Creator 2) fails. Backer 2
   pledges 5 of the 20 USDC target and tips Creator 2 1 USDC. After the
   deadline, settle marks it Failed and Backer 2 gets the 5 back. The tip
   stays with Creator 2, since tips aren't refundable.
4. About 15 seconds later it resets: creators emptied, backers refilled,
   ready to go again. The treasury keeps its fees, since V1 has no way to
   withdraw them.

Each step pauses 3 seconds so the live page can show every move. A whole
run takes about a minute.

On Demo Day I start the same run from the **Run demo** button on the live
page. `cargo run -p townfund-cli -- serve` starts a small runner on my
computer, and it signs every transaction there. A Terminal pane on the page
shows the runner's output next to the program log. On anyone else's
computer, the button just explains that the demo runs from the presenter's
computer.

## The backer badge

After a project succeeds, `mint_backer_badge` gives each backer a soulbound
NFT (Metaplex Core) called "TownFund Backer: \<project title\>". Core's
Permanent Freeze plugin freezes it the moment it's made, and there's no
authority that could ever unfreeze it, so it can't be transferred or sold.
It's one per backer per project, and failed projects don't get one.

## Safety

- No keys in this repo. `.gitignore` blocks keypair files, and every wallet
  key stays on my computer.
- The Run demo button can't move money on its own. Only the runner on my
  computer can start the demo, and it signs with keys that never leave it.
- Devnet only. No mainnet, no real money.
