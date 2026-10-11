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

// Account discriminators (first 8 bytes), from the program's IDL.
const DISC = { project: "bQ9n6Kmh9zA", pledge: "U4PLFQvCKer", badge: "7mtScFmiAfh" };

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

const memcmp = (offset, bytes) => ({ memcmp: { offset, bytes } });
const usdcOf = (r) => {
  const accts = r?.value || [];
  return accts.reduce((sum, a) => sum + Number(a.account.data.parsed.info.tokenAmount.amount), 0);
};

// ---------------------------------------------------------------- state
/** One snapshot of everything the page shows. */
export async function fetchState(cfg = CONFIG) {
  const w = cfg.wallets;
  const tokenByOwner = (owner) => ["getTokenAccountsByOwner", [owner, { mint: cfg.usdcMint }, { encoding: "jsonParsed", commitment: "confirmed" }]];

  const [c, b1, b2, treasury, projectAccts, slot] = await rpcBatch(cfg.rpc, [
    tokenByOwner(w.creator),
    tokenByOwner(w.backer1),
    tokenByOwner(w.backer2),
    ["getTokenAccountBalance", [cfg.treasuryVault, { commitment: "confirmed" }]],
    ["getProgramAccounts", [cfg.program, { encoding: "base64", commitment: "confirmed", filters: [memcmp(0, DISC.project), memcmp(13, w.creator)] }]],
    ["getSlot", [{ commitment: "confirmed" }]],
  ], true);
  if (slot == null) throw new Error("Devnet did not answer");

  const projects = (projectAccts || [])
    .map((a) => decodeProject(a.pubkey, fromBase64(a.account.data[0])))
    .sort((a, b) => b.createdAt - a.createdAt || b.projectId - a.projectId);
  const success = projects.find((p) => p.kind === "Creator") || null;
  const community = projects.find((p) => p.kind === "Community") || null;
  const shown = [success, community].filter(Boolean);

  const calls = [["getBlockTime", [slot]]];
  for (const p of shown) {
    calls.push(["getTokenAccountBalance", [p.vault, { commitment: "confirmed" }]]);
    calls.push(["getProgramAccounts", [cfg.program, { encoding: "base64", commitment: "confirmed", filters: [memcmp(0, DISC.pledge), memcmp(10, p.address)] }]]);
    calls.push(["getProgramAccounts", [cfg.program, { encoding: "base64", commitment: "confirmed", filters: [memcmp(0, DISC.badge), memcmp(10, p.address)] }]]);
  }
  const res = await rpcBatch(cfg.rpc, calls, true);
  const chainTime = res[0];
  shown.forEach((p, i) => {
    p.vaultBalance = Number(res[1 + i * 3]?.value?.amount ?? 0);
    p.pledges = (res[2 + i * 3] || []).map((a) => decodePledge(a.pubkey, fromBase64(a.account.data[0])));
    p.badges = (res[3 + i * 3] || []).map((a) => decodeBadge(a.pubkey, fromBase64(a.account.data[0])));
  });

  return {
    fetchedAt: Date.now(),
    chainTime,
    balances: {
      creator: usdcOf(c),
      backer1: usdcOf(b1),
      backer2: usdcOf(b2),
      treasury: Number(treasury?.value?.amount ?? 0),
    },
    success,
    community,
    projectCount: projects.length,
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
  const missing = sigs.filter((s) => !cache.has(s.signature));
  if (missing.length) {
    const txs = await rpcBatch(cfg.rpc, missing.map((s) => ["getTransaction", [s.signature, { encoding: "json", maxSupportedTransactionVersion: 0, commitment: "confirmed" }]]), true);
    missing.forEach((s, i) => { if (txs[i]) cache.set(s.signature, describeTx(s, txs[i], labels, cfg)); });
  }
  return sigs.map((s) => cache.get(s.signature)).filter(Boolean);
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
