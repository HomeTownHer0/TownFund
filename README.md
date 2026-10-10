# TownFund

[![build-and-test](https://github.com/HomeTownHer0/townfund/actions/workflows/build.yml/badge.svg)](https://github.com/HomeTownHer0/townfund/actions/workflows/build.yml)

A Solana program for community fundraising and a community job board.
Backers pledge USDC to a project. If it hits its target by the deadline,
the money pays out (95% to the payee, 5% to the TownFund treasury). If it
misses, every backer gets a full refund. No one, including the builders,
can take money out any other way.

Built by Christian Arceo ([@HomeTownHer0](https://github.com/HomeTownHer0))
in Solana School Fall 2026

> **Status:** V1 program complete (62 tests), live on Devnet, and the full
> demo has run successfully on Devnet (Oct 10, 2026).

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
  - Target hit → whole vault pays out once: **95% payee / 5% treasury**. Overfunding allowed.
  - Target missed, or creator cancelled → each backer claims a **100% refund**.
- **Tips** go straight to the creator. No fee, not refundable.
- **Treasury** is owned by the program (a PDA, no private key). No withdrawals in V1.
- **Backer badge:** after success, each backer can receive a soulbound NFT
  (see below).

### Known V1 limitations (roadmap, not bugs)
1. A Community contractor is paid at settle, before the work is done. Milestones fix this.
2. One wallet = one vote, so votes can be bought with extra wallets.
3. Treasury funds stay locked until a "door" (multisig or community vote) is built.
4. On Devnet, the upgrade authority can replace the program.

## Build plan

| Step | What | Status |
|---|---|---|
| Scaffold | Anchor 1.2 workspace, keypair-safe `.gitignore`, passing test, GitHub CI check | ✅ |
| Slice 1 | `initialize_project`, Project account, vault | ✅ |
| Slice 2 | Treasury, `pledge` (+ Community vote), `tip` | ✅ |
| Slice 3 | `settle` (95/5), `refund`, `cancel_project` | ✅ |
| Badge | Soulbound backer NFT (Metaplex Core) after success | ✅ |
| Devnet | Deployed (demo build): [`FdaMK3…PXrv`](https://explorer.solana.com/address/FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv?cluster=devnet) | ✅ |
| Demo tool | `townfund` CLI: full Demo Day story with Explorer links (verified on a local chain) | ✅ |
| Devnet run | Full demo on Devnet: [success + badge](https://explorer.solana.com/tx/2Kgegw53mVKEV1YpyoRpsLt1oYGFUpocKb8qtg72tbrHxJJjVz7WbSvrXWRUcXTcfjrjRJWQbQSWYqtYzpyqdpeb?cluster=devnet), [failure + refund](https://explorer.solana.com/tx/4dx7i3uEkWAaaYj2Fe6j8ZxQ4p3BQu8MY6FyRPgsXMVmiWRT6iQ7HgbGBqf3Wo4BY2xD1ifxD2SfRabLP51BekGm?cluster=devnet) | ✅ |
| Demo Day | October 16, 2026 | ⏳ |
| Roadmap | Treasury door, milestones, website | after Oct 16 |

## Deployment

| | |
|---|---|
| Cluster | Devnet only |
| Program ID | `FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv` |
| Build | Demo build (`./scripts/build-devnet.sh`) |
| Upgrade authority | `7cpjF6175EPGZ3gopVLn94h5iuUFvd2VZ9Xq9QKzxU21` (owner's wallet) |
| USDC | Circle Devnet USDC `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` |
| Metaplex Core | `CoREENxT6tW1HoK8ypY1SxRMZTcVPm7R94rH4PZNhX7d` |

## Decisions log

Changes from the original v2.5 design, decided by the owner:

| Date | Decision | Why |
|---|---|---|
| Oct 10 | Refund rent goes back to the backer | Simpler; no donation option |
| Oct 10 | Version byte + spare room on every stored account; event for every money movement | Future fields without breaking stored data; history for the website |
| Oct 10 | Treasury has an empty "door authority" field; no spending in V1 | Spending must never depend on one person; the door gets built later |
| Oct 10 | Community vote cast with the first pledge, locked for top-ups | One wallet = one vote with no extra step |
| Oct 10 | Use **Circle's Devnet USDC** instead of our own mock mint | Use the faucet that already works (reverses the original mock-mint plan) |
| Oct 10 | **Soulbound backer badge** (Metaplex Core) included for Oct 16 | Owner priority; Metaplex Core as taught in class (reverses moving SBTs to later) |
| Oct 10 | Demo build allows **10 s–10 min** projects | A full run must fit in a 5-minute presentation |
| Oct 10 | Demo Day shows Creator-only **success** and Community **failure/refund** | Both outcomes in one demo; Community code is built and tested |

## Build and test

Requires Solana CLI 4.1.2, Anchor 1.2.0, Rust ≥ 1.89.

```bash
./scripts/build.sh
```

That builds the program for SBPF v0 with platform-tools v1.54, then runs
the tests on LiteSVM (an in-memory Solana chain).

```bash
./scripts/build-devnet.sh
```

Builds the Devnet / Demo Day version: identical, plus a `demo` feature that
also accepts projects from 10 seconds to 10 minutes long, so a full
create -> pledge -> settle run fits in a live demo.

**Demo Day plan:** a Creator-only project succeeds live (pledge, escrow,
settle 95/5), and a Community project fails and refunds every backer 100%.

## Run the demo (Devnet)

```bash
cargo run -p townfund-cli -- demo --seconds 20
```

One command runs the Demo Day story against the live program and prints an
Explorer link for every step:

1. Creates (once) and funds three demo wallets in `~/.config/solana/townfund-demo/`
   (creator, backer1, backer2) from your main wallet: SOL for fees, USDC for pledges
2. **Individual project succeeds:** pledge 10 USDC into escrow, tip 1 USDC,
   deadline passes, settle 95/5, soulbound badge minted to the backer
3. **Community project fails:** pledge 5 of 20 USDC, deadline passes,
   settle marks it Failed, backer refunded 100%

Other commands: `setup-wallets`, `init-treasury`, `demo-success`, `demo-fail`,
`status <project>`, `balances`, `recycle` (sends demo USDC back to your main
wallet so you can rehearse again). Needs ~16 USDC in your main wallet the
first time (Circle faucet); each run only consumes the 5% fee.

## Backer badge

After a project succeeds, `mint_backer_badge` gives each backer a
**soulbound NFT** (Metaplex Core): "TownFund Backer: <project title>".
It's frozen forever by Core's Permanent Freeze plugin with no authority
that could unfreeze it, so it can never be transferred or sold. One per
backer per project; never for failed projects. Tests run against the real
Metaplex Core program (`tests/fixtures/mpl_core.so`, dumped from Devnet).

## Token

TownFund accepts exactly one token: Circle's Devnet USDC
(`4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU`, get it at
[faucet.circle.com](https://faucet.circle.com)). Any other token is refused.

## Safety

- **Never commit a keypair.** `.gitignore` blocks `*-keypair.json`, `id.json`, `.env`.
- Devnet only. No mainnet, no real money.
