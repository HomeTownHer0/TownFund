// TownFund live status: data layer.
// Reads everything straight from Solana (no server). Works in the browser
// and in Node (for testing). Every call is a read; nothing here can sign.

export const CONFIG = {
  rpc: "https://api.devnet.solana.com",
  program: "FdaMK3RcAw7oGguD1xZnSrkZa7XKzGU6yGMskPptPXrv",
  usdcMint: "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU",
  treasury: "CSnJkezhDZxZwgzNA3hPh6Skxh5gv2hpETMUSeBdrQnn",
  treasuryVault: "JSs6ZYkdMG34n9UP18B3vxnH7jjAri8fBLDyCRLnyss",
  wallets: {
    creator: "6RU9N2BvzkkJQXQ2QgrGtGwkdfUN3beqvTAJxxL8Zcip",
    backer1: "GxZZ9JJtBPTmr9hioVwaHU43n3LxpUtGXMXK3vkDgAhS",
    backer2: "Ex6bzFJYnw2gz9zgvJg2Am4adXocxU2JkeKKUvTBEFUm",
  },
};


export const KIND = ["Creator", "Community"];
export const STATUS = ["Active", "Cancelled", "Succeeded", "Failed"];

// ------------------------------------------------------------- encoding
const B58 = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

export function b58encode(bytes) {
  let n = 0n;
  for (const b of bytes) n = (n << 8n) + BigInt(b);
  let s = "";
  while (n > 0n) { s = B58[Number(n % 58n)] + s; n /= 58n; }
  for (const b of bytes) { if (b === 0) s = "1" + s; else break; }
  return s;
}

function fromBase64(str) {
  if (typeof atob === "function") {
    const bin = atob(str);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) out[i] = bin.charCodeAt(i);
    return out;
  }
  return new Uint8Array(Buffer.from(str, "base64"));
}

class Reader {
  constructor(bytes, offset = 0) { this.b = bytes; this.o = offset; this.v = new DataView(bytes.buffer, bytes.byteOffset); }
  u8() { return this.b[this.o++]; }
  u32() { const x = this.v.getUint32(this.o, true); this.o += 4; return x; }
  u64() { const x = this.v.getBigUint64(this.o, true); this.o += 8; return Number(x); }
  i64() { const x = this.v.getBigInt64(this.o, true); this.o += 8; return Number(x); }
  key() { const k = b58encode(this.b.subarray(this.o, this.o + 32)); this.o += 32; return k; }
  str() { const n = this.u32(); const s = new TextDecoder().decode(this.b.subarray(this.o, this.o + n)); this.o += n; return s; }
}

export function decodeProject(address, data) {
  const r = new Reader(data, 8);
  const p = { address };
  p.version = r.u8(); r.u8(); r.u8();
  p.kind = KIND[r.u8()];
  p.status = STATUS[r.u8()];
  p.creator = r.key();
  p.projectId = r.u64();
  p.mint = r.key();
  p.vault = r.key();
  p.target = r.u64();
  p.totalPledged = r.u64();
  p.backerCount = r.u32();
  p.createdAt = r.i64();
  p.deadline = r.i64();
  p.title = r.str();
  const n = r.u32();
  p.candidates = [];
  for (let i = 0; i < n; i++) p.candidates.push(r.key());
  p.votes = [r.u32(), r.u32(), r.u32(), r.u32(), r.u32()];
  return p;
}

export function decodePledge(address, data) {
  const r = new Reader(data, 8);
  r.u8(); r.u8();
  const project = r.key(), backer = r.key(), amount = r.u64();
  const vote = r.u8() === 1 ? r.u8() : null;
  return { address, project, backer, amount, vote, refunded: r.u8() === 1 };
}

export function decodeBadge(address, data) {
  const r = new Reader(data, 8);
  r.u8(); r.u8();
  return { address, project: r.key(), backer: r.key(), asset: r.key() };
}

// ------------------------------------------------------------------ rpc
let rpcId = 0;
/**
 * Several RPC calls in one HTTP request. With `lenient`, a call that fails
 * (for example "could not find account" before an account exists) returns
 * null instead of failing the whole batch.
 */
export async function rpcBatch(url, calls, lenient = false) {
  const body = calls.map(([method, params]) => ({ jsonrpc: "2.0", id: ++rpcId, method, params }));
  const res = await fetch(url, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify(body) });
  if (res.status === 429) throw new RateLimited("Devnet rate limit");
  if (!res.ok) throw new Error(`RPC HTTP ${res.status}`);
  const out = await res.json();
  const byId = new Map((Array.isArray(out) ? out : [out]).map((r) => [r.id, r]));
  return body.map((b) => {
    const r = byId.get(b.id);
    if (!r || r.error) {
      if (lenient) return null;
      throw new Error(r?.error?.message || "RPC error");
    }
    return r.result;
  });
}


// ---------------------------------------------------------------- state
/** One snapshot of everything the page shows. */
/** Thrown when the RPC says "too many requests" (HTTP 429). */
export class RateLimited extends Error {}

/** Cheap check: newest TownFund transaction signature (1 request). */
export async function fetchLatestSignature(cfg = CONFIG) {
  const [sigs] = await rpcBatch(cfg.rpc, [["getSignaturesForAddress", [cfg.program, { limit: 1, commitment: "confirmed" }]]]);
  return sigs?.[0]?.signature || null;
}

const sameBytes = (data, disc) => disc.every((b, i) => data[i] === b);
const DISC_BYTES = {
  project: [205, 168, 189, 202, 181, 247, 142, 19],
  pledge: [161, 197, 121, 46, 99, 75, 169, 131],
  badge: [40, 127, 162, 181, 177, 154, 1, 48],
};

/**
 * Each wallet's USDC token account, looked up once and remembered.
 * Devnet's public server refuses several getTokenAccountsByOwner calls at
 * the same time, so these lookups run one after another.
 */
const tokenAccountOf = new Map();
async function resolveTokenAccounts(cfg) {
  for (const owner of Object.values(cfg.wallets)) {
    if (!owner || owner.startsWith("__") || tokenAccountOf.get(owner)) continue;
    const [r] = await rpcBatch(cfg.rpc, [["getTokenAccountsByOwner", [owner, { mint: cfg.usdcMint }, { encoding: "base64", commitment: "confirmed" }]]]);
    const found = r?.value?.[0]?.pubkey;
    if (found) tokenAccountOf.set(owner, found);
  }
}

/** USDC amount held in a jsonParsed token account (0 if it doesn't exist). */
const amountOf = (acct) => Number(acct?.data?.parsed?.info?.tokenAmount?.amount ?? 0);

/**
 * One full snapshot of everything the page shows. Every call in a request
 * is a different method, which keeps the public server happy:
 *  1. ALL TownFund accounts (one getProgramAccounts) and the slot
 *  2. every balance at once (getMultipleAccounts) and the chain clock
 */
export async function fetchState(cfg = CONFIG) {
  const w = cfg.wallets;
  await resolveTokenAccounts(cfg);

  const [all, slot] = await rpcBatch(cfg.rpc, [
    ["getProgramAccounts", [cfg.program, { encoding: "base64", commitment: "confirmed" }]],
    ["getSlot", [{ commitment: "confirmed" }]],
  ], true);
  if (slot == null || all == null) throw new Error("Devnet did not answer");

  const projects = [], pledges = [], badges = [];
  for (const a of all) {
    const data = fromBase64(a.account.data[0]);
    if (sameBytes(data, DISC_BYTES.project)) projects.push(decodeProject(a.pubkey, data));
    else if (sameBytes(data, DISC_BYTES.pledge)) pledges.push(decodePledge(a.pubkey, data));
    else if (sameBytes(data, DISC_BYTES.badge)) badges.push(decodeBadge(a.pubkey, data));
  }
  const mine = projects
    .filter((p) => p.creator === w.creator)
    .sort((a, b) => b.createdAt - a.createdAt || b.projectId - a.projectId);
  const success = mine.find((p) => p.kind === "Creator") || null;
  const community = mine.find((p) => p.kind === "Community") || null;
  const shown = [success, community].filter(Boolean);
  for (const p of shown) {
    p.pledges = pledges.filter((x) => x.project === p.address);
    p.badges = badges.filter((x) => x.project === p.address);
  }

  // Every balance in one call. Unknown token accounts use the program ID
  // as a harmless placeholder (it is never a token account, so it reads 0).
  const keys = [
    tokenAccountOf.get(w.creator) || cfg.program,
    tokenAccountOf.get(w.backer1) || cfg.program,
    tokenAccountOf.get(w.backer2) || cfg.program,
    cfg.treasuryVault,
    ...shown.map((p) => p.vault),
  ];
  const [multi, chainTime] = await rpcBatch(cfg.rpc, [
    ["getMultipleAccounts", [keys, { encoding: "jsonParsed", commitment: "confirmed" }]],
    ["getBlockTime", [slot]],
  ], true);
  if (multi == null) throw new Error("Devnet did not answer");
  const v = multi.value;
  shown.forEach((p, i) => { p.vaultBalance = amountOf(v[4 + i]); });

  // Soulbound badges each wallet holds, across every project (newest first).
  const createdAt = new Map(projects.map((p) => [p.address, p.createdAt]));
  const badgesOf = {};
  for (const [name, addr] of Object.entries(w)) {
    badgesOf[name] = badges
      .filter((b) => b.backer === addr)
      .sort((a, b) => (createdAt.get(b.project) || 0) - (createdAt.get(a.project) || 0));
  }

  return {
    fetchedAt: Date.now(),
    chainTime,
    balances: {
      creator: amountOf(v[0]),
      backer1: amountOf(v[1]),
      backer2: amountOf(v[2]),
      treasury: amountOf(v[3]),
    },
    badgesOf,
    success,
    community,
    projectCount: mine.length,
  };
}

// ------------------------------------------------------------ activity
const IX_NAMES = {
  InitializeProject: "Project opened",
  InitializeTreasury: "Treasury created",
  Pledge: "Pledge",
  Tip: "Tip",
  Settle: "Settle",
  CancelProject: "Project cancelled",
  Refund: "Refund",
  MintBackerBadge: "Backer badge minted",
};

/**
 * Recent TownFund transactions, newest first. `cache` (a Map) keeps already
 * decoded transactions so each one is fetched only once.
 */
export async function fetchActivity(cache, labels, cfg = CONFIG, limit = 12) {
  const [sigs] = await rpcBatch(cfg.rpc, [["getSignaturesForAddress", [cfg.program, { limit, commitment: "confirmed" }]]]);
  // Look up at most 4 new transactions per call, so the public RPC isn't
  // flooded; the rest are picked up on the next refresh.
  const missing = sigs.filter((s) => !cache.has(s.signature)).slice(0, 4);
  // One at a time: the public server refuses several of the same call at once.
  for (const s of missing) {
    const [tx] = await rpcBatch(cfg.rpc, [["getTransaction", [s.signature, { encoding: "json", maxSupportedTransactionVersion: 0, commitment: "confirmed" }]]], true);
    if (tx) cache.set(s.signature, describeTx(s, tx, labels, cfg));
  }
  const items = sigs.map((s) => cache.get(s.signature)).filter(Boolean);
  items.complete = sigs.every((s) => cache.has(s.signature));
  return items;
}

function describeTx(sig, tx, labels, cfg) {
  const logs = tx?.meta?.logMessages || [];
  const names = logs
    .map((l) => /^Program log: Instruction: (\w+)$/.exec(l)?.[1])
    .filter((n) => n && IX_NAMES[n]);
  const name = names.length ? IX_NAMES[names[0]] : "Transaction";
  // USDC movement by owner, from the token balance changes in the tx.
  const delta = new Map();
  for (const [list, sign] of [[tx?.meta?.preTokenBalances, -1], [tx?.meta?.postTokenBalances, 1]]) {
    for (const b of list || []) {
      if (b.mint !== cfg.usdcMint) continue;
      delta.set(b.owner, (delta.get(b.owner) || 0) + sign * Number(b.uiTokenAmount.amount));
    }
  }
  const moves = [...delta.entries()]
    .filter(([, d]) => d !== 0)
    .map(([owner, d]) => ({ who: labels[owner] || short(owner), amount: d }))
    .sort((a, b) => a.amount - b.amount);
  return { signature: sig.signature, time: sig.blockTime, ok: !sig.err, name, moves };
}

export const short = (a) => (a ? `${a.slice(0, 4)}…${a.slice(-4)}` : "");

export function usdc(base) {
  const v = base / 1e6;
  return v.toLocaleString("en-US", { minimumFractionDigits: 0, maximumFractionDigits: 2 });
}
