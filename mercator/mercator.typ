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

/// Renders a GeoJSON and returns an image for it. Uses the same parameters as image.
///
/// - code (string, bytes): GeoJSON to be rendered.
/// - config (dictionary, string): Configuration as a dictionary or JSON string. Optional, defaults to empty config.
/// - all remaining arguments: see image
/// -> content
#let render-map(code, ..args) = {
  let config = args.pos().at(0, default: (:))
  let cfg = if type(config) == dictionary { json.encode(config) } else { config }
  image(bytes(render(code, cfg)), format: "svg", ..args.named())
}
