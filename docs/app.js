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
  // Wrap when the one-line form (accounting for this depth's indent, plus room
  // for a `key: ` prefix) would run long. Arrays wrap a bit more eagerly than
  // dicts so a list of colors/coords lands one-per-line.
  const budget = 74 - indent * 2;
  if (Array.isArray(v)) {
    if (!v.length) return "()";
    const items = v.map((x) => jsonToTypst(x, indent + 1));
    const one = "(" + items.join(", ") + (items.length === 1 ? "," : "") + ")";
    // Keep short numeric tuples inline (domain, a coordinate, rotate); wrap
    // color/string lists and anything long onto one element per line.
    const inlineable = v.every((x) => typeof x === "number") || v.length <= 2;
    if (inlineable && one.length <= budget - 16 && !one.includes("\n")) return one;
    return "(\n" + items.map((it) => pad1 + it).join(",\n") + ",\n" + pad + ")";
  }
  const keys = Object.keys(v);
  if (!keys.length) return "(:)";
  const entries = keys.map((k) => `${identKey(k)}: ${jsonToTypst(v[k], indent + 1)}`);
  const one = "(" + entries.join(", ") + ")";
  // Keep only single-entry dicts inline (e.g. `(type: "mercator")`); expand any
  // dict with two or more entries one-per-line for readability.
  if (keys.length <= 1 && one.length <= budget && !one.includes("\n")) return one;
  return "(\n" + entries.map((e) => pad1 + e).join(",\n") + ",\n" + pad + ")";
}

// ───────────────────────────── projections ──────────────────────────────────
const PROJECTIONS = [
  "equirectangular", "mercator", "cassini", "miller", "gall_stereographic",
  "gall_peters", "robinson", "natural_earth", "winkel_tripel", "hammer",
  "mollweide", "sinusoidal", "eckert4", "eckert6", "kavrayskiy7", "wagner6",
  "aitoff", "van_der_grinten", "bonne", "polyconic",
  "lambert_conformal_conic", "albers_equal_area", "orthographic",
  "azimuthal_equidistant", "lambert_azimuthal_equal_area", "gnomonic", "wiechel",
  "peirce_quincuncial", "authagraph",
];
const AZIMUTHAL = new Set(["orthographic", "azimuthal_equidistant", "lambert_azimuthal_equal_area", "gnomonic", "wiechel"]);
const HAS_CM = new Set(["equirectangular", "mercator", "robinson", "natural_earth", "cassini", "polyconic", "hammer", "winkel_tripel", "bonne", "lambert_conformal_conic", "albers_equal_area", "mollweide", "sinusoidal", "miller", "aitoff", "eckert4", "gall_stereographic", "gall_peters", "eckert6", "kavrayskiy7", "wagner6", "van_der_grinten"]);
const CONIC = new Set(["lambert_conformal_conic", "albers_equal_area"]);
const ANTI_OK = new Set(["equirectangular", "mercator", "cassini", "miller", "gall_stereographic", "gall_peters"]);
const SCHEMES = [
  "blues", "greens", "oranges", "reds", "purples", "greys", "viridis", "magma",
  "ylgnbu", "ylorrd", "rdbu", "rdylbu", "brbg", "piyg", "spectral",
  "category10", "tableau10", "set1", "set2", "dark2",
];
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
    { k: "rot_l", label: "Rotate λ", t: "rng", def: 0, min: -180, max: 180, step: 1, when: (s) => !AZIMUTHAL.has(s.proj_type) },
    { k: "rot_p", label: "Rotate φ (tilt)", t: "rng", def: 0, min: -90, max: 90, step: 1, when: (s) => !AZIMUTHAL.has(s.proj_type) },
    { k: "rot_g", label: "Rotate γ (roll)", t: "rng", def: 0, min: -180, max: 180, step: 1, when: (s) => !AZIMUTHAL.has(s.proj_type) },
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
    { k: "point_shape", label: "Point shape", t: "sel", def: "circle", opts: [["circle", "circle"], ["square", "square"], ["diamond", "diamond"], ["triangle", "triangle"], ["cross", "cross"], ["star", "star"]], when: (s) => s.point_mode !== "none" },
    { k: "point_radius", label: "Point radius (0 = auto)", t: "rng", def: 0, min: 0, max: 2, step: 0.05 },
    { k: "precision", label: "Resample (0 = off)", t: "rng", def: 0, min: 0, max: 0.05, step: 0.002 },
  ] },
  { title: "Graticule", fields: [
    { k: "grat_on", label: "Enabled", t: "bool", def: false },
    { k: "grat_step", label: "Step °", t: "rng", def: 15, min: 5, max: 90, step: 5, when: (s) => s.grat_on },
    { k: "grat_color", label: "Color", t: "color", def: "#cccccc", when: (s) => s.grat_on },
    { k: "grat_opacity", label: "Opacity", t: "rng", def: 0.6, min: 0, max: 1, step: 0.05, when: (s) => s.grat_on },
    { k: "grat_width", label: "Width", t: "rng", def: 0.5, min: 0, max: 2, step: 0.1, when: (s) => s.grat_on },
  ] },
  { title: "Sphere / ocean", fields: [
    { k: "sphere_on", label: "Enabled (disc for globes, frame otherwise)", t: "bool", def: false },
    { k: "sphere_fill", label: "Ocean fill", t: "color", def: "#cfe8ff", when: (s) => s.sphere_on },
    { k: "sphere_stroke", label: "Outline", t: "color", def: "#3388cc", when: (s) => s.sphere_on },
    { k: "sphere_stroke_width", label: "Outline width", t: "rng", def: 0.005, min: 0, max: 0.05, step: 0.001, when: (s) => s.sphere_on },
  ] },
  { title: "Antimeridian clip", when: (s) => ANTI_OK.has(s.proj_type), fields: [
    { k: "anti_on", label: "Enabled", t: "bool", def: false },
  ] },
  { title: "Choropleth (fill_scale)", fields: [
    { k: "fs_on", label: "Enabled", t: "bool", def: false },
    { k: "fs_property", label: "Property", t: "text", def: "", when: (s) => s.fs_on },
    { k: "fs_type", label: "Scale", t: "sel", def: "quantize", when: (s) => s.fs_on,
      opts: [["quantize", "quantize"], ["quantile", "quantile"], ["linear", "linear"], ["diverging", "diverging"], ["category", "category"]] },
    { k: "fs_scheme", label: "Scheme", t: "sel", def: "blues", opts: SCHEMES.map((s) => [s, s]), when: (s) => s.fs_on },
    { k: "fs_n", label: "Classes", t: "rng", def: 5, min: 3, max: 9, step: 1, when: (s) => s.fs_on && (s.fs_type === "quantize" || s.fs_type === "quantile") },
    { k: "legend_on", label: "Legend", t: "bool", def: false, when: (s) => s.fs_on },
    { k: "legend_title", label: "Legend title", t: "text", def: "", when: (s) => s.fs_on && s.legend_on },
    { k: "legend_pos", label: "Legend position", t: "sel", def: "bottom-left", when: (s) => s.fs_on && s.legend_on,
      opts: [["top-left", "top-left"], ["top-right", "top-right"], ["bottom-left", "bottom-left"], ["bottom-right", "bottom-right"]] },
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
  if (!AZIMUTHAL.has(t) && (state.rot_l || state.rot_p || state.rot_g)) {
    cfg.rotate = [state.rot_l, state.rot_p, state.rot_g];
  }

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
  if (state.point_mode !== "none" && state.point_shape !== "circle") cfg.point_shape = state.point_shape;
  if (state.point_radius > 0) cfg.point_radius = state.point_radius;
  if (state.precision > 0) cfg.precision = state.precision;

  if (state.grat_on) cfg.graticule = { step: state.grat_step, color: state.grat_color, opacity: state.grat_opacity, width: state.grat_width };
  if (AZIMUTHAL.has(t) && state.sphere_on) cfg.sphere = { fill: state.sphere_fill, stroke: state.sphere_stroke, stroke_width: state.sphere_stroke_width };
  if (ANTI_OK.has(t) && state.anti_on) cfg.antimeridian = true;
  if (state.tissot_on) cfg.tissot = { step: state.tissot_step, radius: state.tissot_radius, fill: state.tissot_fill, fill_opacity: state.tissot_fill_opacity };

  if (state.fs_on && state.fs_property) {
    const fs = { property: state.fs_property, type: state.fs_type, scheme: state.fs_scheme };
    if (state.fs_type === "quantize" || state.fs_type === "quantile") fs.n = state.fs_n;
    cfg.fill_scale = fs;
    if (state.legend_on) {
      cfg.legend = { pos: state.legend_pos };
      if (state.legend_title) cfg.legend.title = state.legend_title;
    }
  }

  // Interactive zoom writes an explicit viewbox so the code reproduces the view.
  if (view) cfg.viewbox = view.map((n) => Math.round(n * 1000) / 1000);

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
const KNOWN = new Set(["projection", "rotate", "precision", "fill", "fill_opacity", "stroke", "stroke_width", "point_color", "point_shape", "point_radius", "graticule", "sphere", "antimeridian", "tissot", "fill_scale", "legend"]);
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
  if (Array.isArray(cfg.rotate)) { state.rot_l = cfg.rotate[0] || 0; state.rot_p = cfg.rotate[1] || 0; state.rot_g = cfg.rotate[2] || 0; }
  // Templated / named colors the color picker can't represent are kept verbatim.
  const isTemplate = (v) => typeof v === "string" && v.includes("{");
  if (isTemplate(cfg.fill)) _extras.fill = cfg.fill;
  else if (cfg.fill === "none") state.fill_mode = "none";
  else if (cfg.fill != null) { state.fill_mode = "color"; state.fill = toHex(cfg.fill); }
  if (typeof cfg.fill_opacity === "number") state.fill_opacity = cfg.fill_opacity;
  else if (isTemplate(cfg.fill_opacity)) _extras.fill_opacity = cfg.fill_opacity;
  if (isTemplate(cfg.stroke)) _extras.stroke = cfg.stroke;
  else if (cfg.stroke === "none") state.stroke_mode = "none";
  else if (cfg.stroke != null) { state.stroke_mode = "color"; state.stroke = toHex(cfg.stroke); }
  if (typeof cfg.stroke_width === "number") state.stroke_width = cfg.stroke_width;
  else if (isTemplate(cfg.stroke_width)) _extras.stroke_width = cfg.stroke_width;
  if (cfg.point_color === "none") state.point_mode = "none";
  else if (isTemplate(cfg.point_color)) _extras.point_color = cfg.point_color;
  else if (cfg.point_color != null) { state.point_mode = "custom"; state.point_color = toHex(cfg.point_color); }
  if (cfg.point_shape != null) state.point_shape = cfg.point_shape;
  if (cfg.point_radius != null) state.point_radius = cfg.point_radius;
  if (cfg.precision != null) state.precision = cfg.precision;
  if (cfg.graticule) { state.grat_on = true; const g = cfg.graticule; if (g.step != null) state.grat_step = g.step; if (g.color != null) state.grat_color = toHex(g.color); if (g.opacity != null) state.grat_opacity = g.opacity; if (g.width != null) state.grat_width = g.width; }
  if (cfg.sphere) { state.sphere_on = true; const s = cfg.sphere; if (s.fill != null) state.sphere_fill = toHex(s.fill); if (s.stroke != null) state.sphere_stroke = toHex(s.stroke); if (s.stroke_width != null) state.sphere_stroke_width = s.stroke_width; }
  if (cfg.antimeridian) state.anti_on = true;
  if (cfg.tissot) { state.tissot_on = true; const ti = cfg.tissot; if (ti.step != null) state.tissot_step = ti.step; if (ti.radius != null) state.tissot_radius = ti.radius; if (ti.fill != null) state.tissot_fill = toHex(ti.fill); if (ti.fill_opacity != null) state.tissot_fill_opacity = ti.fill_opacity; }
  if (cfg.fill_scale) {
    const fs = cfg.fill_scale;
    state.fs_on = true;
    if (fs.property != null) state.fs_property = fs.property;
    if (fs.type != null) state.fs_type = fs.type;
    if (fs.scheme != null) state.fs_scheme = fs.scheme;
    if (fs.n != null) state.fs_n = fs.n;
  }
  if (cfg.legend) { state.legend_on = true; if (cfg.legend.title != null) state.legend_title = cfg.legend.title; if (cfg.legend.pos != null) state.legend_pos = cfg.legend.pos; }
  for (const k of Object.keys(cfg)) if (!KNOWN.has(k)) _extras[k] = cfg[k];
}

// ───────────────────────────── DOM + form ───────────────────────────────────
const $ = (id) => document.getElementById(id);
const elForm = $("form"), elCode = $("code"), elOut = $("preview"), elErr = $("error");
const elStatus = $("status"), elPreset = $("preset");
let lastSvg = ""; // the most recent rendered SVG string (for download/PNG)
const elExtras = $("extras");
const setStatus = (m) => (elStatus.textContent = m || "");
const showError = (m) => { elErr.textContent = m; elErr.hidden = !m; };

const rows = [];   // { field, el, text }  for visibility refresh + search
const setters = {}; // field key → fn(value) that reflects state into the control
const searchSections = []; // { el, rows }  for the settings search

function buildForm() {
  for (const sec of SCHEMA) {
    const d = document.createElement("details"); d.className = "sec"; d.open = false;
    const sum = document.createElement("summary"); sum.textContent = sec.title; d.append(sum);
    const body = document.createElement("div"); body.className = "body"; d.append(body);
    const secRows = [];
    for (const f of sec.fields) {
      const row = document.createElement("div"); row.className = "row";
      const lab = document.createElement("label"); lab.textContent = f.label; row.append(lab);
      const ctl = document.createElement("div"); ctl.className = "ctl"; row.append(ctl);
      makeControl(f, ctl);
      body.append(row);
      const entry = { field: f, el: row, text: `${sec.title} ${f.label} ${f.k}`.toLowerCase() };
      rows.push(entry);
      secRows.push(entry);
    }
    elForm.append(d);
    sec._el = d;
    searchSections.push({ el: d, rows: secRows });
  }
  refreshVis();
}

// Filter the form by a query — hides non-matching rows/sections (via a class with
// !important so it composes with when-visibility) and expands sections with a hit.
function filterForm(query) {
  const q = (query || "").trim().toLowerCase();
  for (const r of rows) r.el.classList.toggle("search-hidden", !!q && !r.text.includes(q));
  for (const sec of searchSections) {
    const hit = sec.rows.some((r) => !r.el.classList.contains("search-hidden"));
    sec.el.classList.toggle("search-hidden", !!q && !hit);
    sec.el.open = q ? hit : false;
  }
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
    sel.onchange = () => { state[f.k] = sel.value; if (f.k === "proj_type") resetZoom(); onChange(); };
    ctl.append(sel);
    setters[f.k] = (v) => (sel.value = v);
  } else if (f.t === "color") {
    const c = document.createElement("input"); c.type = "color"; c.value = toHex(state[f.k]);
    c.oninput = () => { state[f.k] = c.value; renderTypst(); doRender(); };
    ctl.append(c);
    setters[f.k] = (v) => (c.value = toHex(v));
  } else if (f.t === "text") {
    const t = document.createElement("input"); t.type = "text"; t.value = state[f.k];
    t.oninput = () => { state[f.k] = t.value; renderTypst(); scheduleRender(); };
    ctl.append(t);
    setters[f.k] = (v) => (t.value = v);
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
  const keys = Object.keys(cfg);
  let call;
  if (!keys.length) {
    call = `#render-map(data, width: 100%)`;
  } else {
    // Always expand the top-level config one entry per line (idiomatic, like the
    // docs); nested short dicts stay inline via jsonToTypst.
    const entries = keys.map((k) => `  ${identKey(k)}: ${jsonToTypst(cfg[k], 1)}`);
    call = `#render-map(data, (\n${entries.join(",\n")},\n), width: 100%)`;
  }
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
      lastSvg = svg;
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
  "World — Mollweide (ocean frame)": { file: "data/world.json", config: { projection: { type: "mollweide" }, sphere: { fill: "#dceefb", stroke: "#8899bb", stroke_width: 0.006 }, fill: "#6fbf5f", stroke: "white", stroke_width: 0.008, graticule: { step: 20, color: "#ffffff", opacity: 0.7, width: 0.3 } } },
  "World — Hammer, oblique (rotate)": { file: "data/world.json", config: { projection: { type: "hammer" }, rotate: [-60, -25, 0], sphere: { fill: "#dceefb", stroke: "#8899bb", stroke_width: 0.006 }, fill: "#6fbf5f", stroke: "white", stroke_width: 0.008, graticule: { step: 20, color: "#ffffff", opacity: 0.7, width: 0.3 } } },
  "World — Van der Grinten": { file: "data/world.json", config: { projection: { type: "van_der_grinten" }, sphere: { fill: "#dceefb", stroke: "#8899bb", stroke_width: 0.008 }, fill: "#6fbf5f", stroke: "white", stroke_width: 0.006, graticule: { step: 30, color: "#ffffff", opacity: 0.7, width: 0.4 } } },
  "World — clip to a region": { file: "data/world.json", config: { projection: { type: "equirectangular" }, fill: "#6fbf5f", stroke: "white", stroke_width: 0.03, graticule: { step: 20, color: "#cccccc", opacity: 0.5, width: 0.2 }, clip_extent: [-25, -75, 55, 40] } },
  "World — Tissot's indicatrix": { file: "data/world.json", config: { projection: { type: "mercator" }, fill: "none", stroke: "#aaaaaa", stroke_width: 0.01, graticule: { step: 30, color: "#dddddd", opacity: 0.4, width: 0.2 }, tissot: { step: 30, radius: 5, fill: "#ff0000", fill_opacity: 0.4 } } },
  "Sweden — choropleth + legend": { file: "data/swedish_regions.json", config: { projection: { type: "mercator", central_meridian: 16 }, fill_scale: { property: "color", type: "quantize", scheme: "reds", n: 5 }, legend: { title: "color", pos: "bottom-left" }, stroke: "white", stroke_width: 0.02, point_color: "none" } },
  "Sweden — categorical fill": { file: "data/swedish_regions.json", config: { projection: { type: "mercator", central_meridian: 16 }, fill_scale: { property: "color", type: "category", scheme: "tableau10" }, legend: { title: "class", pos: "bottom-left" }, stroke: "white", stroke_width: 0.02, point_color: "none" } },
  "Sweden — diverging + gradient legend": { file: "data/swedish_regions.json", config: { projection: { type: "mercator", central_meridian: 16 }, fill_scale: { property: "color", type: "diverging", scheme: "rdbu", domain: [0, 4], midpoint: 2 }, legend: { title: "color", pos: "bottom-left" }, stroke: "white", stroke_width: 0.02, point_color: "none" } },
  "Sweden — per-feature fill + patterns": { file: "data/swedish_regions.json", config: { projection: { type: "mercator", central_meridian: 16 }, fill: "{fill_color}", fill_pattern: "{pattern}", stroke: "white", stroke_width: 0.02, point_color: "none" } },
  "Sweden — dashed borders + labels": { file: "data/swedish_regions.json", config: { projection: { type: "mercator", central_meridian: 16 }, fill: "#eef3f8", stroke: "#8899bb", stroke_width: 0.03, stroke_dash: "0.25 0.15", point_color: "none", label: "{name}", label_font_size: 0.32, label_halo: "white", label_halo_width: 0.12, label_collide: true } },
  "Sweden — Dorling cartogram": { file: "data/swedish_regions.json", config: { projection: { type: "mercator", central_meridian: 16 }, fill_scale: { property: "color", type: "quantize", scheme: "reds", n: 5 }, dorling: { property: "l_id", max_radius: 1.0, stroke: "white", stroke_width: 0.02 } } },
  "Points — hexbin density": { file: "data/points.geojson", config: { projection: { type: "mercator", central_meridian: 15 }, hexbin: { radius: 0.2, scheme: "ylorrd", n: 6, stroke: "white", stroke_width: 0.004 } } },
  "Points — density contours": { file: "data/points.geojson", config: { projection: { type: "mercator", central_meridian: 15 }, contour: { bandwidth: 6, n: 7, scheme: "viridis", stroke_width: 0.02 } } },
  "Cities — proportional symbols": { file: "data/cities.geojson", config: { projection: { type: "mercator" }, point_radius_scale: { property: "pop", max_radius: 1.2, min_radius: 0.1 }, point_color: "crimson", fill_opacity: 0.6, stroke: "white", stroke_width: 0.06, size_legend: { title: "population", pos: "bottom-right" }, label: "{name}", label_font_size: 0.5 } },
  "Cities — star markers": { file: "data/cities.geojson", config: { projection: { type: "mercator" }, point_shape: "star", point_radius: 0.5, point_color: "crimson", stroke: "white", stroke_width: 0.05, label: "{name}", label_font_size: 0.4, label_halo: "white", label_halo_width: 0.1 } },
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
  resetZoom();
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

// ─────────────── orbit (1-finger drag) + pinch / wheel zoom ──────────────────
// Orbit re-renders the map (changes the projection center). Zoom sets an explicit
// projected `viewbox` (the same config a Typst document uses to frame a region),
// so the code reflects the view and it's reproducible. The live SVG viewBox is
// nudged instantly for feedback, then re-rendered so legend/graticule follow.
const elMain = $("main");
const elStage = document.querySelector(".preview-wrap");
const pointers = new Map();
let orbit = null, pinch = null;
let view = null; // explicit projected viewbox [x, y, w, h], or null = auto-fit

const dist = (a, b) => Math.hypot(a.x - b.x, a.y - b.y);
const mid = (a, b) => ({ x: (a.x + b.x) / 2, y: (a.y + b.y) / 2 });
const svgEl = () => elOut.querySelector("svg");

const svgViewBox = () => {
  const s = svgEl(), vb = s && s.getAttribute("viewBox");
  return vb ? vb.split(/\s+/).map(Number) : null;
};
function resetZoom() { view = null; }
function applyLiveView() {
  const s = svgEl();
  if (s && view) s.setAttribute("viewBox", view.map((n) => +n.toFixed(4)).join(" "));
}
function updateDragCursor() {
  elStage.style.cursor = AZIMUTHAL.has(state.proj_type) ? "grab" : (view ? "move" : "default");
}
// Screen point → current view coordinates (letterbox-aware, meet fit).
function pxToView(cx, cy) {
  const s = svgEl(), v = view || svgViewBox();
  if (!s || !v) return null;
  const r = s.getBoundingClientRect();
  const sc = Math.min(r.width / v[2], r.height / v[3]);
  const ox = (r.width - v[2] * sc) / 2, oy = (r.height - v[3] * sc) / 2;
  return [v[0] + (cx - r.left - ox) / sc, v[1] + (cy - r.top - oy) / sc];
}
// After the gesture settles, re-render so the wasm bakes the viewbox (and moves
// the legend/graticule) and the code stays in sync.
let _zoomTimer = null;
function commitZoom() {
  clearTimeout(_zoomTimer);
  _zoomTimer = setTimeout(() => { renderTypst(); doRender(); }, 180);
}
// Zoom about screen point (cx,cy) by `factor` (>1 = in), optionally panning first.
function adjustView(cx, cy, factor, dxPx, dyPx) {
  if (!view) view = svgViewBox();
  if (!view) return;
  const s = svgEl(), r = s.getBoundingClientRect();
  const sc0 = Math.min(r.width / view[2], r.height / view[3]);
  if (dxPx || dyPx) view = [view[0] - dxPx / sc0, view[1] - dyPx / sc0, view[2], view[3]];
  if (factor && factor !== 1) {
    const a = pxToView(cx, cy);
    const v = view, nw = v[2] / factor, nh = v[3] / factor;
    const fx = (a[0] - v[0]) / v[2], fy = (a[1] - v[1]) / v[3];
    view = [a[0] - fx * nw, a[1] - fy * nh, nw, nh];
  }
  applyLiveView();  // instant feedback (vector-crisp)
  renderTypst();    // code reflects the new viewbox
  commitZoom();     // re-render after settle so legend/graticule follow
}

elStage.addEventListener("pointerdown", (e) => {
  if (e.target.closest("#tools")) return; // let tool buttons/links act normally
  pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
  elStage.setPointerCapture(e.pointerId);
  if (pointers.size === 2) {
    orbit = null; // second finger down → switch from orbit to pinch
    const [a, b] = [...pointers.values()];
    pinch = { d: dist(a, b), m: mid(a, b) };
  } else if (pointers.size === 1 && AZIMUTHAL.has(state.proj_type)) {
    orbit = { x: e.clientX, y: e.clientY, lon: state.center_lon, lat: state.center_lat };
    elStage.style.cursor = "grabbing";
  }
  e.preventDefault();
});
elStage.addEventListener("pointermove", (e) => {
  if (!pointers.has(e.pointerId)) return;
  pointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
  if (pinch && pointers.size >= 2) {
    const [a, b] = [...pointers.values()];
    const d = dist(a, b), m = mid(a, b);
    const factor = pinch.d > 0 ? d / pinch.d : 1;
    adjustView(m.x, m.y, factor, m.x - pinch.m.x, m.y - pinch.m.y);
    pinch = { d, m };
  } else if (orbit && pointers.size === 1) {
    const sens = 0.35; // deg/px — drag right/down brings western/southern land into view
    const lon = ((orbit.lon - (e.clientX - orbit.x) * sens + 180) % 360 + 360) % 360 - 180;
    const lat = Math.max(-90, Math.min(90, orbit.lat + (e.clientY - orbit.y) * sens));
    state.center_lon = Math.round(lon * 10) / 10;
    state.center_lat = Math.round(lat * 10) / 10;
    setters.center_lon(state.center_lon);
    setters.center_lat(state.center_lat);
    renderTypst();
    doRender();
  }
});
function liftPointer(e) {
  pointers.delete(e.pointerId);
  if (pointers.size < 2) pinch = null;
  if (pointers.size === 0) { orbit = null; updateDragCursor(); }
}
elStage.addEventListener("pointerup", liftPointer);
elStage.addEventListener("pointercancel", liftPointer);

// Wheel / trackpad zoom, focused on the cursor.
elStage.addEventListener("wheel", (e) => {
  e.preventDefault();
  adjustView(e.clientX, e.clientY, e.deltaY < 0 ? 1.15 : 1 / 1.15, 0, 0);
  updateDragCursor();
}, { passive: false });

// Double-click / double-tap resets the zoom to the auto-fit viewbox.
elStage.addEventListener("dblclick", () => {
  resetZoom();
  renderTypst();
  doRender();
  updateDragCursor();
});

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
$("search").addEventListener("input", (e) => filterForm(e.target.value));

// ───────────────────────── tools: download / reset / share ───────────────────
function saveBlob(blob, name) {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = name;
  a.click();
  URL.revokeObjectURL(a.href);
}
$("dl-svg").addEventListener("click", () => {
  if (lastSvg) saveBlob(new Blob([lastSvg], { type: "image/svg+xml" }), "mercator-map.svg");
});
$("dl-png").addEventListener("click", () => {
  if (!lastSvg) return;
  const s = svgEl();
  const vb = (s && s.getAttribute("viewBox") || "").split(/\s+/).map(Number);
  const aspect = vb.length === 4 && vb[3] > 0 ? vb[2] / vb[3] : 1;
  const W = 1600, H = Math.max(1, Math.round(W / aspect));
  // The wasm SVG has a viewBox but no width/height; give it an intrinsic size.
  const sized = lastSvg.replace("<svg ", `<svg width="${W}" height="${H}" `);
  const url = URL.createObjectURL(new Blob([sized], { type: "image/svg+xml" }));
  const img = new Image();
  img.onload = () => {
    const c = document.createElement("canvas");
    c.width = W; c.height = H;
    const ctx = c.getContext("2d");
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, W, H);
    ctx.drawImage(img, 0, 0, W, H);
    URL.revokeObjectURL(url);
    c.toBlob((b) => b && saveBlob(b, "mercator-map.png"), "image/png");
  };
  img.onerror = () => { URL.revokeObjectURL(url); showError("PNG export failed."); };
  img.src = url;
});
$("reset").addEventListener("click", () => {
  resetZoom();
  if (elPreset.value) loadPreset(elPreset.value);
  else doRender();
  history.replaceState(null, "", location.pathname);
});
$("share").addEventListener("click", async () => {
  const url = await shareUrl();
  history.replaceState(null, "", url);
  const btn = $("share"), prev = btn.textContent;
  try { await navigator.clipboard.writeText(url); btn.textContent = "Copied!"; }
  catch { btn.textContent = "Link in URL"; }
  setTimeout(() => (btn.textContent = prev), 1400);
});

$("copy").addEventListener("click", async () => {
  const src = [...elCode.querySelectorAll(".src")].map((s) => s.textContent).join("\n");
  try { await navigator.clipboard.writeText(src); $("copy").textContent = "Copied"; setTimeout(() => ($("copy").textContent = "Copy"), 1200); }
  catch { /* clipboard blocked */ }
});

// Click the "demo" pill to reveal the deployed commit (stamped by CI; "local"
// when served from a working copy).
{
  const b = $("build");
  if (b) {
    b.style.cursor = "pointer";
    let shown = false;
    b.addEventListener("click", () => {
      shown = !shown;
      const c = b.dataset.commit;
      const commit = c && !c.includes("BUILD_COMMIT") ? c : "local";
      b.textContent = shown ? commit : "demo";
    });
  }
}

// ───────────────────────── shareable links ──────────────────────────────────
// Encode the data source + config as `?s=<deflate+base64url>` (or `?j=<base64url>`
// when the browser lacks CompressionStream). Decoded on load.
const b64u = (bytes) => btoa(String.fromCharCode(...bytes)).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
const unb64u = (s) => { const bin = atob(s.replace(/-/g, "+").replace(/_/g, "/")); const a = new Uint8Array(bin.length); for (let i = 0; i < bin.length; i++) a[i] = bin.charCodeAt(i); return a; };
const hasZip = typeof CompressionStream !== "undefined" && typeof DecompressionStream !== "undefined";
async function deflate(str) { const cs = new CompressionStream("deflate-raw"); const w = cs.writable.getWriter(); w.write(ENC.encode(str)); w.close(); return new Uint8Array(await new Response(cs.readable).arrayBuffer()); }
async function inflate(bytes) { const ds = new DecompressionStream("deflate-raw"); const w = ds.writable.getWriter(); w.write(bytes); w.close(); return DEC.decode(await new Response(ds.readable).arrayBuffer()); }
const baseUrl = () => location.origin + location.pathname;
async function shareUrl() {
  const json = JSON.stringify({ f: _currentFile, c: buildConfig() });
  if (hasZip) {
    try { return baseUrl() + "?s=" + b64u(await deflate(json)); } catch { /* fall through */ }
  }
  return baseUrl() + "?j=" + b64u(new TextEncoder().encode(json));
}
async function decodeUrl() {
  const p = new URLSearchParams(location.search);
  try {
    if (p.get("s")) return JSON.parse(await inflate(unb64u(p.get("s"))));
    if (p.get("j")) return JSON.parse(new TextDecoder().decode(unb64u(p.get("j"))));
  } catch { /* malformed link */ }
  return null;
}

// ───────────────────────────── boot ─────────────────────────────────────────
buildForm();
(async () => {
  setStatus("compiling wasm…");
  try { await wReq({ kind: "ensure" }); }
  catch (e) { showError("Failed to load wasm: " + e.message); return; }

  const shared = await decodeUrl();
  if (shared && shared.f) {
    // Restore a shared view: load its data file + config.
    setStatus("loading shared view…");
    try {
      _geojsonText = await (await fetch(shared.f)).text();
      _currentFile = shared.f;
      loadStateFromConfig(shared.c || {});
      syncFormFromState();
      refreshExtras();
      resetZoom();
      if (Array.isArray(shared.c && shared.c.viewbox)) view = shared.c.viewbox.slice();
      elPreset.value = "";
      renderTypst();
      doRender();
      return;
    } catch (e) { showError("Could not load shared view: " + e.message); }
  }

  const first = Object.keys(PRESETS)[0];
  elPreset.value = first;
  await loadPreset(first);
})();
