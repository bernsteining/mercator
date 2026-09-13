/// Mercator: Rendering GeoJSon in typst.
/// Author: Bernstein
/// Tip: make geo-json codeblocks render an image: `#show raw.where(lang: "geojson"): it => render-image(it.text)`

#let mercator = plugin("./mercator.wasm")

/// Renders a GeoJSON and returns SVG code for it.
///
/// - code (string, bytes): GeoJSON to be rendered.
/// - config (string, dictionary): Configuration as a dictionary or JSON string.
/// -> string
#let render(code, config) = {
  let cfg = if type(config) == dictionary { json.encode(config) } else { config }
  return str(mercator.geo(bytes(code), bytes(cfg)))
}

/// Merges external `data` into a GeoJSON's feature properties, matched on `key`.
///
/// Lets you keep geometry (boundaries) and data (a statistics table) in separate
/// sources — the classic GIS "join" — instead of pre-embedding the data. Values
/// from `data` are added to each matching feature's properties (overriding any
/// existing keys), so a `fill_scale` can then read them.
///
/// - code (string, bytes): source GeoJSON.
/// - data (dictionary, array): table to join. Either a dictionary keyed by the
///   join value (e.g. `("SE-01": (pop: 100))`), or an array of records each
///   holding the join value under `by` (e.g. from `csv(.., row-type: dictionary)`).
/// - key (string): feature property to match on (default "id").
/// - by (string, none): field in array records holding the join value (default = `key`).
/// -> string
#let join(code, data, key: "id", by: none) = {
  let gj = json(bytes(code))
  if "features" not in gj { return json.encode(gj) }
  let by = if by == none { key } else { by }
  let lookup = if type(data) == dictionary {
    data
  } else {
    let m = (:)
    for rec in data { m.insert(str(rec.at(by)), rec) }
    m
  }
  gj.features = gj.features.map(f => {
    let props = f.at("properties", default: (:))
    if props == none { props = (:) }
    let jval = str(props.at(key, default: ""))
    if jval in lookup { props = props + lookup.at(jval) }
    f.properties = props
    f
  })
  json.encode(gj)
}

/// Renders a GeoJSON and returns an image for it. Uses the same parameters as image.
///
/// - code (string, bytes): GeoJSON to be rendered.
/// - config (dictionary, string): Configuration as a dictionary or JSON string. Optional, defaults to empty config.
/// - data (dictionary, array): optional table to `join` into feature properties before rendering.
/// - key (string): join key when `data` is given (default "id").
/// - by (string, none): array-record join field when `data` is given (default = `key`).
/// - all remaining arguments: see image
/// -> content
#let render-map(code, ..args) = {
  let config = args.pos().at(0, default: (:))
  let named = args.named()
  let code = if "data" in named {
    join(code, named.data, key: named.at("key", default: "id"), by: named.at("by", default: none))
  } else { code }
  let img-args = (:)
  for (k, v) in named {
    if k not in ("data", "key", "by") { img-args.insert(k, v) }
  }
  let cfg = if type(config) == dictionary { json.encode(config) } else { config }
  image(bytes(render(code, cfg)), format: "svg", ..img-args)
}
