# TownFund

[![build-and-test](https://github.com/HomeTownHer0/townfund/actions/workflows/build.yml/badge.svg)](https://github.com/HomeTownHer0/townfund/actions/workflows/build.yml)

A Solana program for community fundraising and a community job board.
Backers pledge USDC to a project. If it hits its target by the deadline,
the money pays out (95% to the payee, 5% to the TownFund treasury). If it
misses, every backer gets a full refund. No one, including the builders,
can take money out any other way.

Built by Christian Arceo ([@HomeTownHer0](https://github.com/HomeTownHer0))
in Solana School Fall 2026

> **Status:** scaffold only. The code in `programs/townfund` is still
> Anchor's sample counter. It is replaced in slice 1.

## How it works (V1, "Creators + Backers")

| Project kind | Who gets paid on success |
|---|---|
| **Creator-only** | The creator |
| **Community** | A contractor elected by backers (2–5 candidates, one wallet = one vote) |

- **Pledge:** at least 1 USDC. Target at least 10 USDC. Duration 1–60 days.
- **Settle** (anyone can trigger it after the deadline):
  - Target hit → whole vault pays out once: **95% payee / 5% treasury**. Overfunding allowed.
  - Target missed, or creator cancelled → each backer claims a **100% refund**.
- **Tips** go straight to the creator. No fee, not refundable.
- **Treasury** is owned by the program (a PDA, no private key). No withdrawals in V1.

### Known V1 limitations (roadmap, not bugs)
1. A Community contractor is paid at settle, before the work is done. Milestones fix this.
2. One wallet = one vote, so votes can be bought with extra wallets.
3. Treasury funds stay locked until a "door" (multisig or community vote) is built.
4. On Devnet, the upgrade authority can replace the program.

## Build plan

| Step | What | Status |
|---|---|---|
| Scaffold | Anchor 1.2 workspace, keypair-safe `.gitignore`, passing test, GitHub CI check | ✅ |
| Slice 1 | `initialize_project`, Project account, vault | ⏳ |
| Slice 2 | Treasury, `pledge`, `tip` | ⏳ |
| Slice 3 | `settle`, `refund`, `cancel_project` | ⏳ |
| Devnet | Deploy, with a mock USDC mint | ⏳ |
| Roadmap | Community election, treasury door, milestones, SBTs, website | after Oct 16 |

## Build and test

Requires Solana CLI 4.1.2, Anchor 1.2.0, Rust ≥ 1.89.

```bash
./scripts/build.sh
```

That builds the program for SBPF v0 with platform-tools v1.54, then runs
the tests on LiteSVM (an in-memory Solana chain).

## Safety

- **Never commit a keypair.** `.gitignore` blocks `*-keypair.json`, `id.json`, `.env`.
- Devnet only. No mainnet, no real money.
