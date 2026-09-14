/// Mercator: Rendering GeoJSon in typst.
/// Author: Bernstein
/// Tip: make geo-json codeblocks render an image: `#show raw.where(lang: "geojson"): it => render-image(it.text)`

#let mercator = plugin("./mercator.wasm")

/// Builds a GeoJSON polygon approximating a spherical circle (a "range ring")
/// of `radius` degrees around `center`, following the great circle — like
/// d3.geoCircle. Returns a GeoJSON Feature dictionary; wrap several in a
/// `(type: "FeatureCollection", features: (..))` (optionally with your basemap)
/// and `json.encode` before passing to `render-map`.
///
/// - center (array): `(lon, lat)` of the circle's center, in degrees.
/// - radius (float): angular radius in degrees (e.g. `500 / 111.32` for ~500 km).
/// - steps (int): number of segments around the ring (default 64).
/// - properties (dictionary): properties attached to the feature (default empty).
/// -> dictionary
#let geo-circle(center: (0, 0), radius: 10, steps: 64, properties: (:)) = {
  let d2r = calc.pi / 180
  let lon1 = center.at(0) * d2r
  let lat1 = center.at(1) * d2r
  let d = radius * d2r
  let coords = ()
  for i in range(steps + 1) {
    let brng = 2 * calc.pi * i / steps
    let lat2 = calc.asin(calc.sin(lat1) * calc.cos(d) + calc.cos(lat1) * calc.sin(d) * calc.cos(brng)).rad()
    let lon2 = lon1 + calc.atan2(
      calc.cos(d) - calc.sin(lat1) * calc.sin(lat2),
      calc.sin(brng) * calc.sin(d) * calc.cos(lat1),
    ).rad()
    coords.push((lon2 / d2r, lat2 / d2r))
  }
  (type: "Feature", properties: properties, geometry: (type: "Polygon", coordinates: (coords,)))
}

/// Builds a GeoJSON LineString following the great-circle (shortest) path between
/// two points — the building block of a flow / connection map. Interpolates on the
/// sphere (slerp), so the arc curves correctly under any projection.
///
/// - from (array): `(lon, lat)` start, in degrees.
/// - to (array): `(lon, lat)` end, in degrees.
/// - steps (int): number of segments along the arc (default 48).
/// - properties (dictionary): properties attached to the feature (default empty).
/// -> dictionary
#let geo-arc(from, to, steps: 48, properties: (:)) = {
  let d2r = calc.pi / 180
  let a = (calc.cos(from.at(1) * d2r) * calc.cos(from.at(0) * d2r),
           calc.cos(from.at(1) * d2r) * calc.sin(from.at(0) * d2r),
           calc.sin(from.at(1) * d2r))
  let b = (calc.cos(to.at(1) * d2r) * calc.cos(to.at(0) * d2r),
           calc.cos(to.at(1) * d2r) * calc.sin(to.at(0) * d2r),
           calc.sin(to.at(1) * d2r))
  let dot = calc.max(-1, calc.min(1, a.at(0) * b.at(0) + a.at(1) * b.at(1) + a.at(2) * b.at(2)))
  let d = calc.acos(dot).rad()
  let coords = ()
  for i in range(steps + 1) {
    let t = i / steps
    let (f1, f2) = if d < 1e-6 { (1 - t, t) } else {
      (calc.sin((1 - t) * d) / calc.sin(d), calc.sin(t * d) / calc.sin(d))
    }
    let x = f1 * a.at(0) + f2 * b.at(0)
    let y = f1 * a.at(1) + f2 * b.at(1)
    let z = f1 * a.at(2) + f2 * b.at(2)
    coords.push((calc.atan2(x, y).rad() / d2r, calc.atan2(calc.sqrt(x * x + y * y), z).rad() / d2r))
  }
  (type: "Feature", properties: properties, geometry: (type: "LineString", coordinates: coords))
}

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

/// Projected bounds of a GeoJSON under a projection, as `(x, y, w, h)`.
/// Used by `render-layers` to align layers; also handy on its own.
#let map-bounds(code, projection: (:)) = {
  let out = mercator.bounds(bytes(code), bytes(json.encode((projection: projection))))
  json(out)
}

/// Projected centroid of each feature, as an array of `(x, y)` (or `none`), in
/// the same coordinate space as the rendered SVG / `viewbox`. Handy for placing
/// custom markers or labels at feature centers.
///
/// - code (string, bytes): the GeoJSON.
/// - projection (dictionary): the projection to measure under.
/// -> array
#let map-centroids(code, projection: (:)) = {
  json(mercator.centroids(bytes(code), bytes(json.encode((projection: projection)))))
}

/// Renders several GeoJSON layers onto one shared projection and viewbox, so a
/// basemap, boundaries, data overlay and points all line up — the equivalent of
/// drawing multiple d3 selections. Layers are drawn back-to-front (first = bottom).
///
/// - layers (array): each item is a dictionary with `data` (the GeoJSON) plus any
///   config keys for that layer (`fill`, `stroke`, `fill_scale`, `graticule`, …).
///   Put a `graticule`/`sphere`/`legend` on whichever layer should carry it.
/// - projection (dictionary): the shared projection, applied to every layer.
/// - viewbox (array, none): shared `(x, y, w, h)`; when `none`, the union of all
///   layers' projected bounds (with `viewbox-padding`) is used.
/// - viewbox-padding (float): padding fraction for the auto viewbox (default 0.15).
/// - all remaining arguments: see image (e.g. `width`); applied to every layer.
/// -> content
#let render-layers(layers, projection: (:), viewbox: none, viewbox-padding: 0.15, ..args) = {
  // Resolve a shared viewbox: explicit, or the union of each layer's bounds.
  let vb = viewbox
  if vb == none {
    let (minx, miny, maxx, maxy) = (none, none, none, none)
    for lyr in layers {
      let (x, y, w, h) = map-bounds(lyr.data, projection: projection)
      minx = if minx == none { x } else { calc.min(minx, x) }
      miny = if miny == none { y } else { calc.min(miny, y) }
      maxx = if maxx == none { x + w } else { calc.max(maxx, x + w) }
      maxy = if maxy == none { y + h } else { calc.max(maxy, y + h) }
    }
    let (w, h) = (maxx - minx, maxy - miny)
    let p = calc.max(w, h) * viewbox-padding
    vb = (minx - p, miny - p, calc.max(w + 2 * p, 1), calc.max(h + 2 * p, 1))
  }
  // Render each layer with the shared projection + viewbox, then overlay them.
  let imgs = layers.map(lyr => {
    let cfg = (:)
    for (k, v) in lyr { if k != "data" { cfg.insert(k, v) } }
    cfg.projection = projection
    cfg.viewbox = vb
    render-map(lyr.data, cfg, ..args.named())
  })
  if imgs.len() == 0 { return }
  // First image is in flow (sets the box size); the rest overlay it (same
  // viewbox + width → identical size → pixel-aligned).
  box(imgs.first() + imgs.slice(1).map(im => place(top + left, im)).join())
}
