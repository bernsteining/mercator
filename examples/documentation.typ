#import "../mercator/mercator.typ": *

#set page(width: 15cm, height: 21cm, margin: 1cm)
#set text(font: "New Computer Modern")

// --- Data ---

#let sweden = read("data/swedish_regions.json", encoding: "utf8")
#let world = read("data/worldmap.json", encoding: "utf8")
#let belgium = read("data/belgium.json", encoding: "utf8")

// --- Example helper ---
// Show rule that displays a code block and executes it.

#let doc-scope = ("render-map": render-map, sweden: sweden, world: world, belgium: belgium)
#show raw.where(lang: "example"): it => {
  raw(block: true, lang: "typst", it.text)
  eval(it.text, mode: "markup", scope: doc-scope)
}

// --- Hero ---

#let hero_data = world
#let hero_config = json.encode((
  stroke: "white",
  stroke_width: 0.001,
  fill: "steelblue",
  fill_opacity: 0.85,
  projection: (type: "orthographic", center_lat: 45, center_lon: 10),
  graticule: (step: 15, color: "red", opacity: 0.5),
))

#align(center + horizon)[
  #text(size: 32pt, weight: "bold", "Mercator")

  #v(0.5em)

  #text(size: 12pt, fill: gray)[A Typst plugin for rendering GeoJSON and TopoJSON]

  #v(0.3em)

  #link("https://github.com/bernsteining/mercator")[#text(size: 10pt, fill: blue)[github.com/bernsteining/mercator]] · #link("https://typst.app/universe/package/mercator")[#text(size: 10pt, fill: blue)[typst.app/universe/package/mercator]]

  #v(1.5em)

  #render-map(hero_data, hero_config, width: 95%)

]

#pagebreak()

== Quick start

```example
#let sweden = read("data/swedish_regions.json", encoding: "utf8")
#render-map(sweden, json.encode((
)), width: 80%)
```

The `render-map` function takes three arguments:
- `data` — GeoJSON or TopoJSON string
- `config` — JSON-encoded configuration string
- `width` — rendering width (Typst length)

If no config is passed the maps will default to this visually.

#pagebreak()

== Configuration overview

All rendering options are passed as a JSON-encoded dictionary. Every field is optional and defaults to the value shown below. Fields marked with `{property}` support per-feature interpolation from GeoJSON properties. (cf. per-feature styling section)

```json
{
  // --- Appearance ---
  "stroke":       "black", // string – CSS color name or hex
  "stroke_width": 0.05,    // float – border thickness
  "fill":         "red",   // string – CSS color name or hex
  "fill_opacity": 0.5,     // float – 0.0 to 1.0 (transparent⟶opaque)
  "fill_pattern": null,    // "hatched" | "crosshatched" | "dotted"
  "point_radius": null,    // float – defaults to stroke_width × 5
  "point_color":  null,    // string – defaults to fill; "none" hides points

  // --- Labels ---
  "label":             null,    // string "{name}" or array of line objects
  "label_color":       "black", // string
  "label_font_size":   0.3,     // float
  "label_font_family": "Arial", // string

  // --- Viewbox ---
  "viewbox":         null,  // [x, y, width, height] – auto-computed if null
  "viewbox_padding": 0.1,   // float – padding fraction around auto viewbox

  // --- Projection ---
  "projection": null,  // object, see Projections section
  // --- Graticule ---
  "graticule": null    // object, see Graticule section
}
```

Mercator handles all standard GeoJSON geometry types:

- `Point` / `MultiPoint` — rendered as circles (radius controlled by `point_radius`)
- `LineString` / `MultiLineString` — rendered as stroked paths
- `Polygon` / `MultiPolygon` — rendered as filled and stroked shapes
- `GeometryCollection` — all contained geometries are rendered

#pagebreak()

== Styling

To control the map appearance, configure:
- `stroke`
- `stroke_width`
- `fill`
- `fill_opacity` 


#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  [
    ```example
    #render-map(sweden, json.encode((
      stroke: "white",
      stroke_width: 0.01,
      fill: "teal",
      fill_opacity: 0.8,
      point_color: "none",
    )), width: 100%)
    ```
  ],
  [
    ```example
    #render-map(sweden, json.encode((
      stroke: "#333",
      stroke_width: 0.08,
      fill: "#f7fc0f",
      fill_opacity: 0.2,
      point_color: "none",
    )), width: 100%)
    ```
  ],
)

== Viewbox

By default, the viewbox is auto-computed from the GeoJSON bounds with a 10% padding. Use `viewbox` to manually specify `(x, y, width, height)` to zoom into a specific area. Use `viewbox_padding` to adjust the auto-computed padding.

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  text(size: 8pt)[
    ```typst
    #render-map(sweden, json.encode((
      stroke: "black",
      stroke_width: 0.02,
      fill: "grey",
      fill_opacity: 0.5,
      viewbox: array((15.0, -69.4, 10.0, 6.0)),
      point_color: "none",
    )), width: 70%)
    ```
  ],
  render-map(sweden, json.encode((
    stroke: "black",
    stroke_width: 0.02,
    fill: "grey",
    fill_opacity: 0.5,
    viewbox: array((15.0, -69.4, 10.0, 6.0)),
    point_color: "none",
  )), width: 70%),
)

#pagebreak()


== Labels

Set `label` to a string template with `{property_name}` placeholders.

```example
#render-map(sweden, json.encode((
    stroke: "white",
    stroke_width: 0.03,
    fill: "steelblue",
    fill_opacity: 0.8,
    point_color: "none",
    label: "{name}",
    label_color: "black",
    label_font_size: 0.25,
  )), width: 80%)
```

#pagebreak()

=== Multi-line labels

Pass an array of label line objects instead of a string. Each line can have its own `text`, `font_size`, `color`, and `font_family`.

```example
#render-map(sweden, json.encode((
    stroke: "white",
    stroke_width: 0.03,
    fill: "steelblue",
    fill_opacity: 0.8,
    point_color: "none",
    label: (
      (text: "{name}", font_size: 0.25, color: "black"),
      (text: "#{l_id}", font_size: 0.15, color: "red"),
    ),
  )), width: 80%)
```

_NB: GeoJSON `Feature.id` is automatically available as `{id}` in templates, even if it's not part of the feature's `properties`._

#pagebreak()

== Points

`Point` and `MultiPoint` geometries are rendered as circles. Use `point_radius` to control their size (defaults to `stroke_width × 5`).

```example
#render-map(sweden, json.encode((
    stroke: "black",
    stroke_width: 0.02,
    fill: "white",
    point_radius: 0.15,
    point_color: "red",
    label: "{point}",
    label_color: "black",
    label_font_size: 0.2,
  )), width: 80%)
```

#pagebreak()

== Per-feature styling

Use `{property_name}` in `stroke`, `fill`, or `fill_pattern` to resolve values from each feature's GeoJSON properties. If a feature is missing the referenced property, `fill` falls back to `"none"` and `stroke` falls back to `"black"`.

Use `fill_pattern` with `"hatched"`, `"crosshatched"`, or `"dotted"`. The pattern uses the `fill` color. Supports per-feature interpolation via `{property_name}` — features without the property get a solid fill.

```example
#render-map(sweden, json.encode((
    stroke: "black",
    stroke_width: 0.02,
    fill: "{fill_color}",
    fill_opacity: 0.9,
    fill_pattern: "{pattern}",
    point_color: "none",
  )), width: 70%)
```

#pagebreak()

== Projections

Several projections modes are supported, and a `graticule` (wireframe) is optionally available to visually see what the projections does.

```typst
#graticule: (
  step: 15,      // degrees between lines (default: 15)
  color: "red",  // line color (default: "#ccc")
  opacity: 0.5,  // line opacity (default: 0.6)
  width: 0.5,    // line width (default: 0.5)
)
```

The right-hand map of each projection above includes a graticule.


#let world_config = (
  stroke: "white",
  stroke_width: 0.05,
  fill: "steelblue",
  fill_opacity: 0.85,
)

#let conic_config = (
  ..world_config,
  stroke_width: 0.001,
)

#let graticule = (step: 15, color: "red", opacity: 0.5)

#{
  let projections = (
    (
      config: world_config,
      grat_config: (..world_config, graticule: graticule),
      name: "Equirectangular (default)",
      params: none,
      code: "#let world = read(\"data/worldmap.json\", encoding: \"utf8\")\n#render-map(world, json.encode((\n  graticule: (step: 15),\n)), width: 100%)",
    ),
    (
      config: (..world_config, projection: (type: "mercator")),
      grat_config: (..world_config, projection: (type: "mercator"), graticule: graticule),
      name: "Mercator",
      params: "`central_meridian` (default: 0)",
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"mercator\",\n    central_meridian: 0,\n  ),\n)), width: 100%)",
    ),
    (
      config: (..conic_config, projection: (
        type: "lambert_conformal_conic",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10,
      )),
      grat_config: (..conic_config, projection: (
        type: "lambert_conformal_conic",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10,
      ), graticule: graticule),
      name: "Lambert Conformal Conic",
      params: "`standard_parallel_1` (33), `standard_parallel_2` (45), `central_meridian` (0), `latitude_of_origin` (0)",
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"lambert_conformal_conic\",\n    standard_parallel_1: 30,\n    standard_parallel_2: 60,\n    central_meridian: 10,\n    latitude_of_origin: 0,\n  ),\n)), width: 100%)",
    ),
    (
      config: (..conic_config, projection: (
        type: "albers_equal_area",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10, latitude_of_origin: 40,
      )),
      grat_config: (..conic_config, projection: (
        type: "albers_equal_area",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10, latitude_of_origin: 40,
      ), graticule: graticule),
      name: "Albers Equal-Area",
      params: "`standard_parallel_1` (33), `standard_parallel_2` (45), `central_meridian` (0), `latitude_of_origin` (0)",
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"albers_equal_area\",\n    standard_parallel_1: 30,\n    standard_parallel_2: 60,\n    central_meridian: 10,\n    latitude_of_origin: 40,\n  ),\n)), width: 100%)",
    ),
    (
      config: (..conic_config, projection: (type: "robinson")),
      grat_config: (..conic_config, projection: (type: "robinson"), graticule: graticule),
      name: "Robinson",
      params: "`central_meridian` (default: 0)",
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"robinson\",\n    central_meridian: 0,\n  ),\n)), width: 100%)",
    ),
    (
      config: (..conic_config, projection: (
        type: "orthographic", center_lat: 45, center_lon: 10,
      )),
      grat_config: (..conic_config, projection: (
        type: "orthographic", center_lat: 45, center_lon: 10,
      ), graticule: graticule),
      name: "Orthographic",
      params: "`center_lat` (default: 0), `center_lon` (default: 0)",
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"orthographic\",\n    center_lat: 45,\n    center_lon: 10,\n  ),\n)), width: 100%)",
    ),
    (
      config: (..conic_config, projection: (type: "natural_earth")),
      grat_config: (..conic_config, projection: (type: "natural_earth"), graticule: graticule),
      name: "Natural Earth",
      params: "`central_meridian` (default: 0)",
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"natural_earth\",\n    central_meridian: 0,\n  ),\n)), width: 100%)",
    ),
  )

  for proj in projections {
    [=== #proj.name]
    grid(
      columns: (1fr, 1fr),
      gutter: 1em,
      render-map(world, json.encode(proj.config), width: 100%),
      render-map(world, json.encode(proj.grat_config), width: 100%),
    )
    if proj.code != none {
      raw(block: true, lang: "typst", proj.code)
    }
  }
}

== TopoJSON

Mercator automatically detects and handles TopoJSON input.

```example
#render-map(belgium, json.encode((
    stroke: "white",
    stroke_width: 0.003,
    fill: "red",
    fill_opacity: 1,
  )), width: 80%)
```

#pagebreak()

== Putting it all together

Combining projection, graticule, per-feature styling, fill patterns, and multi-line labels on a single map.

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  text(size: 7pt)[
    ```typst
    #render-map(sweden, json.encode((
      stroke: "white",
      stroke_width: 0.01,
      fill: "{fill_color}",
      fill_opacity: 0.8,
      fill_pattern: "{pattern}",
      label: (
        (text: "{name}", font_size: 0.18,
         color: "black",
         font_family: "New Computer Modern"),
        (text: "id: {l_id}", font_size: 0.12,
         color: "gray"),
      ),
      projection: (
        type: "mercator",
        central_meridian: 16,
      ),
      viewbox_padding: 0.02,
      graticule: (step: 2, color: "#ccc",
        opacity: 0.4, width: 0.3),
    )), width: 100%)
    ```
  ],
  render-map(sweden, json.encode((
    stroke: "white",
    stroke_width: 0.01,
    fill: "{fill_color}",
    fill_opacity: 0.8,
    fill_pattern: "{pattern}",
     point_radius: 0.15,
    point_color: "red",
    label: (
      (text: "{name}", font_size: 0.4, color: "black", font_family: "New Computer Modern"),
      (text: "id: {l_id}", font_size: 0.22, color: "red"),
      (text:"{point}", label_color: "blue", color: "green" ),
    ),
    projection: (
      type: "mercator",
      central_meridian: 16,
    ),
    viewbox_padding: 0.02,
    graticule: (step: 2, color: "blue", opacity: 0.2, width: 0.2),
  )), width: 100%),
)

#pagebreak()

== Inline GeoJSON with show rule

Use a Typst show rule to render inline GeoJSON code blocks as maps.

```typst
#show raw.where(lang: "geojson"): it => align(
  center, render-map(it.text, config, width: 40%)
)
```

#let inline_config = json.encode((
  stroke: "goldenrod",
  stroke_width: 0.08,
  fill: "gold",
  fill_opacity: 0.7,
))

#show raw.where(lang: "geojson"): it => align(center, render-map(it.text, inline_config, width: 40%))

```geojson
{"type":"GeometryCollection","geometries":[{"type":"Polygon","coordinates":[[[9.5,5.0],[9.46,5.59],[9.35,6.16],[9.16,6.72],[8.9,7.25],[8.57,7.74],[8.18,8.18],[7.74,8.57],[7.25,8.9],[6.72,9.16],[6.16,9.35],[5.59,9.46],[5.0,9.5],[4.41,9.46],[3.84,9.35],[3.28,9.16],[2.75,8.9],[2.26,8.57],[1.82,8.18],[1.43,7.74],[1.1,7.25],[0.84,6.72],[0.65,6.16],[0.54,5.59],[0.5,5.0],[0.54,4.41],[0.65,3.84],[0.84,3.28],[1.1,2.75],[1.43,2.26],[1.82,1.82],[2.26,1.43],[2.75,1.1],[3.28,0.84],[3.84,0.65],[4.41,0.54],[5.0,0.5],[5.59,0.54],[6.16,0.65],[6.72,0.84],[7.25,1.1],[7.74,1.43],[8.18,1.82],[8.57,2.26],[8.9,2.75],[9.16,3.28],[9.35,3.84],[9.46,4.41],[9.5,5.0]]]},{"type":"Polygon","coordinates":[[[3.85,6.2],[3.81,6.41],[3.69,6.59],[3.51,6.71],[3.3,6.75],[3.09,6.71],[2.91,6.59],[2.79,6.41],[2.75,6.2],[2.79,5.99],[2.91,5.81],[3.09,5.69],[3.3,5.65],[3.51,5.69],[3.69,5.81],[3.81,5.99],[3.85,6.2]]]},{"type":"Polygon","coordinates":[[[7.25,6.2],[7.21,6.41],[7.09,6.59],[6.91,6.71],[6.7,6.75],[6.49,6.71],[6.31,6.59],[6.19,6.41],[6.15,6.2],[6.19,5.99],[6.31,5.81],[6.49,5.69],[6.7,5.65],[6.91,5.69],[7.09,5.81],[7.21,5.99],[7.25,6.2]]]},{"type":"LineString","coordinates":[[2.86,3.4],[3.06,3.18],[3.3,2.98],[3.55,2.81],[3.82,2.66],[4.1,2.55],[4.39,2.47],[4.7,2.42],[5.0,2.4],[5.3,2.42],[5.61,2.47],[5.9,2.55],[6.18,2.66],[6.45,2.81],[6.7,2.98],[6.94,3.18],[7.14,3.4]]}]}
```
