# TownFund

[![build-and-test](https://github.com/HomeTownHer0/TownFund/actions/workflows/build.yml/badge.svg)](https://github.com/HomeTownHer0/TownFund/actions/workflows/build.yml)

People who pick up short-term work, like landscaping, repairs or moving
help, often wait days or weeks to get paid, and the worker and the client
each have to trust the other with the money. Communities that want to fix
something local, like a pothole, have no simple way to pool money with a
guarantee that it comes back if the effort falls short.

**TownFund** is a USDC escrow on Solana for both. The money is locked in a
vault before the work starts. If the project reaches its target by the
deadline, the vault pays 95% to whoever does the work and 5% to a town
treasury. If it doesn't, every backer gets 100% back. Nobody holds the money
in between, me included.

Built by Christian Arceo ([@HomeTownHer0](https://github.com/HomeTownHer0))
for Solana School, Fall 2026. Deployed on Devnet, 62 passing tests.

**Watch it live:** [hometownher0.github.io/TownFund](https://hometownher0.github.io/TownFund/)

## How it works

There are two kinds of projects:

- **Job (Creator-only):** the contractor posts the job, the client pays
  into escrow, and contractor is paid.
- **Community:** neighbors chip in, and each backer pays
  into escrow, and contractor is paid. 

The rules:

- A pledge is at least 1 USDC, and you can add more later. A target is at
  least 10 USDC, and a project runs 1 to 60 days (the Devnet demo build also
  allows 20 seconds, so a whole project fits in a live demo).
- After the deadline, account will **settle**. If the target was met, the
  whole vault pays out 95/5; the fee rounds down so nothing is left behind.
  If it was missed, the project is marked Failed.
- The creator can **cancel** before the deadline.
- After a cancel or a missed target, each backer claims a **refund** of 100%
  of their pledge, no fee. Refunds open at the deadline even if nobody has
  run settle.
- **Tips** go straight to whoever opened the project. No fee, no refunds,
  and they don't count toward the target.
- The **treasury** belongs to the program, and V1 has no way to withdraw
  from it.
- After a success, each backer get's a **soulbound badge**: a Metaplex
  Core NFT, frozen the moment it's made, that can't be sold or transferred.

## The live page

The live page reads straight from Devnet every few seconds and shows every
wallet, both project vaults & the town treasury. When the demo runs,
coins travel along the money map. Then, the program log underneath prints
each transaction as it lands. Looking at the page can't move any money.

On the map, **TIP** isn't a real account: a tip goes straight to whoever
opened the project, but the coin passes through TIP so you can see who
tipped whom. The ★ next to the Client counts their badges.

## Check it on Devnet

Program: [`FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv`](https://explorer.solana.com/address/FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv?cluster=devnet)

Real transactions from the demo run on Oct 17, 2026:

| Step | Link | What you'll see |
|---|---|---|
| Client pays 10 USDC into escrow | [tx](https://explorer.solana.com/tx/5NFAmsrzq9P3F3deyBqm9A3mE7J2R9oMXFSGfHV8azyBy4j3cSHjK2VMyA3fNice38WwsG8Vws1nPbVUmZeUU9Nc?cluster=devnet) | USDC moves from the client to the vault |
| Settle (success) | [tx](https://explorer.solana.com/tx/s4CLdq6ZYrFtMyWvSXXyNeBe5nZSNAmp4oce1zuk7oNasxqsVXVG8KTYboKJDMaRnU6HATDnZzVsVzrcahZ5JnC?cluster=devnet) | 9.5 USDC to the contractor, 0.5 to the treasury, vault empty |
| Soulbound badge | [NFT](https://explorer.solana.com/address/9Q2YcrsQZACfyY7yuGK45nxRaercsPDFG9nvcRiDRBKN?cluster=devnet) | Owned by the client and frozen for good |
| Settle (failure) | [tx](https://explorer.solana.com/tx/5CAAf9W6fDRvNodHAkWU5DcVDD9tZcWvny5ErxQYCFKH94dmpb2K98Rd5xMnNK9uJkmPYxzkm8WmQSuPsxNuFzCt?cluster=devnet) | 5 of 20 USDC raised, so it's marked Failed and no money moves |
| Refund | [tx](https://explorer.solana.com/tx/24YyJ4RMnmuYYE7icQhsRnCm6gNHbSMf35rADYqykcyaHHNjyKYmVTb9yWv1T3tmBKbzaZApwT4rgGMp57DuyW5A?cluster=devnet) | 5 USDC back to the backer, and the pledge receipt is closed |

Other addresses: Circle Devnet USDC `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
([faucet](https://faucet.circle.com)), Metaplex Core
`CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d`, upgrade authority (my
wallet) `7cpjF6175EPGZ3gopVLn94h5iuUFvd2VZ9Xq9QKzxU21`.

## The program

Eight instructions:

- `initialize_project`: opens a project and its empty vault. Checks the
  target, duration, title (64 bytes max) and contractor list.
- `initialize_treasury`: creates the treasury and its fee vault, once.
  There's nothing to configure, so whoever calls it gets no control.
- `pledge`: moves USDC from the backer's own account into this project's
  vault, before the deadline.
- `tip`: sends USDC straight to whoever opened the project.
- `settle`: pays out 95/5 to the real payee, or marks the project Failed.
- `cancel_project`: only the creator, only before the deadline.
- `refund`: 100% back to the backer, and the receipt is closed so it can't
  be claimed twice.
- `mint_backer_badge`: one badge per backer per successful project.

Every account is a PDA, an address the program works out from fixed seeds,
so there's no private key behind any of them:

- **Project** `["project", creator, project_id]`: terms, totals, status,
  candidates and votes.
- **Vault** `["vault", project]`: the escrowed USDC, controlled by the
  project account.
- **Pledge receipt** `["pledge", project, backer]`: one backer's total and
  vote.
- **Treasury** `["treasury"]` and **fee vault** `["treasury_vault"]`: the
  5% fees.
- **Badge record** `["badge", project, backer]`: blocks a second badge.
- **Badge authority** `["badge_authority"]`: never signs, so badges can't
  be edited.

Only Circle's Devnet USDC is accepted, all math is checked so an overflow
stops the transaction, and every money movement emits an event.

### Tests

Each rule has a test that tries to break it:

- Only Devnet USDC: `rejects_any_mint_except_townfund_usdc`, `rejects_fake_usdc`
- A pledge only goes into its own project's vault: `rejects_sending_pledge_to_a_different_vault`
- You can't spend someone else's USDC: `rejects_spending_someone_elses_usdc_account`
- Only the real payee gets paid: `success_requires_the_payees_own_usdc_account`, `community_success_pays_elected_contractor_not_creator`
- The 95/5 split leaves nothing behind: `fee_is_rounded_down_and_nothing_is_left_behind`
- Nothing happens twice: `cannot_settle_twice`, `cannot_refund_twice`, `no_second_badge_for_the_same_backer`
- Refunds only go to the backer who pledged: `cannot_claim_someone_elses_refund`
- No refunds before the deadline or after a success: `no_refunds_while_active_or_after_success`
- One wallet, one vote, locked after the first pledge: `community_first_pledge_casts_one_vote`, `community_top_up_keeps_vote_locked`
- The badge can't be transferred: `badge_cannot_be_transferred`
- A failed transaction changes nothing: `rejects_pledge_with_more_usdc_than_backer_has`

The tests run on LiteSVM against the compiled program, with the real
Metaplex Core program copied from Devnet. For several rules I also broke
the rule on purpose in the code to make sure its test caught it.

## Build, test and run

You'll need Solana CLI 4.1.2, Anchor 1.2.0 and Rust 1.89 or newer.

```bash
./scripts/build.sh                  # normal build + 62 tests
./scripts/build-devnet.sh           # demo build (10 s to 10 min projects) + 63 tests
cargo run -p townfund-cli -- demo   # full demo on Devnet, with an Explorer link per step
cargo run -p townfund-cli -- serve  # local runner behind the live page's Run demo button
```

GitHub runs both builds on every push. The demo funds four wallets
(Contractor 1099, Creator 2, Client, Backer) and runs two stories:

1. **"Landscaping project"**, a job: the Client pays 10 USDC and tips
   Contractor 1099 1 USDC. At the deadline, settle pays out 95/5 and the
   Client gets a badge.
2. **"Pothole on Westheimer"**, a community crowdfund: the Backer pledges 5
   of the 20 USDC target and tips Creator 2 1 USDC. Settle marks it Failed
   and the Backer gets the 5 back.

About 15 seconds later it resets the wallets so it can run again. The whole
run takes about a minute. The Run demo button only works on my computer,
where `serve` signs every transaction; on anyone else's it explains that.

## Where things are

- `programs/townfund/src/lib.rs`: entry point and the list of instructions
- `programs/townfund/src/instructions/`: one file per instruction, account checks first, then the logic
- `programs/townfund/src/state.rs`, `constants.rs`, `error.rs`: accounts, limits and seeds, and every reason an instruction can be refused
- `programs/townfund/tests/`: the tests
- `cli/src/main.rs`: the `townfund` command-line tool (demo, reset, runner)
- `site/`: the live page, published to GitHub Pages

## Safety

- No keys in this repo. `.gitignore` blocks keypair files, and every wallet
  key stays on my computer.
- Devnet only. No mainnet, no real money.
