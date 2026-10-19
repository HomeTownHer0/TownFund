# TownFund

[![build-and-test](https://github.com/HomeTownHer0/TownFund/actions/workflows/build.yml/badge.svg)](https://github.com/HomeTownHer0/TownFund/actions/workflows/build.yml)

People who pick up short-term work, like landscaping, repairs or moving
help, often wait days or weeks to get paid, and the worker and the client
each have to trust the other with the money. Communities that want to fix
something local, like cleaning up trash in a park, have no simple way to
pool money with a guarantee that it comes back if the effort falls short.

**TownFund** is a USDC escrow on Solana for both. The money is locked in a
vault before the work starts. If the project reaches its target by the
deadline, the vault pays 95% to whoever does the work and 5% to a town
treasury. If it doesn't, every backer gets 100% back. Nobody holds a key to
the vaults; only the program's rules can release the money.

Built by Christian Arceo ([@HomeTownHer0](https://github.com/HomeTownHer0))
for Solana School, Fall 2026. Deployed on Devnet, 65 passing tests.

**Watch it live:** [hometownher0.github.io/TownFund](https://hometownher0.github.io/TownFund/)

## How it works

There are two kinds of projects:

- **Job (Creator-only):** the client opens the job and names the one
  contractor they're hiring, pays into escrow, and the contractor is paid.
- **Community:** neighbors chip in, and each backer votes with their first
  pledge on which listed contractor does the work. The contractor with the
  most votes is paid.

The rules:

- A pledge is at least 1 USDC, and you can add more later. A target is at
  least 10 USDC, and a project runs 1 to 60 days. In the live demo on
  Devnet, each project runs for 20 seconds so the whole thing fits on stage.
- After the deadline, anyone can send **settle**. If the target was met, the
  whole vault pays out 95/5; the fee rounds down so nothing is left behind.
  If it was missed, the project is marked Failed.
- The creator can **cancel** before the deadline.
- After a cancel or a missed target, each backer claims a **refund** of 100%
  of their pledge, no fee. Refunds open at the deadline even if nobody has
  run settle.
- **Tips** go straight to the contractor on a job, or to whoever opened a
  community project. No fee, no refunds, and they don't count toward the
  target.
- The **treasury** belongs to the program, and V1 has no way to withdraw
  from it.
- After a success, each backer can get a **soulbound badge**: a Metaplex
  Core NFT, frozen the moment it's made, that can't be sold or transferred.

## The live page

The live page reads straight from Devnet every few seconds and shows every
wallet, both project vaults and the town treasury. When the demo runs,
coins travel along the money map, and the program log underneath prints
each transaction as it lands. Looking at the page can't move any money.

On the map, **TIP** isn't a real account: a tip goes straight to the
contractor (or the community project's creator), but the coin passes
through TIP so you can see who tipped whom. The ★ next to the Client counts their badges.

Under the map, a labeled simulation previews the next version, described
below.

## Next version: the digital work ticket

The live page includes a simulation of the next version, built around the
Landscaping job. It plays on its own and is labeled as a simulation: it
doesn't touch Devnet or move any money, and none of it is in the V1 program
yet.

Day labor still runs on paper tickets. A boss on site signs off on the hours
and the work, and the worker's pay waits on that ticket getting back to an
office. The digital work ticket puts the ticket on-chain and pays the job in
phases:

1. **The Client funds the job** from a pay link on their phone. The full
   amount locks in escrow.
2. **Phase 1 (33.33%)** releases right away, so the contractor can buy
   supplies and start.
3. **The contractor checks in** on site, and the arrival time is recorded
   on-chain.
4. **Photos are the proof.** Before, progress and after photos go to
   permanent storage, and each photo's fingerprint is saved on-chain in the
   job's gallery.
5. **Phase 2 (33.33%)** releases when the boss on site approves the progress
   photos from their phone.
6. **Phase 3 (33.34%)** releases when the boss signs the work ticket: hours,
   work done, and whether to bring the contractor back.
7. **The signed ticket** joins the contractor's on-chain work history.

Each release pays 95% to the contractor and 5% to the treasury, and the
vault ends at exactly 0.

### The rules it adds

- **The Client sets the milestones when funding.** What proof each phase
  needs and how long the Client has to review it are saved on-chain, so
  neither side can change the rules later.
- **The boss's signature is the approval.** Signing from a phone releases
  the phase immediately, at any hour, with no trip back to an office.
- **Silence counts as approval.** If the Client doesn't respond before the
  review window ends, anyone can send settle and the program releases the
  phase on its own. The money can only go to the contractor and the
  treasury, so whoever sends it gains nothing.
- **A dispute freezes a phase.** If the Client disputes in time, that
  phase's money stays in escrow instead of releasing.
- **Check-in and check-out are timestamped** by the chain's own clock, so
  they can't be backdated.
- **Work history belongs to the worker.** Each signed ticket, with its hours
  and the "bring them back" answer, builds a record the contractor carries
  from job to job.

### How it would be built

| Step | Tool |
|---|---|
| Client pays from a text or a link | Solana Actions (blinks), with a QR code as an option in person |
| Boss signs without a seed phrase | An embedded wallet that signs in with Google or Face ID |
| Photos stored for good | Arweave permanent storage, with each photo's fingerprint saved on-chain |
| The job gallery | A Metaplex Core collection per job, one NFT per proof photo, held in the contractor's wallet |
| Phases, approvals and timeouts | New TownFund instructions on the same escrow vault |

Photos on permanent storage are public and can't be deleted, so faces,
addresses and card numbers on receipts have to be cropped or blurred before
upload.

## Check it on Devnet

Program: [`FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv`](https://explorer.solana.com/address/FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv?cluster=devnet)

Real transactions from the demo run on Oct 18, 2026: the "Landscaping project" job
and the "Trash clean up Buffalo Bayou Park" crowdfund.

| Step | Link | What you'll see |
|---|---|---|
| Client opens the job and hires Contractor 1099 | [tx](https://explorer.solana.com/tx/3iXxfcGzaguowQmwDQW416XT7GRWs7qtyHK4zKeGNJG7Ld7pzUu5AKK4uM72f5oYj9MLfjS4XWMFRujFq7QzwqVQ?cluster=devnet) | The client signs; the project and its empty vault are created |
| Client pays 10 USDC into escrow | [tx](https://explorer.solana.com/tx/3YW3HvK8GGwkMTxU3xQvw2xBBajUZLVGKEPZ5iVKhbQi3vmJhceKSoMaCVKarDnH4KKiRmpw4xh8tcaAR21EBVBv?cluster=devnet) | USDC moves from the client to the vault |
| Client tips the contractor | [tx](https://explorer.solana.com/tx/Vch3jzDcS98fx6ebvcBkYcQnR7xWr55fx9r5rZ4qucaP6zrAJ59MALz3iNfTgQhUfhUoETaqAPmYsgPhuyvs6Yo?cluster=devnet) | 1 USDC straight to the contractor, no fee |
| Settle (success) | [tx](https://explorer.solana.com/tx/NMBPgUpwoCBaAyNzde2mPVybnBw3z9oTMPLggqiyjntuiSSFkVtr1ZVK2kdqQMNXDcZQyqjvkRRfiGEvfJJJ5zt?cluster=devnet) | 9.5 USDC to the contractor, 0.5 to the treasury, vault empty |
| Soulbound badge | [NFT](https://explorer.solana.com/address/CmTduKhrR4hUo7x6j4Xii8YK38XpPfNZbjPiHCQLrkyj?cluster=devnet) | Owned by the client and frozen for good |
| Settle (failure) | [tx](https://explorer.solana.com/tx/5t55B47c75EWyBB2LwEsL12888wh7X88ytzndm2VXqE2JP8QeuQhP5oXYzu8rvuP31Qzn5NHgfgbruVX4mFmGfV2?cluster=devnet) | 5 of 20 USDC raised, so it's marked Failed and no money moves |
| Refund | [tx](https://explorer.solana.com/tx/3KpuoSkG4n32kfHwu2acsZFB2xQuVWDCDmyffyPoEiDcNLpKVSzsCvmPuMkgdAxjcKSD3NuihY92oMzXQGn3EsVd?cluster=devnet) | 5 USDC back to the backer, and the pledge receipt is closed |

Other addresses: Circle Devnet USDC `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`
([faucet](https://faucet.circle.com)), Metaplex Core
`CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d`, upgrade authority (my
wallet) `7cpjF6175EPGZ3gopVLn94h5iuUFvd2VZ9Xq9QKzxU21`.

## The program

Eight instructions:

- `initialize_project`: opens a project and its empty vault. Checks the
  target, duration, title (64 bytes max) and contractor list: one hired
  contractor (or none) on a job, 2 to 5 candidates on a community project.
- `initialize_treasury`: creates the treasury and its fee vault, once.
  There's nothing to configure, so whoever calls it gets no control.
- `pledge`: moves USDC from the backer's own account into this project's
  vault, before the deadline.
- `tip`: sends USDC straight to the job's contractor, or to whoever opened
  a community project.
- `settle`: pays out 95/5 to the real payee, or marks the project Failed.
- `cancel_project`: only the creator, only before the deadline.
- `refund`: 100% back to the backer, and the receipt is closed so it can't
  be claimed twice.
- `mint_backer_badge`: one badge per backer per successful project.

Every account TownFund creates and stores is a PDA, an address the program
works out from fixed seeds, so there's no private key behind any of them:

- **Project** `["project", creator, project_id]`: terms, totals, status,
  candidates and votes.
- **Vault** `["vault", project]`: the escrowed USDC, controlled by the
  project account.
- **Pledge receipt** `["pledge", project, backer]`: one backer's total and
  vote.
- **Treasury** `["treasury"]` and **fee vault** `["treasury_vault"]`: the
  5% fees.
- **Badge record** `["badge", project, backer]`: blocks a second badge.
- **Badge authority** `["badge_authority"]`: a PDA address with no data,
  set as every badge's update authority. It never signs, so badges can't be
  edited.

The badge NFT itself isn't a PDA: a fresh one-time key creates it and signs
once, and the badge record above is what stops a second one.

Only Circle's Devnet USDC is accepted, all math is checked so an overflow
stops the transaction, and every money movement emits an event.

### Tests

Each rule has a test that tries to break it:

- Only Devnet USDC: `rejects_any_mint_except_townfund_usdc`, `rejects_fake_usdc`
- A pledge only goes into its own project's vault: `rejects_sending_pledge_to_a_different_vault`
- You can't spend someone else's USDC: `rejects_spending_someone_elses_usdc_account`
- Only the real payee gets paid: `success_requires_the_payees_own_usdc_account`, `client_opened_job_pays_the_hired_contractor_not_the_client`, `community_success_pays_elected_contractor_not_creator`
- Tips reach the contractor, not the client: `tip_on_client_opened_job_goes_to_the_hired_contractor`
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
./scripts/build.sh                  # normal build + 65 tests
./scripts/build-devnet.sh           # demo build (10 s to 10 min projects) + 66 tests
cargo run -p townfund-cli -- demo   # full demo on Devnet, with an Explorer link per step
cargo run -p townfund-cli -- serve  # local runner behind the live page's Run demo button
```

GitHub runs both builds on every push. The demo funds four wallets
(Contractor 1099, Creator 2, Client, Backer) and runs two stories:

1. **"Landscaping project"**, a job: the Client opens it and hires
   Contractor 1099, pays 10 USDC and tips Contractor 1099 1 USDC. At the deadline, settle pays out 95/5 and the
   Client gets a badge.
2. **"Trash clean up Buffalo Bayou Park"**, a community crowdfund: the
   Backer pledges 5 of the 20 USDC target and tips Creator 2 1 USDC. Settle
   marks it Failed and the Backer gets the 5 back.

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
