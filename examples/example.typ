#import "../mercator/mercator.typ": *

#set page(width: 15cm, height: auto, margin: 1cm)
#set text(font: "New Computer Modern")

#let sweden = read("data/swedish_regions.json", encoding: "utf8")
#let sweden_config = json.encode((
  stroke: "white",
  stroke_width: 0.03,
  fill: "steelblue",
  fill_opacity: 0.8,
  label: "{name}",
  label_color: "black",
  label_font_size: 0.25,
))

#let zoom_config = json.encode((
  "stroke": "black",
  "stroke_width": 0.02,
  "fill": "grey",
  "fill_opacity": 0.5,
  "viewbox": array((15.0, -69.4, 10.0, 6.0))))


#let pattern_config = json.encode((
  stroke: "black",
  stroke_width: 0.02,
  fill: "{fill_color}",
  fill_opacity: 0.9,
  fill_pattern: "{pattern}",
  label: "{name}",
  label_font_size: 0.2,
))

#let france = read("data/departements_fr.json", encoding: "utf8")
#let france_config = json.encode((
  stroke: "red",
  stroke_width: 0.003,
  fill: "white",
  fill_opacity: 0.8,
))

#let belgium_config = json.encode((
  stroke: "white",
  stroke_width: 0.003,
  fill: "red",
  fill_opacity: 1,
))

#let inline_config = json.encode((
  stroke: "goldenrod",
  stroke_width: 0.08,
  fill: "gold",
  fill_opacity: 0.7,
))

= Mercator — Examples

== Labels

#figure(
  render-map(sweden, sweden_config, width: 80%),
  caption: "Labels from GeoJSON properties with `\"{name}\"`",
)

== Custom viewbox

Use `viewbox` as `(x, y, width, height)` to zoom into a specific area.

#figure(
  render-map(sweden, zoom_config, width: 80%),
  caption: "Custom viewbox",
)

== Fill patterns

Use `fill_pattern` with `"hatched"`, `"crosshatched"`, or `"dotted"`. Supports per-feature interpolation via `{property_name}` — regions without the property get a solid fill.

#figure(
  render-map(sweden, pattern_config, width: 80%),
  caption: "Per-feature colors and fill patterns (hatched, crosshatched, dotted, solid)",
)

== TopoJSON

Mercator also accepts TopoJSON input.

#figure(
  render-map(read("data/belgium.json", encoding: "utf8"), belgium_config, width: 80%),
  caption: "Belgian municipalities from a TopoJSON file",
)

== Inline GeoJSON with show rule

#show raw.where(lang: "geojson"): it => align(center, render-map(it.text, inline_config, width: 40%))

```geojson
{"type":"GeometryCollection","geometries":[{"type":"Polygon","coordinates":[[[9.5,5.0],[9.46,5.59],[9.35,6.16],[9.16,6.72],[8.9,7.25],[8.57,7.74],[8.18,8.18],[7.74,8.57],[7.25,8.9],[6.72,9.16],[6.16,9.35],[5.59,9.46],[5.0,9.5],[4.41,9.46],[3.84,9.35],[3.28,9.16],[2.75,8.9],[2.26,8.57],[1.82,8.18],[1.43,7.74],[1.1,7.25],[0.84,6.72],[0.65,6.16],[0.54,5.59],[0.5,5.0],[0.54,4.41],[0.65,3.84],[0.84,3.28],[1.1,2.75],[1.43,2.26],[1.82,1.82],[2.26,1.43],[2.75,1.1],[3.28,0.84],[3.84,0.65],[4.41,0.54],[5.0,0.5],[5.59,0.54],[6.16,0.65],[6.72,0.84],[7.25,1.1],[7.74,1.43],[8.18,1.82],[8.57,2.26],[8.9,2.75],[9.16,3.28],[9.35,3.84],[9.46,4.41],[9.5,5.0]]]},{"type":"Polygon","coordinates":[[[3.85,6.2],[3.81,6.41],[3.69,6.59],[3.51,6.71],[3.3,6.75],[3.09,6.71],[2.91,6.59],[2.79,6.41],[2.75,6.2],[2.79,5.99],[2.91,5.81],[3.09,5.69],[3.3,5.65],[3.51,5.69],[3.69,5.81],[3.81,5.99],[3.85,6.2]]]},{"type":"Polygon","coordinates":[[[7.25,6.2],[7.21,6.41],[7.09,6.59],[6.91,6.71],[6.7,6.75],[6.49,6.71],[6.31,6.59],[6.19,6.41],[6.15,6.2],[6.19,5.99],[6.31,5.81],[6.49,5.69],[6.7,5.65],[6.91,5.69],[7.09,5.81],[7.21,5.99],[7.25,6.2]]]},{"type":"LineString","coordinates":[[2.86,3.4],[3.06,3.18],[3.3,2.98],[3.55,2.81],[3.82,2.66],[4.1,2.55],[4.39,2.47],[4.7,2.42],[5.0,2.4],[5.3,2.42],[5.61,2.47],[5.9,2.55],[6.18,2.66],[6.45,2.81],[6.7,2.98],[6.94,3.18],[7.14,3.4]]}]}
```
