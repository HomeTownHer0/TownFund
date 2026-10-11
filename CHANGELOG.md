# Changelog

Every revision to TownFund, newest first. Times are US Central. Each entry
links to its commit, which shows exactly what changed.

The program deployed on Devnet (`FdaMK3…PXrv`) was built from commit
**b86740d**. Later entries changed only the demo tool, tests and docs, not
the program (verified: same source, same 409,488-byte build).

## 2026-10-11

### 08:45 · Both backers tip, through a TIP junction on the map
- Replaces the 08:25 "Supporter" wallet (owner's call): each backer tips the
  creator 1 USDC, Backer 1 on "Pothole on Westheimer" and Backer 2 on "Fix
  Local Park". Funding: backer1 11, backer2 6 USDC. `recycle` also empties
  the old supporter wallet if it exists.
- Live page: a TIP junction between the two vaults. Each backer has its own
  tip road into TIP, and one road runs from TIP to the creator, so every
  wallet's route to the creator is drawn. The coin travels the tipping
  backer's road, then TIP to the creator. TIP shows the total tipped this
  run. (On-chain a tip goes straight to the creator; TIP is a route marker,
  not an account.)

### 08:25 · Compact badge star
- The large badge card is replaced by a small ★ count (successful projects
  that wallet backed) on the map and in the backer list, linking to the
  newest badge NFT on Explorer.
- CLI writes the demo wallets' public addresses to `demo-wallets.json`
  (git-ignored).

### 07:45 · Live page: works within Devnet's rate limits
- Fixed: on the public Devnet server the page showed "Reconnecting" with
  HTTP 429 (too many requests). It sent ~13 requests every 2.5 s, including
  5 account searches.
- Now it asks one cheap question every 2.5 s ("any new TownFund
  transaction?") and reloads only when something happened, or every 20 s,
  using 1 account search instead of 5. About 16 requests per 10 s.
- Root cause found by testing from a real browser: the public Devnet
  server refuses several calls of the same kind in one request ("Too many
  requests for a specific RPC call"). Wallet token accounts are now looked
  up once, one at a time; every refresh reads all six balances with a single
  getMultipleAccounts. Feed transactions are fetched one at a time.
- Files are version-stamped on publish, so a browser can't mix a cached
  old file with a new one.
- If Devnet still says "slow down", the page backs off (5–30 s) and says
  "Devnet is busy, retrying in N s".
- Fixed the published link: GitHub Pages paths are case-sensitive
  (`/TownFund/`).

## 2026-10-10

### 20:05 · Live status page
- `site/`: read-only page that shows every demo wallet, both project vaults
  and the town treasury straight from Devnet, refreshing every 2.5 s. Money
  map with coins moving along each road, per-run change tags, countdowns,
  step-by-step project progress, badge card, activity feed with Explorer links.
- Published to GitHub Pages by `.github/workflows/pages.yml`.
- Tested in a headless browser against a local chain during full demo runs
  (pledge, tip, settle, badge, failure, refund) and at phone width.

### 19:15 · Audit pass ([ed2fe39](https://github.com/HomeTownHer0/townfund/commit/ed2fe39))
- README rewritten for reviewers: Devnet links, instruction table, account
  map, each security property mapped to the test that proves it, code map.
- Corrected: "no other way out" now notes the upgrade-authority caveat;
  per-project rent is ~0.006 SOL (was stated as ~0.004).
- Added known limitations: rent not reclaimed, untracked direct vault
  deposits, Community tips go to the creator.
- Demo tool: `demo-fail` creates the treasury if missing; settle retry also
  matches fault 0x1780.
- Tests: sample title changed to "Pothole on Westheimer".

### 19:03 · Demo titles ([2e5f8c2](https://github.com/HomeTownHer0/townfund/commit/2e5f8c2))
- Demo projects renamed by the owner: **"Pothole on Westheimer"**
  (Creator-only, succeeds) and **"Fix Local Park"** (Community, fails).
  The first Devnet run (18:57) still shows the earlier sample titles.

### 18:57 · First full demo run on Devnet ([c3798a6](https://github.com/HomeTownHer0/townfund/commit/c3798a6))
- Success story: 10 USDC pledged into escrow, 1 USDC tip, settle paid 9.5 to
  the creator and 0.5 to the treasury, soulbound badge minted.
- Failure story: 5 of 20 USDC pledged, settle marked Failed, 100% refund.

### 18:40 · Demo tool ([12b6852](https://github.com/HomeTownHer0/townfund/commit/12b6852))
- `townfund` CLI: one command funds demo wallets, creates the treasury, and
  runs the success and failure stories with Explorer links. Also `status`,
  `balances`, `recycle`.

### 18:31 · Deployed to Devnet
- Program `FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv`, demo build, 409,488
  bytes. Upgrade authority: owner's wallet `7cpjF…xU21`.

### 18:14 · Soulbound backer badge ([b86740d](https://github.com/HomeTownHer0/townfund/commit/b86740d))
- `mint_backer_badge`: Metaplex Core NFT for each backer of a successful
  project; frozen forever (Permanent Freeze, no authority). 8 tests against
  the real Metaplex Core program dumped from Devnet.
- **Design change:** soulbound NFTs had been moved to after Oct 16; the
  owner brought them back into V1, using Metaplex Core as taught in class.

### 17:46 · Demo clock ([11188ea](https://github.com/HomeTownHer0/townfund/commit/11188ea))
- Demo build accepts projects from 10 seconds to 10 minutes, so a full run
  fits in a 5-minute presentation. Normal build unchanged (1–60 days).

### 17:17 · Program ID ([15ae03b](https://github.com/HomeTownHer0/townfund/commit/15ae03b))
- Program ID set to `FdaMK3…PXrv`, from a keypair generated on the owner's Mac.

### 16:50 · Devnet preparation ([8920ce2](https://github.com/HomeTownHer0/townfund/commit/8920ce2))
- **Design change:** accept Circle's Devnet USDC instead of our own mock
  mint (owner's choice: use the faucet that already works).
- Demo build script; GitHub Actions checks both builds.

### 16:34 · Slice 3 ([fb2fb0a](https://github.com/HomeTownHer0/townfund/commit/fb2fb0a))
- `settle` (95/5 on success, Failed otherwise), `cancel_project`, `refund`
  (100%, receipt closed, rent back to the backer).

### 16:31 · Slice 2 ([363a5c5](https://github.com/HomeTownHer0/townfund/commit/363a5c5))
- Treasury (no spending in V1; empty "door authority" field reserved),
  `pledge` with Community voting on the first pledge, `tip`.

### 14:59 · Slice 1 ([852216f](https://github.com/HomeTownHer0/townfund/commit/852216f))
- `initialize_project`: Project account and vault, with all creation checks.
- Every stored account has a version byte and spare room; every money
  movement emits an event.

### 14:53 · Independent checks ([8b37196](https://github.com/HomeTownHer0/townfund/commit/8b37196))
- GitHub Actions builds and tests every push.

### 14:20 · Scaffold ([0916088](https://github.com/HomeTownHer0/townfund/commit/0916088))
- Anchor 1.2 workspace, keypair-safe `.gitignore`, build script.

### 13:39 · Repository created
