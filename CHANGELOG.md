# Changelog

Every revision to TownFund, newest first. Times are US Central. Each entry
links to its commit, which shows exactly what changed.

The program deployed on Devnet (`FdaMK3…PXrv`) was built from commit
**b86740d**. Later entries changed only the demo tool, tests and docs, not
the program (verified: same source, same 409,488-byte build).

## 2026-10-12

### 01:20 · One link: the button is on the main page
- The Run demo button is on the plain link for everyone; the `?presenter`
  link is gone (owner's call).
- The page reaches the runner only after the button is clicked, then
  remembers that computer and shows the runner's state and Terminal pane on
  load. Visitors who click get a note that the demo runs from the
  presenter's computer; opening the page never makes their browser contact a
  local runner.
- Verified on a local chain: visitor (0 runner calls before clicking, note
  on click), presenter (click starts the demo, Terminal pane appears, still
  there after reload).

## 2026-10-11

### 16:20 · Map keeps pace with the Terminal
- Measured the delay: the demo sent "open, pledge, tip" within about 1 s,
  so coins piled up and the map ran ~2 s behind the Terminal and the
  program log up to ~4.5 s (more on public Devnet, which the page checks
  every 2.5 s).
- Demo now pauses 3 s after each step (`--pace`, also for `serve`), so each
  move gets its own moment. A full run takes about a minute.
- On the presenter's computer, a confirmed transaction in the Terminal pane
  makes the page check the chain at once instead of waiting for its next
  check. The Terminal pane types faster and the runner is asked more often
  while running.
- Verified on a local chain: each Terminal step, its coin and its program
  log entry appear within 0.1 s of each other. **Restart `serve` after
  pulling.**

### 14:40 · One link for everything
- The plain link (`hometownher0.github.io/TownFund/`) now shows the Run demo
  button and Terminal pane on the presenter's computer. Open `?presenter`
  once in that browser and it is remembered (`?presenter=off` forgets it).
  Visitors still get the read-only page, and their browsers never try to
  reach a local runner (which would make Chrome ask them for permission).
- `serve` now points to the plain link.

### 14:20 · Consoles stay side by side
- The program log and Terminal panes now sit side by side, each half the
  width, on any window wider than 640 px (before: they stacked full-width
  below 900 px). Phones still stack them.

### 14:00 · Live consoles under the map
- The "Latest move" bar is replaced by a **TownFund program log** pane:
  every transaction, as it lands, prints a summary line (who paid whom, tx
  link) and then the program's real log lines from Devnet (program invoked,
  instruction, token transfer, event, compute units, success), with slot
  and fee. Everyone sees it.
- With `?presenter`, a **Terminal** pane beside it streams the runner's
  output live (same lines as the Terminal window; the deadline countdown
  updates in place; Explorer URLs become short links).
- Runner: `serve` now runs the demo as a child process to capture its
  output; `/status?since=N` returns new Terminal lines. **Restart `serve`
  after pulling.**
- Verified on a local chain: full run, both panes; visitor view shows only
  the program log; no sideways scrolling at phone width.

### 13:05 · README and GitHub brought up to date
- README: status (button + auto-reset), what TIP and the ★ mean on the map,
  boxes change as coins land, presenter row in the build plan, runner in the
  code map and Safety. Build badge points at the renamed repo
  (`HomeTownHer0/TownFund`).
- Live page's "Code on GitHub" link and the git remote use the new name.

### 12:45 · Boxes change when the coin lands
- Fixed: Creator 2 showed Backer 2's tip before the coin reached it (and
  the same for other boxes). The page now waits for each coin: a box that
  receives money changes the moment its coin arrives (TIP after the first
  leg, the creator after the second); the sender's box drops as the coin
  leaves. Verified on a local chain: every box changed within 0.1 s of its
  coin's arrival.

### 11:00 · "Run demo" button, auto-reset, slower coins
- `demo` now resets itself 15 s after it finishes (`--reset-after`, `0` to
  keep the final state): creators emptied back to the main wallet, backers
  topped up to pledge + tip (11 and 6 USDC). New `reset` command does the same
  on demand. The treasury keeps its fees (no withdraw in V1).
- New `serve` command: a small runner on `127.0.0.1:8787` for the live
  page. Opening the page with `?presenter` shows a **Run demo** button
  (Running… → Resetting… → Run demo). Only this computer can reach the
  runner, only the TownFund page (or a local page) may call it, and keys stay
  on the computer.
- After a reset every box returns to its starting value within a second or
  two, TIP included.
- Coins move slower (2.6 s per road) so each transfer can be followed.
- Settle retries if the connection to Devnet drops (safe: the program
  refuses a second settle).
- Verified on a local chain: two back-to-back button runs, each followed by
  a full reset.

### 10:10 · TIP starts at 0 when the page opens
- TIP now counts only tips made after the page was opened (and in the
  current run), so it reads 0 immediately on load instead of showing the
  previous run's total. Verified: open after a finished run → 0 → 1 → 2.

### 09:45 · Cleaner boxes; TIP resets each run
- Removed the green/red "change since this run" tags (+10.5, −11, …) from
  every box, and the legend line about them.
- Fixed: TIP carried the previous run's tips until "Fix Local Park"
  opened. A run now starts when its "Pothole on Westheimer" project opens,
  so TIP drops to 0 right away (verified: 2 → 0 → 1 → 2).
- The map's screen-reader description no longer pops up as a tooltip
  when hovering over Backer 1.

### 09:10 · "Latest move" line on the map
- Every transaction is spelled out under the map as it lands, from the real
  decoded transaction, e.g. "Tip: Backer 2 −1 → Creator 2 +1" or
  "Refund: Vault: Fix Local Park −5 → Backer 2 +5", so no move is missed
  even if its coin animation is.

### 09:05 · Second creator for "Fix Local Park"
- New demo wallet **creator2** opens "Fix Local Park"; Backer 2's tip goes to
  Creator 2. Creator 1 keeps "Pothole on Westheimer".
- Live page: Creator 1 and Creator 2 boxes; TIP has a road to each, so a tip
  travels backer -> TIP -> that project's creator. Both creators are
  discovered from the on-chain projects (no addresses to configure).
- TIP counter never counts down within a run.

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
