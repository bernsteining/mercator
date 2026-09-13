// Demo front-end for the mercator Typst plugin. The config is driven by a
// schema-generated control panel (sliders / colors / toggles / selects), the
// data comes from a preset, a file picker, or drag-and-drop, and a read-only
// "preview.typ" panel mirrors the real `#render-map(..)` call. The wasm runs in
// worker.js (off the main thread). Dragging an azimuthal map spins the globe.

const ENC = new TextEncoder();
const DEC = new TextDecoder();
const VERSION = "0.1.2";

// ───────────────────────────── worker RPC ───────────────────────────────────
const worker = new Worker("worker.js");
let _rpcId = 0;
const _pending = new Map();
worker.onmessage = (e) => {
  const { id, ok, result, error } = e.data;
  const p = _pending.get(id);
  if (!p) return;
  _pending.delete(id);
  ok ? p.resolve(result) : p.reject(new Error(error));
};
function wReq(msg) {
  const id = ++_rpcId;
  return new Promise((resolve, reject) => {
    _pending.set(id, { resolve, reject });
    worker.postMessage({ id, ...msg }, msg.args ? msg.args.map((a) => a.buffer) : []);
  });
}
async function renderMap(bytes, cfgStr) {
  const out = await wReq({ kind: "call", fn: "geo", args: [bytes, ENC.encode(cfgStr)] });
  return DEC.decode(out);
}

// ───────────────────────────── Typst highlighting ───────────────────────────
const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
const TS_RULES = [
  ["ws", /^\s+/], ["comment", /^\/\/[^\n]*/],
  ["string", /^"(?:[^"\\]|\\.)*"/], ["number", /^-?\d+(?:\.\d+)?/],
  ["directive", /^#[A-Za-z][\w-]*/], ["kw", /^(?:none|true|false|auto)\b/],
  ["ident", /^[A-Za-z_][\w-]*/], ["punct", /^[(){}\[\],:*+=]/],
];
function highlightTypstLine(line) {
  let s = line, out = "";
  outer: while (s) {
    for (let [type, re] of TS_RULES) {
      const m = re.exec(s);
      if (!m) continue;
      const txt = m[0];
      if (type === "ident" && /^\s*:/.test(s.slice(txt.length))) type = "key";
      out += type === "ws" ? esc(txt) : `<span class="t-${type}">${esc(txt)}</span>`;
      s = s.slice(txt.length);
      continue outer;
    }
    out += esc(s[0]); s = s.slice(1);
  }
  return out || "&nbsp;";
}
const identKey = (k) => (/^[A-Za-z_][\w-]*$/.test(k) ? k : JSON.stringify(k));
function jsonToTypst(v, indent) {
  const pad = "  ".repeat(indent), pad1 = "  ".repeat(indent + 1);
  if (v === null) return "none";
  if (typeof v === "boolean" || typeof v === "number") return String(v);
  if (typeof v === "string") return JSON.stringify(v);
  if (Array.isArray(v)) {
    if (!v.length) return "()";
    const items = v.map((x) => jsonToTypst(x, indent + 1));
    const one = "(" + items.join(", ") + (items.length === 1 ? "," : "") + ")";
    if (one.length <= 58 && !one.includes("\n")) return one;
    return "(\n" + items.map((it) => pad1 + it).join(",\n") + ",\n" + pad + ")";
  }
  const keys = Object.keys(v);
  if (!keys.length) return "(:)";
  const entries = keys.map((k) => `${identKey(k)}: ${jsonToTypst(v[k], indent + 1)}`);
  const one = "(" + entries.join(", ") + ")";
  if (one.length <= 58 && !one.includes("\n")) return one;
  return "(\n" + entries.map((e) => pad1 + e).join(",\n") + ",\n" + pad + ")";
}

// ───────────────────────────── projections ──────────────────────────────────
const PROJECTIONS = [
  "equirectangular", "mercator", "cassini", "robinson", "natural_earth",
  "winkel_tripel", "hammer", "bonne", "polyconic", "lambert_conformal_conic",
  "albers_equal_area", "orthographic", "azimuthal_equidistant",
  "lambert_azimuthal_equal_area", "gnomonic", "wiechel", "peirce_quincuncial",
  "authagraph",
];
const AZIMUTHAL = new Set(["orthographic", "azimuthal_equidistant", "lambert_azimuthal_equal_area", "gnomonic", "wiechel"]);
const HAS_CM = new Set(["equirectangular", "mercator", "robinson", "natural_earth", "cassini", "polyconic", "hammer", "winkel_tripel", "bonne", "lambert_conformal_conic", "albers_equal_area"]);
const CONIC = new Set(["lambert_conformal_conic", "albers_equal_area"]);
const ANTI_OK = new Set(["equirectangular", "mercator", "cassini"]);
const hasCenterLon = (t) => AZIMUTHAL.has(t) || t === "peirce_quincuncial";

// ───────────────────────────── config schema ────────────────────────────────
// Each field: state key + control type + default (matching the plugin's own
// default, so buildConfig can omit unchanged values). `when` hides a row.
const SCHEMA = [
  { title: "Projection", fields: [
    { k: "proj_type", label: "Type", t: "sel", def: "equirectangular", opts: PROJECTIONS.map((p) => [p, p.replace(/_/g, " ")]) },
    { k: "central_meridian", label: "Central meridian", t: "rng", def: 0, min: -180, max: 180, step: 1, when: (s) => HAS_CM.has(s.proj_type) },
    { k: "center_lon", label: "Center lon", t: "rng", def: 0, min: -180, max: 180, step: 1, when: (s) => hasCenterLon(s.proj_type) },
    { k: "center_lat", label: "Center lat", t: "rng", def: 0, min: -90, max: 90, step: 1, when: (s) => AZIMUTHAL.has(s.proj_type) },
    { k: "standard_parallel_1", label: "Std parallel 1", t: "rng", def: 20, min: -80, max: 80, step: 1, when: (s) => CONIC.has(s.proj_type) },
    { k: "standard_parallel_2", label: "Std parallel 2", t: "rng", def: 50, min: -80, max: 80, step: 1, when: (s) => CONIC.has(s.proj_type) },
    { k: "latitude_of_origin", label: "Lat of origin", t: "rng", def: 0, min: -80, max: 80, step: 1, when: (s) => CONIC.has(s.proj_type) },
    { k: "standard_parallel", label: "Std parallel", t: "rng", def: 50, min: -80, max: 80, step: 1, when: (s) => s.proj_type === "bonne" },
  ] },
  { title: "Style", fields: [
    { k: "fill_mode", label: "Fill", t: "sel", def: "color", opts: [["color", "solid color"], ["none", "none"]] },
    { k: "fill", label: "Fill color", t: "color", def: "#ffffff", when: (s) => s.fill_mode === "color" },
    { k: "fill_opacity", label: "Fill opacity", t: "rng", def: 1, min: 0, max: 1, step: 0.05, when: (s) => s.fill_mode === "color" },
    { k: "stroke_mode", label: "Stroke", t: "sel", def: "color", opts: [["color", "solid color"], ["none", "none"]] },
    { k: "stroke", label: "Stroke color", t: "color", def: "#000000", when: (s) => s.stroke_mode === "color" },
    { k: "stroke_width", label: "Stroke width", t: "rng", def: 0.05, min: 0, max: 0.3, step: 0.002, when: (s) => s.stroke_mode === "color" },
    { k: "point_mode", label: "Points", t: "sel", def: "auto", opts: [["auto", "auto (= fill)"], ["none", "hidden"], ["custom", "custom color"]] },
    { k: "point_color", label: "Point color", t: "color", def: "#cc0033", when: (s) => s.point_mode === "custom" },
    { k: "point_radius", label: "Point radius (0 = auto)", t: "rng", def: 0, min: 0, max: 2, step: 0.05 },
  ] },
  { title: "Graticule", fields: [
    { k: "grat_on", label: "Enabled", t: "bool", def: false },
    { k: "grat_step", label: "Step °", t: "rng", def: 15, min: 5, max: 90, step: 5, when: (s) => s.grat_on },
    { k: "grat_color", label: "Color", t: "color", def: "#cccccc", when: (s) => s.grat_on },
    { k: "grat_opacity", label: "Opacity", t: "rng", def: 0.6, min: 0, max: 1, step: 0.05, when: (s) => s.grat_on },
    { k: "grat_width", label: "Width", t: "rng", def: 0.5, min: 0, max: 2, step: 0.1, when: (s) => s.grat_on },
  ] },
  { title: "Sphere / ocean", when: (s) => AZIMUTHAL.has(s.proj_type), fields: [
    { k: "sphere_on", label: "Enabled", t: "bool", def: false },
    { k: "sphere_fill", label: "Ocean fill", t: "color", def: "#cfe8ff", when: (s) => s.sphere_on },
    { k: "sphere_stroke", label: "Outline", t: "color", def: "#3388cc", when: (s) => s.sphere_on },
    { k: "sphere_stroke_width", label: "Outline width", t: "rng", def: 0.005, min: 0, max: 0.05, step: 0.001, when: (s) => s.sphere_on },
  ] },
  { title: "Antimeridian clip", when: (s) => ANTI_OK.has(s.proj_type), fields: [
    { k: "anti_on", label: "Enabled", t: "bool", def: false },
  ] },
  { title: "Tissot's indicatrix", fields: [
    { k: "tissot_on", label: "Enabled", t: "bool", def: false },
    { k: "tissot_step", label: "Step °", t: "rng", def: 30, min: 10, max: 60, step: 5, when: (s) => s.tissot_on },
    { k: "tissot_radius", label: "Radius °", t: "rng", def: 5, min: 1, max: 15, step: 1, when: (s) => s.tissot_on },
    { k: "tissot_fill", label: "Fill", t: "color", def: "#ff0000", when: (s) => s.tissot_on },
    { k: "tissot_fill_opacity", label: "Fill opacity", t: "rng", def: 0.3, min: 0, max: 1, step: 0.05, when: (s) => s.tissot_on },
  ] },
];
const FIELDS = Object.fromEntries(SCHEMA.flatMap((s) => s.fields).map((f) => [f.k, f]));

// state holds every field's current value; `_extras` keeps preset config the
// form doesn't model (fill_scale, legend, point_radius_scale, label, …).
const state = {};
for (const f of Object.values(FIELDS)) state[f.k] = f.def;
let _extras = {};

// ───────────────────────────── config assembly ──────────────────────────────
const changed = (k) => state[k] !== FIELDS[k].def;
function buildConfig() {
  const cfg = {};
  const t = state.proj_type;
  const proj = { type: t };
  if (HAS_CM.has(t) && state.central_meridian !== 0) proj.central_meridian = state.central_meridian;
  if (AZIMUTHAL.has(t)) { proj.center_lat = state.center_lat; proj.center_lon = state.center_lon; }
  else if (t === "peirce_quincuncial" && state.center_lon !== 0) proj.center_lon = state.center_lon;
  if (CONIC.has(t)) { proj.standard_parallel_1 = state.standard_parallel_1; proj.standard_parallel_2 = state.standard_parallel_2; if (state.latitude_of_origin) proj.latitude_of_origin = state.latitude_of_origin; }
  if (t === "bonne") proj.standard_parallel = state.standard_parallel;
  cfg.projection = proj;

  if (state.fill_mode === "none") cfg.fill = "none";
  else if (changed("fill")) cfg.fill = state.fill;
  if (state.fill_mode !== "none" && changed("fill_opacity")) cfg.fill_opacity = state.fill_opacity;
  if (state.stroke_mode === "none") cfg.stroke = "none";
  else {
    if (changed("stroke")) cfg.stroke = state.stroke;
    if (changed("stroke_width")) cfg.stroke_width = state.stroke_width;
  }
  if (state.point_mode === "none") cfg.point_color = "none";
  else if (state.point_mode === "custom") cfg.point_color = state.point_color;
  if (state.point_radius > 0) cfg.point_radius = state.point_radius;

  if (state.grat_on) cfg.graticule = { step: state.grat_step, color: state.grat_color, opacity: state.grat_opacity, width: state.grat_width };
  if (AZIMUTHAL.has(t) && state.sphere_on) cfg.sphere = { fill: state.sphere_fill, stroke: state.sphere_stroke, stroke_width: state.sphere_stroke_width };
  if (ANTI_OK.has(t) && state.anti_on) cfg.antimeridian = true;
  if (state.tissot_on) cfg.tissot = { step: state.tissot_step, radius: state.tissot_radius, fill: state.tissot_fill, fill_opacity: state.tissot_fill_opacity };

  return { ...cfg, ..._extras };
}

// Convert any CSS color (named too) to #rrggbb so <input type=color> can show it.
function toHex(c) {
  if (!c || typeof c !== "string" || c === "none") return "#000000";
  if (/^#[0-9a-f]{6}$/i.test(c)) return c.toLowerCase();
  const el = document.createElement("div");
  el.style.color = c; el.style.display = "none"; document.body.appendChild(el);
  const m = /rgba?\((\d+),\s*(\d+),\s*(\d+)/.exec(getComputedStyle(el).color);
  el.remove();
  return m ? "#" + [1, 2, 3].map((i) => (+m[i]).toString(16).padStart(2, "0")).join("") : "#000000";
}

// Load a preset config object into `state` (+ `_extras` for what we don't model).
const KNOWN = new Set(["projection", "fill", "fill_opacity", "stroke", "stroke_width", "point_color", "point_radius", "graticule", "sphere", "antimeridian", "tissot"]);
function loadStateFromConfig(cfg) {
  for (const f of Object.values(FIELDS)) state[f.k] = f.def; // reset to defaults
  _extras = {};
  const p = cfg.projection || {};
  state.proj_type = p.type || "equirectangular";
  if (p.central_meridian != null) state.central_meridian = p.central_meridian;
  if (p.center_lon != null) state.center_lon = p.center_lon;
  if (p.center_lat != null) state.center_lat = p.center_lat;
  if (p.standard_parallel_1 != null) state.standard_parallel_1 = p.standard_parallel_1;
  if (p.standard_parallel_2 != null) state.standard_parallel_2 = p.standard_parallel_2;
  if (p.latitude_of_origin != null) state.latitude_of_origin = p.latitude_of_origin;
  if (p.standard_parallel != null) state.standard_parallel = p.standard_parallel;
  if (cfg.fill === "none") state.fill_mode = "none";
  else if (cfg.fill != null) { state.fill_mode = "color"; state.fill = toHex(cfg.fill); }
  if (cfg.fill_opacity != null) state.fill_opacity = cfg.fill_opacity;
  if (cfg.stroke === "none") state.stroke_mode = "none";
  else if (cfg.stroke != null) { state.stroke_mode = "color"; state.stroke = toHex(cfg.stroke); }
  if (cfg.stroke_width != null) state.stroke_width = cfg.stroke_width;
  if (cfg.point_color === "none") state.point_mode = "none";
  else if (cfg.point_color != null) { state.point_mode = "custom"; state.point_color = toHex(cfg.point_color); }
  if (cfg.point_radius != null) state.point_radius = cfg.point_radius;
  if (cfg.graticule) { state.grat_on = true; const g = cfg.graticule; if (g.step != null) state.grat_step = g.step; if (g.color != null) state.grat_color = toHex(g.color); if (g.opacity != null) state.grat_opacity = g.opacity; if (g.width != null) state.grat_width = g.width; }
  if (cfg.sphere) { state.sphere_on = true; const s = cfg.sphere; if (s.fill != null) state.sphere_fill = toHex(s.fill); if (s.stroke != null) state.sphere_stroke = toHex(s.stroke); if (s.stroke_width != null) state.sphere_stroke_width = s.stroke_width; }
  if (cfg.antimeridian) state.anti_on = true;
  if (cfg.tissot) { state.tissot_on = true; const ti = cfg.tissot; if (ti.step != null) state.tissot_step = ti.step; if (ti.radius != null) state.tissot_radius = ti.radius; if (ti.fill != null) state.tissot_fill = toHex(ti.fill); if (ti.fill_opacity != null) state.tissot_fill_opacity = ti.fill_opacity; }
  for (const k of Object.keys(cfg)) if (!KNOWN.has(k)) _extras[k] = cfg[k];
}

// ───────────────────────────── DOM + form ───────────────────────────────────
const $ = (id) => document.getElementById(id);
const elForm = $("form"), elCode = $("code"), elOut = $("preview"), elErr = $("error");
const elStatus = $("status"), elPreset = $("preset"), elDownload = $("download");
const elExtras = $("extras");
const setStatus = (m) => (elStatus.textContent = m || "");
const showError = (m) => { elErr.textContent = m; elErr.hidden = !m; };

const rows = [];   // { field, el }  for visibility refresh
const setters = {}; // field key → fn(value) that reflects state into the control

function buildForm() {
  for (const sec of SCHEMA) {
    const d = document.createElement("details"); d.className = "sec"; d.open = true;
    const sum = document.createElement("summary"); sum.textContent = sec.title; d.append(sum);
    const body = document.createElement("div"); body.className = "body"; d.append(body);
    for (const f of sec.fields) {
      const row = document.createElement("div"); row.className = "row";
      const lab = document.createElement("label"); lab.textContent = f.label; row.append(lab);
      const ctl = document.createElement("div"); ctl.className = "ctl"; row.append(ctl);
      makeControl(f, ctl);
      body.append(row);
      rows.push({ field: f, el: row });
    }
    elForm.append(d);
    sec._el = d;
  }
  refreshVis();
}
function makeControl(f, ctl) {
  const onChange = () => { renderTypst(); doRender(); refreshVis(); };
  if (f.t === "bool") {
    const w = document.createElement("label"); w.className = "switch";
    const cb = document.createElement("input"); cb.type = "checkbox"; cb.checked = state[f.k];
    cb.onchange = () => { state[f.k] = cb.checked; onChange(); };
    w.append(cb); ctl.append(w);
    setters[f.k] = (v) => (cb.checked = v);
  } else if (f.t === "sel") {
    const sel = document.createElement("select");
    for (const [v, label] of f.opts) { const o = document.createElement("option"); o.value = v; o.textContent = label; sel.append(o); }
    sel.value = state[f.k];
    sel.onchange = () => { state[f.k] = sel.value; onChange(); };
    ctl.append(sel);
    setters[f.k] = (v) => (sel.value = v);
  } else if (f.t === "color") {
    const c = document.createElement("input"); c.type = "color"; c.value = toHex(state[f.k]);
    c.oninput = () => { state[f.k] = c.value; renderTypst(); doRender(); };
    ctl.append(c);
    setters[f.k] = (v) => (c.value = toHex(v));
  } else if (f.t === "rng") {
    const r = document.createElement("input"); r.type = "range"; r.min = f.min; r.max = f.max; r.step = f.step; r.value = state[f.k];
    const n = document.createElement("input"); n.type = "number"; n.step = f.step; n.value = state[f.k];
    const sync = (v, live) => { state[f.k] = +v; r.value = v; n.value = v; renderTypst(); if (live) doRender(); };
    r.oninput = () => sync(r.value, true);
    n.oninput = () => sync(n.value, true);
    ctl.append(r, n);
    setters[f.k] = (v) => { r.value = v; n.value = v; };
  }
}
function refreshVis() {
  for (const { field, el } of rows) el.hidden = field.when ? !field.when(state) : false;
  for (const sec of SCHEMA) sec._el.hidden = sec.when ? !sec.when(state) : false;
}
function syncFormFromState() {
  for (const k in setters) setters[k](state[k]);
  refreshVis();
}
function refreshExtras() {
  const ks = Object.keys(_extras);
  elExtras.hidden = ks.length === 0;
  if (ks.length) elExtras.textContent = "Advanced config (not editable here, shown in the code): " + ks.join(", ");
}

// ───────────────────────────── render pipeline ──────────────────────────────
let _geojsonText = "", _currentFile = "data.json";
let _renderSeq = 0, _debounce = null, _rendering = false, _renderAgain = false;

function renderTypst() {
  const cfg = buildConfig();
  const dict = Object.keys(cfg).length ? jsonToTypst(cfg, 0) : null;
  const call = dict ? `#render-map(data, ${dict}, width: 100%)` : `#render-map(data, width: 100%)`;
  const src = [
    `#import "@preview/mercator:${VERSION}": *`, ``,
    `#let data = read("${_currentFile.split("/").pop()}", encoding: none)`, ``, call,
  ].join("\n");
  elCode.innerHTML = src.split("\n").map((l, i) =>
    `<div class="cline"><span class="gutter">${i + 1}</span><span class="src">${highlightTypstLine(l)}</span></div>`).join("");
}

async function doRender() {
  if (_rendering) { _renderAgain = true; return; }
  _rendering = true;
  const seq = ++_renderSeq;
  if (!_geojsonText.trim()) { showError("No data loaded."); _rendering = false; return; }
  setStatus("rendering…");
  try {
    const svg = await renderMap(ENC.encode(_geojsonText), JSON.stringify(buildConfig()));
    if (seq === _renderSeq) {
      elOut.innerHTML = svg;
      const s = elOut.querySelector("svg");
      if (s) { s.removeAttribute("width"); s.removeAttribute("height"); }
      showError("");
      setStatus(`rendered · ${(svg.length / 1024).toFixed(1)} KB SVG`);
      elDownload.disabled = false; elDownload.dataset.svg = svg;
      updateDragCursor();
    }
  } catch (e) {
    if (seq === _renderSeq) { showError(String(e.message || e)); setStatus("error"); }
  } finally {
    _rendering = false;
    if (_renderAgain) { _renderAgain = false; doRender(); }
  }
}

// ───────────────────────────── data sources ─────────────────────────────────
const PRESETS = {
  "World — orthographic globe": { file: "data/world.json", config: { projection: { type: "orthographic", center_lat: 30, center_lon: 10 }, sphere: { fill: "#dcefff", stroke: "#9cc4e0", stroke_width: 0.004 }, fill: "steelblue", fill_opacity: 0.95, stroke: "white", stroke_width: 0.0015, graticule: { step: 15, color: "#ffffff", opacity: 0.55, width: 0.4 } } },
  "World — Robinson + graticule": { file: "data/world.json", config: { projection: { type: "robinson" }, fill: "#6fbf5f", stroke: "white", stroke_width: 0.03, graticule: { step: 30, color: "#bbbbbb", opacity: 0.5, width: 0.3 } } },
  "World — antimeridian clip": { file: "data/world.json", config: { projection: { type: "equirectangular" }, antimeridian: true, fill: "#6fbf5f", stroke: "#356b2c", stroke_width: 0.04 } },
  "World — Tissot's indicatrix": { file: "data/world.json", config: { projection: { type: "mercator" }, fill: "none", stroke: "#aaaaaa", stroke_width: 0.01, graticule: { step: 30, color: "#dddddd", opacity: 0.4, width: 0.2 }, tissot: { step: 30, radius: 5, fill: "#ff0000", fill_opacity: 0.4 } } },
  "Sweden — choropleth + legend": { file: "data/swedish_regions.json", config: { projection: { type: "mercator", central_meridian: 16 }, fill_scale: { property: "color", type: "quantize", range: ["#fee5d9", "#fcae91", "#fb6a4a", "#de2d26", "#a50f15"] }, legend: { title: "color", pos: "bottom-left" }, stroke: "white", stroke_width: 0.02, point_color: "none" } },
  "Cities — proportional symbols": { file: "data/cities.geojson", config: { projection: { type: "mercator" }, point_radius_scale: { property: "pop", max_radius: 1.2 }, point_color: "crimson", fill_opacity: 0.6, stroke: "white", stroke_width: 0.06, label: "{name}", label_font_size: 0.5 } },
};

async function loadPreset(name) {
  const preset = PRESETS[name];
  if (!preset) return;
  setStatus("loading data…");
  try { _geojsonText = await (await fetch(preset.file)).text(); }
  catch (e) { showError("Could not load " + preset.file + ": " + e.message); return; }
  _currentFile = preset.file;
  loadStateFromConfig(preset.config);
  syncFormFromState();
  refreshExtras();
  renderTypst();
  doRender();
}
function loadFileText(name, text) {
  _geojsonText = text;
  _currentFile = name || "data.json";
  // Your data renders with the current form controls only — drop any preset's
  // data-specific config (a fill_scale keyed on a property your file lacks).
  _extras = {};
  refreshExtras();
  renderTypst();
  doRender();
}

// ───────────────────────── drag-to-rotate the globe ─────────────────────────
const elMain = $("main");
const elStage = document.querySelector(".preview-wrap");
let _drag = null;
function updateDragCursor() { elStage.style.cursor = AZIMUTHAL.has(state.proj_type) ? "grab" : "default"; }
elStage.addEventListener("pointerdown", (e) => {
  if (!AZIMUTHAL.has(state.proj_type)) return;
  _drag = { x: e.clientX, y: e.clientY, lon: state.center_lon, lat: state.center_lat };
  elStage.setPointerCapture(e.pointerId);
  elStage.style.cursor = "grabbing";
  e.preventDefault();
});
elStage.addEventListener("pointermove", (e) => {
  if (!_drag) return;
  const sens = 0.35; // deg/px
  // Grab-and-turn feel: dragging right/down brings western/southern land into view.
  const lon = ((_drag.lon - (e.clientX - _drag.x) * sens + 180) % 360 + 360) % 360 - 180;
  const lat = Math.max(-90, Math.min(90, _drag.lat + (e.clientY - _drag.y) * sens));
  state.center_lon = Math.round(lon * 10) / 10;
  state.center_lat = Math.round(lat * 10) / 10;
  setters.center_lon(state.center_lon);
  setters.center_lat(state.center_lat);
  renderTypst();
  doRender();
});
const endDrag = () => { if (_drag) { _drag = null; updateDragCursor(); } };
elStage.addEventListener("pointerup", endDrag);
elStage.addEventListener("pointercancel", endDrag);

// ───────────────────────── file drop + picker ───────────────────────────────
["dragenter", "dragover"].forEach((ev) => elMain.addEventListener(ev, (e) => { e.preventDefault(); elMain.classList.add("dragging"); }));
["dragleave", "drop"].forEach((ev) => elMain.addEventListener(ev, (e) => { e.preventDefault(); if (ev === "drop" || e.target === elMain) elMain.classList.remove("dragging"); }));
elMain.addEventListener("drop", async (e) => {
  const file = e.dataTransfer.files && e.dataTransfer.files[0];
  if (!file) return;
  try { loadFileText(file.name, await file.text()); elPreset.value = ""; }
  catch (err) { showError("Could not read file: " + err.message); }
});
$("browse").addEventListener("click", () => $("file").click());
$("file").addEventListener("change", async (e) => {
  const file = e.target.files && e.target.files[0];
  if (!file) return;
  try { loadFileText(file.name, await file.text()); elPreset.value = ""; }
  catch (err) { showError("Could not read file: " + err.message); }
});

// ───────────────────────────── misc events ──────────────────────────────────
for (const name of Object.keys(PRESETS)) { const o = document.createElement("option"); o.value = o.textContent = name; elPreset.append(o); }
elPreset.addEventListener("change", () => { if (elPreset.value) loadPreset(elPreset.value); });
elDownload.addEventListener("click", () => {
  const svg = elDownload.dataset.svg; if (!svg) return;
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([svg], { type: "image/svg+xml" }));
  a.download = "mercator-map.svg"; a.click(); URL.revokeObjectURL(a.href);
});
$("copy").addEventListener("click", async () => {
  const src = [...elCode.querySelectorAll(".src")].map((s) => s.textContent).join("\n");
  try { await navigator.clipboard.writeText(src); $("copy").textContent = "Copied"; setTimeout(() => ($("copy").textContent = "Copy"), 1200); }
  catch { /* clipboard blocked */ }
});

// ───────────────────────────── boot ─────────────────────────────────────────
buildForm();
(async () => {
  setStatus("compiling wasm…");
  try { await wReq({ kind: "ensure" }); }
  catch (e) { showError("Failed to load wasm: " + e.message); return; }
  const first = Object.keys(PRESETS)[0];
  elPreset.value = first;
  await loadPreset(first);
})();
