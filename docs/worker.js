// Web Worker: owns the mercator wasm plugin so its synchronous `.call()` runs
// off the main thread — a large GeoJSON never freezes the editor UI.
//
// mercator is a `wasm-minimal-protocol` plugin (same ABI as any Typst plugin):
// the host passes each argument's *length* as an i32 and supplies two import
// callbacks — one to copy the argument bytes into wasm memory, one to receive
// the result bytes. A non-zero return code means the result buffer holds an
// error string instead of the SVG.
//
// Message protocol (both directions carry a matching `id`):
//   in : { id, kind: "call", fn, args: [Uint8Array, ...] }
//   out: { id, ok: true, result } | { id, ok: false, error }
//
// The compiled WebAssembly.Module is cached in IndexedDB keyed by the wasm
// file's ETag/Last-Modified, so repeat visits skip both download and compile;
// a CI redeploy of a fresh wasm invalidates the cache automatically.

const WASM_URL = "mercator.wasm";

// Module-level scratch for the wasm-minimal-protocol host callbacks: they run
// synchronously inside the wasm call and read/write via `mem.buffer`.
let _argParts = [];
let _result = new Uint8Array();
let inst = null;
let mem = null;
let ensurePromise = null;

const imports = {
  typst_env: {
    // Copy each argument's bytes contiguously at `ptr` — no full-size temp buffer.
    wasm_minimal_protocol_write_args_to_buffer: (ptr) => {
      const dst = new Uint8Array(mem.buffer);
      let o = ptr;
      for (const a of _argParts) { dst.set(a, o); o += a.length; }
    },
    // Copy the result out before wasm reuses the memory.
    wasm_minimal_protocol_send_result_to_host: (ptr, len) => {
      _result = new Uint8Array(mem.buffer, ptr, len).slice();
    },
  },
};

async function ensure() {
  if (inst) return;
  ensurePromise ??= (async () => {
    const module = await loadModule(WASM_URL);
    const i = await WebAssembly.instantiate(module, imports);
    inst = i;
    mem = i.exports.memory;
  })();
  await ensurePromise;
}

function call(fn, args) {
  _argParts = args;
  _result = new Uint8Array();
  const rc = inst.exports[fn](...args.map((a) => a.length));
  if (rc !== 0) {
    throw new Error(new TextDecoder().decode(_result) || `${fn} call failed`);
  }
  return _result;
}

// ───────────────────────────── IndexedDB module cache ───────────────────────
const IDB_NAME = "mercator-cache";
const IDB_STORE = "modules";
let _dbPromise = null;

function idbOpen() {
  return (_dbPromise ??= new Promise((res, rej) => {
    const r = indexedDB.open(IDB_NAME, 1);
    r.onupgradeneeded = () => r.result.createObjectStore(IDB_STORE);
    r.onsuccess = () => res(r.result);
    r.onerror = () => { _dbPromise = null; rej(r.error); };
  }));
}
async function idbGet(key) {
  try {
    const db = await idbOpen();
    return await new Promise((res, rej) => {
      const q = db.transaction(IDB_STORE, "readonly").objectStore(IDB_STORE).get(key);
      q.onsuccess = () => res(q.result);
      q.onerror = () => rej(q.error);
    });
  } catch { return undefined; }
}
async function idbPut(key, val) {
  try {
    const db = await idbOpen();
    await new Promise((res, rej) => {
      const q = db.transaction(IDB_STORE, "readwrite").objectStore(IDB_STORE).put(val, key);
      q.onsuccess = () => res();
      q.onerror = () => rej(q.error);
    });
  } catch { /* private mode / quota / no structured-clone of Module: skip caching */ }
}

async function compileModule(url) {
  try { return await WebAssembly.compileStreaming(fetch(url)); }
  catch { return await WebAssembly.compile(await (await fetch(url)).arrayBuffer()); }
}
async function loadModule(url) {
  let tag = null;
  try {
    const h = await fetch(url, { method: "HEAD" });
    tag = h.headers.get("etag") || h.headers.get("last-modified");
  } catch { /* no freshness signal → compile fresh, don't cache */ }

  if (tag) {
    const hit = await idbGet(url);
    if (hit && hit.tag === tag && hit.module instanceof WebAssembly.Module) return hit.module;
  }
  const module = await compileModule(url);
  if (tag) idbPut(url, { tag, module });
  return module;
}

// ───────────────────────────── message dispatch ─────────────────────────────
self.onmessage = async (e) => {
  const { id, kind, fn, args } = e.data;
  try {
    if (kind === "ensure") {
      await ensure();
      return self.postMessage({ id, ok: true });
    }
    if (kind === "call") {
      await ensure();
      const result = call(fn, args);
      // Transfer the buffer — the main thread only reads it.
      return self.postMessage({ id, ok: true, result }, [result.buffer]);
    }
    self.postMessage({ id, ok: false, error: `unknown kind: ${kind}` });
  } catch (err) {
    self.postMessage({ id, ok: false, error: (err && err.message) || String(err) });
  }
};
