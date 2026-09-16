#import "../mercator/mercator.typ": *

#set page(
  width: 15cm, height: 21cm, margin: 1cm,
  numbering: "1",
  footer: context {
    let page-num = counter(page).get().first()
    if page-num > 1 {
      grid(
        columns: (1fr, 1fr),
        align(left, text(size: 7.5pt, fill: luma(120))[Mercator Documentation]),
        align(right, text(size: 7.5pt, fill: luma(120))[#page-num]),
      )
    }
  },
)
#set text(font: "New Computer Modern")
#set par(justify: true)
#show heading.where(level: 3): set text(size: 1.17em)
#show heading.where(level: 4): set text(size: 1.05em)

// --- Data ---

#let sweden = read("data/swedish_regions.json", encoding: none)
#let world = read("data/world.json", encoding: none)
#let world_no_ant = read("data/world_no_antartica.json", encoding: none)
#let cities = read("data/cities.geojson", encoding: none)
#let points = read("data/points.geojson", encoding: none)

// --- Example helper ---
// Show rule that displays a code block and executes it.

#let doc-scope = (
  "render-map": render-map, "render-layers": render-layers,
  "geo-arc": geo-arc, "geo-circle": geo-circle,
  "join": join, "map-centroids": map-centroids, "map-bounds": map-bounds,
  sweden: sweden, world: world, world_no_ant: world_no_ant,
  cities: cities, points: points,
)
#let code-block(body) = block(
  width: 100%, inset: 8pt, radius: 3pt,
  fill: luma(245), stroke: 0.5pt + luma(200), body,
)
#show raw.where(lang: "example"): it => {
  code-block(text(size: 7pt, raw(block: true, lang: "typst", it.text)))
  align(center, eval(it.text, mode: "markup", scope: doc-scope))
}

// --- Hero ---

#let hero_data = world
#let hero_config = (
  stroke: "white",
  stroke_width: 0.001,
  fill: "steelblue",
  fill_opacity: 0.85,
  projection: (type: "orthographic", center_lat: 45, center_lon: 10),
  graticule: (step: 15, color: "red", opacity: 0.5),
)

#set page(numbering: none) // no number on cover
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
#set page(numbering: "1")
#counter(page).update(1)

#text(size: 9pt)[#outline(indent: 1em)]

#pagebreak()

== Quick start

```example
#import "../mercator/mercator.typ": *

#let sweden = read("data/swedish_regions.json", encoding: none)
#render-map(sweden, width: 80%)
```

The `render-map` function takes two positional arguments:
- `data` — GeoJSON or TopoJSON string
- `config` — configuration dictionary (optional)

All additional named arguments (`width`, `height`, `fit`, `alt`) are forwarded to Typst's built-in `image` function.

If no config is passed the maps will default to this visually.

#pagebreak()

== Inline GeoJSON with show rule

Use a Typst show rule to render inline GeoJSON code blocks as maps. 

Any #raw("```geojson```") code block will automatically be rendered as an image.

#code-block(text(size: 7pt, raw(block: true, lang: "typst", "#show raw.where(lang: \"geojson\"): it => align(\n  center, render-map(it.text, width: 40%)\n)")))

#let smiley = `{"type":"GeometryCollection","geometries":[{"type":"Polygon","coordinates":[[[9.5,5.0],[9.46,5.59],[9.35,6.16],[9.16,6.72],[8.9,7.25],[8.57,7.74],[8.18,8.18],[7.74,8.57],[7.25,8.9],[6.72,9.16],[6.16,9.35],[5.59,9.46],[5.0,9.5],[4.41,9.46],[3.84,9.35],[3.28,9.16],[2.75,8.9],[2.26,8.57],[1.82,8.18],[1.43,7.74],[1.1,7.25],[0.84,6.72],[0.65,6.16],[0.54,5.59],[0.5,5.0],[0.54,4.41],[0.65,3.84],[0.84,3.28],[1.1,2.75],[1.43,2.26],[1.82,1.82],[2.26,1.43],[2.75,1.1],[3.28,0.84],[3.84,0.65],[4.41,0.54],[5.0,0.5],[5.59,0.54],[6.16,0.65],[6.72,0.84],[7.25,1.1],[7.74,1.43],[8.18,1.82],[8.57,2.26],[8.9,2.75],[9.16,3.28],[9.35,3.84],[9.46,4.41],[9.5,5.0]]]},{"type":"Polygon","coordinates":[[[3.85,6.2],[3.81,6.41],[3.69,6.59],[3.51,6.71],[3.3,6.75],[3.09,6.71],[2.91,6.59],[2.79,6.41],[2.75,6.2],[2.79,5.99],[2.91,5.81],[3.09,5.69],[3.3,5.65],[3.51,5.69],[3.69,5.81],[3.81,5.99],[3.85,6.2]]]},{"type":"Polygon","coordinates":[[[7.25,6.2],[7.21,6.41],[7.09,6.59],[6.91,6.71],[6.7,6.75],[6.49,6.71],[6.31,6.59],[6.19,6.41],[6.15,6.2],[6.19,5.99],[6.31,5.81],[6.49,5.69],[6.7,5.65],[6.91,5.69],[7.09,5.81],[7.21,5.99],[7.25,6.2]]]},{"type":"LineString","coordinates":[[2.86,3.4],[3.06,3.18],[3.3,2.98],[3.55,2.81],[3.82,2.66],[4.1,2.55],[4.39,2.47],[4.7,2.42],[5.0,2.4],[5.3,2.42],[5.61,2.47],[5.9,2.55],[6.18,2.66],[6.45,2.81],[6.7,2.98],[6.94,3.18],[7.14,3.4]]}]}`

Then, this `GeoJSON` code block renders as a follows:

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  text(size: 5pt, raw(block: true, lang: "json", "```geojson\n" + smiley.text + "\n```")),
  align(center + horizon, render-map(smiley.text, width: 80%)),
)

== GeoJSON types handling

Mercator handles all standard GeoJSON geometry types:

- `Point` / `MultiPoint` — rendered as circles (radius controlled by `point_radius`)
- `LineString` / `MultiLineString` — rendered as stroked paths
- `Polygon` / `MultiPolygon` — rendered as filled and stroked shapes
- `GeometryCollection` — all contained geometries are rendered

#pagebreak()

== Configuration overview

All rendering options are passed as a Typst dictionary. Every field is optional and defaults to the value shown below. Fields marked with `{property}` support per-feature interpolation from GeoJSON properties. (cf. per-feature styling section)

#code-block(text(size: 7pt, raw(block: true, lang: "json", ```
{
  // --- Appearance --- ({property} = per-feature interpolation)
  "stroke":       "black",   // color; "none" to hide. {property}
  "stroke_width": 0.05,      // float, or "{property}" template
  "fill":         "white",   // color; "none". {property}
  "fill_opacity": 1.0,       // 0..1, or "{property}" template
  "fill_pattern": null,      // "hatched"|"crosshatched"|"dotted". {property}
  "stroke_dash":  null,      // SVG dash pattern, e.g. "0.25 0.15"
  "point_radius": null,      // float – defaults to stroke_width × 5
  "point_color":  null,      // color – defaults to fill; "none" hides points
  "point_shape":  "circle",  // circle|square|diamond|triangle|cross|star

  // --- Labels ---
  "label":             null,    // "{name}", a line object, or an array of them
  "label_color":       "black",
  "label_font_size":   0.3,
  "label_font_family": "Arial",
  "label_halo":        null,    // halo color drawn behind the text
  "label_halo_width":  0.1,     // halo thickness
  "label_collide":     false,   // drop overlapping labels

  // --- Data-driven / thematic ---
  "fill_scale":         null,   // choropleth (see Choropleth)
  "legend":             null,   // color-scale legend (see Choropleth)
  "point_radius_scale": null,   // proportional symbols (see Proportional symbols)
  "size_legend":        null,   // nested-circle size key (see Size legend)
  "filter":             null,   // render only matching features (see Filtering)
  "fit":                null,   // frame the view to a subset (see Fit to a region)

  // --- Aggregation ---
  "hexbin":  null,   // hexagonal binning (see Density)
  "contour": null,   // density isolines (see Density)
  "dorling": null,   // Dorling cartogram (see Cartograms)

  // --- Geometry / framing ---
  "viewbox":         null,      // [x, y, width, height] – auto if null
  "viewbox_padding": 0.15,      // padding fraction around the auto viewbox
  "precision":       null,      // adaptive resampling step (none = off)
  "clip_extent":     null,      // [lon0, lat0, lon1, lat1] rectangular clip
  "clip_angle":      null,      // azimuthal small-circle clip, degrees

  // --- Projection & overlays ---
  "projection":   null,         // object (see Projections)
  "rotate":       null,         // [lambda, phi, gamma] deg (non-azimuthal)
  "graticule":    null,         // object (see Graticule)
  "sphere":       null,         // ocean disc / frame (see Sphere)
  "antimeridian": false,        // cut & re-stitch at ±180° (see Antimeridian)
  "tissot":       null          // object (see Tissot's Indicatrix)
}
```.text)))

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
    #render-map(sweden, (
      stroke: "white",
      stroke_width: 0.01,
      fill: "teal",
      fill_opacity: 0.8,
      point_color: "none",
    ), width: 100%)
    ```
  ],
  [
    ```example
    #render-map(sweden, (
      stroke: "#333",
      stroke_width: 0.08,
      fill: "#f7fc0f",
      fill_opacity: 0.2,
      point_color: "none",
    ), width: 100%)
    ```
  ],
)

== Viewbox

By default, the viewbox is auto-computed from the GeoJSON bounds with a 10% padding. Use `viewbox` to manually specify `(x, y, width, height)` to zoom into a specific area. Use `viewbox_padding` to adjust the auto-computed padding.

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  code-block(text(size: 7pt, raw(block: true, lang: "typst", "#render-map(sweden, (\n  stroke: \"black\",\n  stroke_width: 0.02,\n  fill: \"grey\",\n  fill_opacity: 0.5,\n  viewbox: array((15.0, -69.4, 10.0, 6.0)),\n  point_color: \"none\",\n), width: 70%)"))),
  align(center + horizon, render-map(sweden, (
    stroke: "black",
    stroke_width: 0.02,
    fill: "grey",
    fill_opacity: 0.5,
    viewbox: array((15.0, -69.4, 10.0, 6.0)),
    point_color: "none",
  ), width: 70%)),
)

#pagebreak()


== Labels

Set `label` to a string template with `{property_name}` placeholders.

```example
#render-map(sweden, (
    stroke: "white",
    stroke_width: 0.03,
    fill: "steelblue",
    fill_opacity: 0.8,
    point_color: "none",
    label: "{name}",
    label_color: "black",
    label_font_size: 0.25,
  ), width: 80%)
```

#pagebreak()

=== Multi-line labels

Pass an array of label line objects instead of a string. Each line can have its own `text`, `font_size`, `color`, and `font_family`.

```example
#render-map(sweden, (
    stroke: "white",
    stroke_width: 0.03,
    fill: "steelblue",
    fill_opacity: 0.8,
    point_color: "none",
    label: (
      (text: "{name}", font_size: 0.25, color: "black"),
      (text: "#{l_id}", font_size: 0.15, color: "red"),
    ),
  ), width: 80%)
```

_NB: GeoJSON `Feature.id` is automatically available as `{id}` in templates, even if it's not part of the feature's `properties`._

#pagebreak()

== Points

`Point` and `MultiPoint` geometries are rendered as circles. Use `point_radius` to control their size (defaults to `stroke_width × 5`).

```example
#render-map(sweden, (
    stroke: "black",
    stroke_width: 0.02,
    fill: "white",
    point_radius: 0.15,
    point_color: "red",
    label: "{point}",
    label_color: "red",
    label_font_size: 0.6,
  ), width: 80%)
```

#pagebreak()

== Per-feature styling

Use `{property_name}` in `stroke`, `fill`, or `fill_pattern` to resolve values from each feature's GeoJSON properties. If a feature is missing the referenced property, `fill` falls back to `"none"` and `stroke` falls back to `"black"`.

Use `fill_pattern` with `"hatched"`, `"crosshatched"`, or `"dotted"`. The pattern uses the `fill` color. Supports per-feature interpolation via `{property_name}` — features without the property get a solid fill.

```example
#render-map(sweden, (
    stroke: "black",
    stroke_width: 0.02,
    fill: "{fill_color}",
    fill_opacity: 0.9,
    fill_pattern: "{pattern}",
    point_color: "none",
  ), width: 80%)
```

#pagebreak()

== Choropleth

A *choropleth* colors each feature from a numeric property. Set `fill_scale` with the `property` to read, a scale `type`, and a `range` of colors; it overrides `fill` per feature. Use `type: "quantize"` for discrete bins or `type: "linear"` to interpolate between hex colors. The `domain` (min, max) is computed from the data when omitted, and `default` colors features whose property is missing.

Add a `legend` to draw a swatch key in any corner (`pos`): `"top-left"`, `"top-right"`, `"bottom-left"`, or `"bottom-right"`.

```example
#render-map(sweden, (
    fill_scale: (
      property: "color",
      type: "quantize",
      range: ("#fee5d9", "#fcae91", "#fb6a4a",
              "#de2d26", "#a50f15"),
    ),
    stroke: "white",
    stroke_width: 0.02,
    point_color: "none",
    legend: (title: "color", pos: "bottom-left"),
  ), width: 80%)
```

With `type: "linear"` the `range` becomes gradient stops, interpolated across an explicit `domain`:

```example
#render-map(sweden, (
    fill_scale: (
      property: "color",
      type: "linear",
      domain: (0, 4),
      range: ("#ffffcc", "#006837"),
    ),
    stroke: "white",
    stroke_width: 0.02,
    point_color: "none",
    legend: (title: "color", pos: "bottom-left"),
  ), width: 80%)
```

=== Categorical coloring

For non-numeric data, `type: "category"` maps each distinct value to a color — for political, land-use, or typological maps. Give an explicit `categories` map (`value → color`), or a `palette` that is auto-assigned to the distinct values in first-seen order. The property may be a string or a number; unmatched features take `default`.

```example
#render-map(sweden, (
    fill_scale: (property: "color", type: "category",
      categories: (
        "0": "#e41a1c", "1": "#377eb8",
        "2": "#4daf4a", "3": "#984ea3", "4": "#ff7f00"),
    ),
    stroke: "white",
    stroke_width: 0.02,
    point_color: "none",
    legend: (title: "class", pos: "bottom-left"),
  ), width: 80%)
```

=== Joining external data

A `fill_scale` reads a numeric property that must be present on each feature. When your data lives in a separate table (a CSV, a database export) rather than inside the GeoJSON, pass it to `render-map` as `data` and it is merged into each feature's properties, matched on `key`. `data` is either a dictionary keyed by the join value, or an array of records (e.g. from `csv(.., row-type: dictionary)`); joined values override existing properties. The standalone `join(code, data, key: ..)` function returns the merged GeoJSON if you prefer to do it explicitly.

Here `rate` is *not* in the GeoJSON — it is joined from an external dictionary keyed by each region's `l_id`. Regions with no matching row take the scale's `default` color.

```example
#let rates = ("1": (rate: 90), "4": (rate: 55),
  "5": (rate: 30), "6": (rate: 70), "21": (rate: 45))

#render-map(sweden, (
    fill_scale: (property: "rate", type: "quantize",
      domain: (0, 100),
      range: ("#f7fbff", "#c6dbef", "#6baed6",
              "#2171b5", "#08306b")),
    stroke: "white",
    stroke_width: 0.02,
    point_color: "none",
    legend: (title: "rate", pos: "bottom-left"),
  ), data: rates, key: "l_id", width: 80%)
```

#pagebreak()

== Proportional symbols

`point_radius_scale` sizes `Point` and `MultiPoint` symbols from a numeric property — a proportional-symbol (bubble) map. By default it uses `type: "sqrt"`, so symbol _area_ tracks the value (the perceptually correct encoding); use `type: "linear"` to scale the radius directly. Set `max_radius` for the largest value; `min_radius` (default 0) keeps the smallest symbols visible. The `domain` is computed from the data when omitted.

Symbols are usually drawn over a *basemap* for geographic context. `render-map` takes one GeoJSON, so combine the two sources into a single `FeatureCollection` — the polygon features become the base and the point features the symbols. Here the Swedish regions form a light-grey base and each city is a `pop`-proportional bubble:

```example
#let base = json(bytes(sweden)).features
#let pts = json(bytes(cities)).features
#let combined = json.encode(
  (type: "FeatureCollection", features: base + pts))

#render-map(combined, (
    projection: (type: "mercator"),
    fill: "#e8e8e8", stroke: "white", stroke_width: 0.02,
    point_radius_scale: (
      property: "pop",
      max_radius: 1.2,
      min_radius: 0.1,
    ),
    point_color: "crimson",
  ), width: 55%)
```

=== Bivariate: size and color together

`point_radius_scale` and `fill_scale` compose. Because `point_color` defaults to `fill`, a `fill_scale` colors the symbols too — so each city can encode two variables at once: *size* by population and *color* by growth rate (a diverging blue–white–red scale). The scale's `default` colors the basemap regions (which have no `growth`), and a `legend` keys the color.

```example
#let base = json(bytes(sweden)).features
#let pts = json(bytes(cities)).features
#let combined = json.encode(
  (type: "FeatureCollection", features: base + pts))

#render-map(combined, (
    projection: (type: "mercator"),
    point_radius_scale: (property: "pop",
      max_radius: 1.4, min_radius: 0.15),
    fill_scale: (property: "growth", type: "linear",
      domain: (0, 2.5),
      range: ("#2166ac", "#f7f7f7", "#b2182b"),
      default: "#e8e8e8"),
    stroke: "white", stroke_width: 0.03,
    legend: (title: "growth %", pos: "top-right"),
  ), width: 55%)
```

#pagebreak()

== Projections

A map projection transforms coordinates from the curved surface of the Earth onto a flat plane. Every projection introduces some distortion — it is mathematically impossible to flatten a sphere without stretching, compressing, or tearing it somewhere. Projections differ in _what_ they preserve: shape (conformal), area (equal-area), or neither (compromise).

Mercator supports several projection families. Two overlay tools help visualize how each projection behaves:

=== Graticule

A *graticule* draws a grid of meridians (longitude) and parallels (latitude) on the map, making the projection's geometry immediately visible.

#let ortho_base = (
  stroke: "white", stroke_width: 0.001, fill: "steelblue", fill_opacity: 0.85,
  projection: (type: "orthographic", center_lat: 45, center_lon: 10),
)
#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  [
    #render-map(world, ortho_base, width: 100%)
    #align(center, text(size: 8pt)[Without graticule])
  ],
  [
    #render-map(world, (..ortho_base,
      graticule: (step: 15, color: "red", opacity: 0.5),
    ), width: 100%)
    #align(center, text(size: 8pt)[With graticule])
  ],
)

#code-block(text(size: 7pt, raw(block: true, lang: "typst", "graticule: (\n  step: 15,      // degrees between lines (default: 15)\n  color: \"red\",  // line color (default: \"#ccc\")\n  opacity: 0.5,  // line opacity (default: 0.6)\n  width: 0.5,    // line width (default: 0.5)\n)")))

The following examples of projections will use a graticule to visually represent the distortions due to the projections.

#pagebreak()

=== Tissot's Indicatrix

#link("https://en.wikipedia.org/wiki/Tissot%27s_indicatrix")[Tissot's indicatrix] places small circles at regular grid points. After projection, these circles deform into ellipses that reveal how the projection distorts shapes and areas.

On a *conformal* projection (like Mercator), circles stay circular but grow near the poles. On an *equal-area* projection (like Albers), circles keep the same area but get squished. On a *compromise* projection (like Robinson), both shape and area change.

#align(center, render-map(world, (
  projection: (type: "mercator"),
  stroke: "#aaa", stroke_width: 0.01, fill: "none",
  graticule: (step: 30, color: "#ddd", opacity: 0.4, width: 0.2),
  tissot: (step: 30, radius: 5, fill: "red", fill_opacity: 0.4, stroke: "darkred", stroke_width: 0.3),
), width: 80%))

#code-block(text(size: 7pt, raw(block: true, lang: "typst", "tissot: (\n  step: 30,          // degrees between circles (default: 30)\n  radius: 5,         // circle radius in degrees (default: 5)\n  fill: \"red\",       // fill color (default: \"red\")\n  fill_opacity: 0.3, // fill opacity (default: 0.3)\n  stroke: \"red\",     // stroke color (default: \"red\")\n  stroke_width: 0.5, // stroke width (default: 0.5)\n  max_lat: 60,       // maximum latitude in degrees (default: 60)\n)")))

=== Sphere and globe clipping

Azimuthal globe projections (`orthographic`) *always* clip geometry to the visible hemisphere — land is cut at the limb and re-stitched along it, so coastlines close cleanly against the horizon instead of breaking, with no configuration needed:

```example
#render-map(world, (
    projection: (type: "orthographic",
      center_lat: 25, center_lon: 10),
    fill: "#6fbf5f", stroke: "#356b2c",
    stroke_width: 0.004,
  ), width: 60%)
```

The optional `sphere` config adds a filled ocean disc behind the land: `fill` colors it (`"none"` for no ocean), and `stroke`/`stroke_width` outline it.

```example
#render-map(world, (
    projection: (type: "orthographic",
      center_lat: 25, center_lon: 10),
    sphere: (fill: "#a8d4f0", stroke: "#268",
      stroke_width: 0.005),
    fill: "#6fbf5f", stroke: "#356b2c",
    stroke_width: 0.004,
    graticule: (step: 15, color: "#ffffff",
      opacity: 0.5),
  ), width: 60%)
```

=== Antimeridian clipping

Cylindrical projections have a seam at the ±180° antimeridian. Polygons that cross it — most visibly Antarctica, which also wraps the south pole — otherwise break apart. `antimeridian: true` cuts crossing polygons and re-stitches them along the map boundary (around the pole where needed), so they fill correctly. It uses the d3-geo clip/rejoin algorithm.

```example
#render-map(world, (
    projection: (type: "equirectangular"),
    antimeridian: true,
    fill: "#6fbf5f",
    stroke: "#356b2c",
    stroke_width: 0.04,
  ), width: 78%)
```

#pagebreak()

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
#let tissot = (step: 30, radius: 5, fill: "red", fill_opacity: 0.3, stroke: "darkred", stroke_width: 0.3, max_lat: 80)

#{
  let projections = (
    // --- Cylindrical ---
    (
      category: "Cylindrical",
      config: world_config,
      grat_config: (..world_config, graticule: graticule),
      name: "Equirectangular (default)",
      url: "https://en.wikipedia.org/wiki/Equirectangular_projection",
      desc: "The simplest projection: longitude and latitude map directly to x and y. Attributed to Marinus of Tyre (c. 100 AD). Neither conformal nor equal-area, but trivial to compute and widely used as a baseline.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#let world = read(\"data/world.json\", encoding: none)\n#render-map(world, (\n  graticule: (step: 15),\n))",
    ),
    (
      category: "Cylindrical",
      config: (..world_config, projection: (type: "mercator")),
      grat_config: (..world_config, projection: (type: "mercator"), graticule: graticule),
      name: "Mercator",
      url: "https://en.wikipedia.org/wiki/Mercator_projection",
      desc: "Introduced by Gerardus Mercator in 1569 for nautical navigation. Conformal: preserves local angles and shapes, so any straight line is a constant-bearing rhumb line. Extreme area distortion near the poles.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, (\n  projection: (\n    type: \"mercator\",\n    central_meridian: 0,\n  ),\n))",
    ),
    (
      category: "Cylindrical",
      render_width: 55%,
      config: (..conic_config, projection: (type: "cassini", central_meridian: 0)),
      grat_config: (..conic_config, projection: (type: "cassini", central_meridian: 0), graticule: graticule),
      name: "Cassini",
      url: "https://en.wikipedia.org/wiki/Cassini_projection",
      desc: "Developed by C\u{e9}sar-Fran\u{e7}ois Cassini de Thury in 1745 for the triangulation of France. A transverse equirectangular projection: the central meridian plays the role of the equator. Useful for mapping narrow north-south strips.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, (\n  projection: (\n    type: \"cassini\",\n    central_meridian: 0,\n  ),\n))",
    ),
    // --- Conic ---
    (
      category: "Conic",
      data: world_no_ant,
      config: (..conic_config, projection: (
        type: "lambert_conformal_conic",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10,
      )),
      grat_config: (..conic_config, projection: (
        type: "lambert_conformal_conic",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10,
      ), graticule: graticule),
      name: "Lambert Conformal Conic",
      url: "https://en.wikipedia.org/wiki/Lambert_conformal_conic_projection",
      desc: "Proposed by Johann Heinrich Lambert in 1772. Conformal: preserves local shapes. Meridians are straight lines radiating from a pole, parallels are concentric arcs. Standard for aeronautical charts and many national mapping systems.",
      params: (
        (name: "standard_parallel_1", typ: "float", default: "33"),
        (name: "standard_parallel_2", typ: "float", default: "45"),
        (name: "central_meridian", typ: "float", default: "0"),
        (name: "latitude_of_origin", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"lambert_conformal_conic\",\n    standard_parallel_1: 30,\n    standard_parallel_2: 60,\n    central_meridian: 10,\n    latitude_of_origin: 0,\n  ),\n))",
    ),
    (
      category: "Conic",
      data: world_no_ant,
      config: (..conic_config, projection: (
        type: "albers_equal_area",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10, latitude_of_origin: 40,
      )),
      grat_config: (..conic_config, projection: (
        type: "albers_equal_area",
        standard_parallel_1: 30, standard_parallel_2: 60, central_meridian: 10, latitude_of_origin: 40,
      ), graticule: graticule),
      name: "Albers Equal-Area",
      url: "https://en.wikipedia.org/wiki/Albers_projection",
      desc: "Introduced by Heinrich C. Albers in 1805. Equal-area: faithfully represents relative sizes of regions. Parallels are concentric arcs, meridians are straight lines. Used by the USGS for maps of the contiguous United States.",
      params: (
        (name: "standard_parallel_1", typ: "float", default: "33"),
        (name: "standard_parallel_2", typ: "float", default: "45"),
        (name: "central_meridian", typ: "float", default: "0"),
        (name: "latitude_of_origin", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"albers_equal_area\",\n    standard_parallel_1: 30,\n    standard_parallel_2: 60,\n    central_meridian: 10,\n    latitude_of_origin: 40,\n  ),\n))",
    ),
    // --- Pseudo-conic ---
    (
      category: "Pseudo-conic",
      data: world,
      render_width: 75%,
      config: (..conic_config, viewbox: (-2.5, -2.5, 5, 5.5), projection: (
        type: "bonne", standard_parallel: 45, central_meridian: 10,
      )),
      grat_config: (..conic_config, viewbox: (-2.5, -2.5, 5, 5.5), projection: (
        type: "bonne", standard_parallel: 45, central_meridian: 10,
      ), graticule: graticule),
      name: "Bonne",
      url: "https://en.wikipedia.org/wiki/Bonne_projection",
      desc: "Named after Rigobert Bonne (1727--1795), though used much earlier. Equal-area and pseudoconic: parallels are concentric arcs (as in conic projections) but meridians are curved, giving the map its characteristic heart shape. Widely used for atlas maps of continents in the 19th century.",
      params: (
        (name: "standard_parallel", typ: "float", default: "45"),
        (name: "central_meridian", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"bonne\",\n    standard_parallel: 45,\n    central_meridian: 10,\n  ),\n))",
    ),
    (
      category: "Pseudo-conic",
      data: world_no_ant,
      config: (..conic_config, viewbox: (-4, -3.5, 8, 7), projection: (
        type: "polyconic", central_meridian: 10,
      )),
      grat_config: (..conic_config, viewbox: (-4, -3.5, 8, 7), projection: (
        type: "polyconic", central_meridian: 10,
      ), graticule: graticule),
      name: "American Polyconic",
      url: "https://en.wikipedia.org/wiki/American_polyconic_projection",
      desc: "Devised by Ferdinand Hassler around 1820 for the U.S. Coast Survey. Neither conformal nor equal-area, but distortion is low near the central meridian. Each parallel is a circular arc of true scale, but unlike conic projections they are not concentric --- hence \"polyconic\". Was the standard projection for USGS topographic maps until the 1950s.",
      params: (
        (name: "central_meridian", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"polyconic\",\n    central_meridian: 10,\n  ),\n))",
    ),
    // --- Pseudo-cylindrical ---
    (
      category: "Pseudo-cylindrical",
      config: (..conic_config, projection: (type: "robinson")),
      grat_config: (..conic_config, projection: (type: "robinson"), graticule: graticule),
      name: "Robinson",
      url: "https://en.wikipedia.org/wiki/Robinson_projection",
      desc: "Created by Arthur H. Robinson in 1963 for Rand McNally. A compromise projection: neither conformal nor equal-area, but visually pleasing with moderate distortion everywhere. Used by National Geographic from 1988 to 1998.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, (\n  projection: (\n    type: \"robinson\",\n    central_meridian: 0,\n  ),\n))",
    ),
    (
      category: "Pseudo-cylindrical",
      config: (..conic_config, projection: (type: "natural_earth")),
      grat_config: (..conic_config, projection: (type: "natural_earth"), graticule: graticule),
      name: "Natural Earth",
      url: "https://en.wikipedia.org/wiki/Natural_Earth_projection",
      desc: "Designed by Tom Patterson in 2008 for the Natural Earth dataset. A compromise projection similar to Robinson but with smoother, rounder corners. Adopted by many open-source mapping tools as a default world view.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, (\n  projection: (\n    type: \"natural_earth\",\n    central_meridian: 0,\n  ),\n))",
    ),
    (
      category: "Pseudo-cylindrical",
      config: (..conic_config, projection: (type: "hammer")),
      grat_config: (..conic_config, projection: (type: "hammer"), graticule: graticule),
      name: "Hammer",
      url: "https://en.wikipedia.org/wiki/Hammer_projection",
      desc: "Developed by Ernst Hammer in 1892 as a modification of the Aitoff projection. Equal-area: maps the entire globe into an ellipse with a 2:1 axis ratio. Meridians are curved, equally spaced along the equator. Widely used for whole-world maps in atlases.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, (\n  projection: (\n    type: \"hammer\",\n    central_meridian: 0,\n  ),\n))",
    ),
    (
      category: "Pseudo-cylindrical",
      config: (..conic_config, projection: (type: "winkel_tripel")),
      grat_config: (..conic_config, projection: (type: "winkel_tripel"), graticule: graticule),
      name: "Winkel Tripel",
      url: "https://en.wikipedia.org/wiki/Winkel_tripel_projection",
      desc: "Created by Oswald Winkel in 1921. A compromise projection computed as the arithmetic mean of equirectangular and Aitoff projections. Minimizes the sum of distortions in area, direction, and distance --- hence \"tripel\" (German for triple). Adopted by the National Geographic Society in 1998 as their standard world map projection.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, (\n  projection: (\n    type: \"winkel_tripel\",\n    central_meridian: 0,\n  ),\n))",
    ),
    // --- Azimuthal ---
    (
      category: "Azimuthal",
      render_width: 80%,
      config: (..conic_config, viewbox_padding: 0.25, projection: (
        type: "lambert_azimuthal_equal_area", center_lat: 45, center_lon: 10,
      )),
      grat_config: (..conic_config, viewbox_padding: 0.25, projection: (
        type: "lambert_azimuthal_equal_area", center_lat: 45, center_lon: 10,
      ), graticule: graticule),
      name: "Lambert Azimuthal Equal-Area",
      url: "https://en.wikipedia.org/wiki/Lambert_azimuthal_equal-area_projection",
      desc: "Another Lambert contribution (1772). Equal-area: areas are preserved across the entire map. Projects the globe onto a tangent plane. Commonly used for continental and hemispheric maps where accurate area representation matters.",
      params: (
        (name: "center_lat", typ: "float", default: "0"),
        (name: "center_lon", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"lambert_azimuthal_equal_area\",\n    center_lat: 45,\n    center_lon: 10,\n  ),\n))",
    ),
    (
      category: "Azimuthal",
      data: world_no_ant,
      render_width: 75%,
      config: (..conic_config, projection: (
        type: "gnomonic", center_lat: 90, center_lon: 0,
      ), viewbox: (-3, -3, 6, 6)),
      grat_config: (..conic_config, projection: (
        type: "gnomonic", center_lat: 90, center_lon: 0,
      ), graticule: graticule, viewbox: (-3, -3, 6, 6)),
      name: "Gnomonic",
      url: "https://en.wikipedia.org/wiki/Gnomonic_projection",
      desc: "Known since antiquity, attributed to Thales (c. 580 BC). The only projection where all great circles appear as straight lines, making it invaluable for plotting shortest-distance routes. Can only show less than a hemisphere.",
      params: (
        (name: "center_lat", typ: "float", default: "0"),
        (name: "center_lon", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"gnomonic\",\n    center_lat: 45,\n    center_lon: 10,\n  ),\n))",
    ),
    (
      category: "Azimuthal",
      config: (..conic_config, projection: (
        type: "orthographic", center_lat: 45, center_lon: 10,
      )),
      grat_config: (..conic_config, projection: (
        type: "orthographic", center_lat: 45, center_lon: 10,
      ), graticule: graticule),
      name: "Orthographic",
      url: "https://en.wikipedia.org/wiki/Orthographic_map_projection",
      desc: "Used by the ancient Greeks and formalized by Hipparchus (c. 150 BC). Simulates viewing the Earth from infinite distance, giving a natural globe-like appearance. Neither conformal nor equal-area.",
      params: (
        (name: "center_lat", typ: "float", default: "0"),
        (name: "center_lon", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"orthographic\",\n    center_lat: 45,\n    center_lon: 10)))",
    ),
    (
      category: "Azimuthal",
      render_width: 90%,
      config: (..conic_config, projection: (
        type: "azimuthal_equidistant", center_lat: -90, center_lon: 0,
      )),
      grat_config: (..conic_config, projection: (
        type: "azimuthal_equidistant", center_lat: -90, center_lon: 0,
      ), graticule: graticule),
      name: "Azimuthal Equidistant",
      url: "https://en.wikipedia.org/wiki/Azimuthal_equidistant_projection",
      desc: "Distances from the center point are preserved in all directions. Can display the entire globe, with the antipodal point mapped to a bounding circle. Used for the UN emblem (centered on the North Pole) and for radio and seismic distance analysis.",
      params: (
        (name: "center_lat", typ: "float", default: "0"),
        (name: "center_lon", typ: "float", default: "0"),
      ),
      code: "#render-map(world, (\n  projection: (\n    type: \"azimuthal_equidistant\",\n    center_lat: 45,\n    center_lon: 10,\n  ),\n))",
    ),
  )

  let categories = projections.map(p => p.category).dedup()
  for cat in categories {
    pagebreak()
    [=== #cat]
    let cat_projs = projections.filter(p => p.category == cat)
    for (i, proj) in cat_projs.enumerate() {
      let geo_data = if "data" in proj { proj.data } else { world }
      let w = if "render_width" in proj { proj.render_width } else { 100% }
      if i > 0 { pagebreak() }
      [==== #link(proj.url)[#proj.name]]
      text(size: 8.5pt, style: "italic")[#proj.desc]
      align(center, render-map(geo_data, proj.grat_config, width: w))
      if proj.params != none {
        text(size: 8pt, weight: "bold")[Parameters]
        for p in proj.params {
          [- #text(size: 8pt)[#raw(p.name) _(#p.typ, default: #p.default)_]]
        }
      }
      if proj.code != none {
        block(
          width: 100%,
          inset: 8pt,
          radius: 3pt,
          fill: luma(245),
          stroke: 0.5pt + luma(200),
          text(size: 7pt, raw(block: true, lang: "typst", proj.code))
        )
      }
    }
  }

  pagebreak()
  [=== Pseudo-azimuthal]
  [==== #link("https://en.wikipedia.org/wiki/Wiechel_projection")[Wiechel]]
  text(size: 8.5pt, style: "italic")[Invented by H. Wiechel in 1879. An equal-area azimuthal projection with a distinctive swirl: each meridian is a circular arc, giving the map a pinwheel-like appearance. One of the few projections that is both equal-area and visually striking.]
  align(center, render-map(world, (..conic_config, projection: (
    type: "wiechel", center_lat: 90, center_lon: 0,
  ), graticule: graticule), width: 92%))
  text(size: 8pt, weight: "bold")[Parameters]
  [- #text(size: 8pt)[#raw("center_lat") _(float, default: 0)_]]
  [- #text(size: 8pt)[#raw("center_lon") _(float, default: 0)_]]
  block(
    width: 100%, inset: 8pt, radius: 3pt, fill: luma(245), stroke: 0.5pt + luma(200),
    text(size: 7pt, raw(block: true, lang: "typst", "#render-map(world, (\n  projection: (\n    type: \"wiechel\",\n    center_lat: 90,\n    center_lon: 0,\n  ),\n), width: 100%)"))

  )

  pagebreak()
  [=== Other]
  [==== #link("https://en.wikipedia.org/wiki/Peirce_quincuncial_projection")[Peirce Quincuncial]]
  text(size: 8.5pt, style: "italic")[Published by Charles Sanders Peirce in 1879. Conformal: maps the entire globe onto a square using elliptic integrals. The north pole sits at the center, the south pole is split across the four corners, and the equator forms a diamond. Tessellates the plane.]
  align(center, render-map(world, (..conic_config, projection: (
    type: "peirce_quincuncial",
  ), graticule: graticule), width: 92%))
  text(size: 8pt, weight: "bold")[Parameters]
  [- #text(size: 8pt)[#raw("center_lon") _(float, default: 0)_]]
  block(
    width: 100%, inset: 8pt, radius: 3pt, fill: luma(245), stroke: 0.5pt + luma(200),
    text(size: 7pt, raw(block: true, lang: "typst", "#render-map(world, (\n  projection: (\n    type: \"peirce_quincuncial\",\n    center_lon: 0,\n  ),\n))"))

  )

  pagebreak()
  [==== #link("https://en.wikipedia.org/wiki/AuthaGraph_projection")[AuthaGraph]]
  text(size: 8.5pt, style: "italic")[Invented by Hajime Narukawa in 1999. Maps the sphere onto a tetrahedron, then unfolds it into a rectangle. Nearly equal-area with minimal shape distortion, distributing errors at four oceanic points. The original formulas are proprietary; this is Kunimune's open-source approximation.]
  align(center,
    render-map(world, (..conic_config, projection: (
      type: "authagraph",
    ), graticule: (step: 10, color: "red", opacity: 0.5)), width: 100%, height: 50%)
  )
  block(
    width: 100%, inset: 8pt, radius: 3pt, fill: luma(245), stroke: 0.5pt + luma(200),
    text(size: 7pt, raw(block: true, lang: "typst", "#render-map(world, (\n  projection: (\n    type: \"authagraph\",\n  ),\n))"))

  )
}

#pagebreak()

== Point shapes

`point_shape` sets the marker glyph for `Point`/`MultiPoint` features: `circle` (default), `square`, `diamond`, `triangle`, `cross`, or `star`. Combine with `point_radius` and `point_color`.

```example
#render-map(sweden, (
    fill: "#eef", stroke: "#88a", stroke_width: 0.02,
    point_shape: "star", point_radius: 0.25,
    point_color: "crimson",
  ), width: 70%)
```

#pagebreak()

== Dashed strokes

`stroke_dash` sets an SVG dash pattern — a space-separated list of on/off lengths in map units — for administrative, disputed, or planned boundaries.

```example
#render-map(sweden, (
    fill: "#f7fafc", stroke: "#557", stroke_width: 0.03,
    stroke_dash: "0.25 0.15", point_color: "none",
  ), width: 70%)
```

#pagebreak()

== Label halos and collision

`label_halo` draws a contrasting outline behind label text (via SVG `paint-order`), keeping it readable over busy fills; `label_halo_width` sets its thickness. `label_collide: true` drops labels that would overlap a already-placed one.

```example
#render-map(sweden, (
    fill: "steelblue", fill_opacity: 0.8, stroke: "white",
    stroke_width: 0.02, point_color: "none",
    label: "{name}", label_color: "black", label_font_size: 0.22,
    label_halo: "white", label_halo_width: 0.12,
    label_collide: true,
  ), width: 80%)
```

#pagebreak()

== Filtering features

`filter` renders only the features whose `property` satisfies every given condition: `eq` / `ne` (match any value), `in` (a list of values), or the numeric comparisons `gt` / `lt` / `gte` / `lte`. Conditions combine with logical AND.

```example
#render-map(sweden, (
    filter: (property: "color", gte: 3),
    fill: "seagreen", stroke: "white", stroke_width: 0.02,
    point_color: "none",
  ), width: 70%)
```

#pagebreak()

== Fit to a region

`fit` frames the view on the subset of features matching a selector — the equivalent of d3-geo's `fitExtent` / `fitSize`, but chosen by property rather than by passing a geometry. Every feature is still drawn (so the focus keeps its surrounding context); the viewbox is set to the projected bounds of the matching features, with optional `padding`. Use the same conditions as `filter` (`eq` / `ne` / `in` / `gt` / `lt` / `gte` / `lte`). An explicit `viewbox` overrides it.

To draw *only* a subset instead, use `filter` — the view already fits whatever is drawn.

```example
#render-map(sweden, (
    fill: "#dfe7f0", stroke: "#8fa8c8", stroke_width: 0.02,
    point_color: "none",
    fit: (property: "l_id", "in": (1, 4, 5, 6), padding: 0.08),
    label: "{name}", label_font_size: 0.22,
    label_halo: "white", label_halo_width: 0.1,
  ), width: 70%)
```

#pagebreak()

== Scale types and color schemes

Beyond `quantize` and `linear`, `fill_scale` supports `quantile` (equal-count bins), `threshold` (explicit break points via `domain`), and `diverging` (a two-hue ramp around a `midpoint`). Instead of an explicit `range`, name a built-in `scheme` with `n` classes:

- *Sequential:* `blues`, `greens`, `oranges`, `reds`, `purples`, `greys`, `viridis`, `magma`, `ylgnbu`, `ylorrd`
- *Diverging:* `rdbu`, `rdylbu`, `brbg`, `piyg`, `spectral`
- *Categorical:* `category10`, `tableau10`, `set1`, `set2`, `dark2`

```example
#render-map(sweden, (
    fill_scale: (property: "color", type: "quantile",
      scheme: "viridis", n: 5),
    stroke: "white", stroke_width: 0.02, point_color: "none",
    legend: (title: "color", pos: "bottom-left"),
  ), width: 70%)
```

A `diverging` scale automatically draws a continuous *gradient* legend:

```example
#render-map(sweden, (
    fill_scale: (property: "color", type: "diverging",
      scheme: "rdbu", domain: (0, 4), midpoint: 2),
    stroke: "white", stroke_width: 0.02, point_color: "none",
    legend: (title: "color", pos: "bottom-left"),
  ), width: 70%)
```

#pagebreak()

== Size legend

`size_legend: (title, pos)` draws a nested-circle key for the active `point_radius_scale`, so readers can decode symbol areas back to values.

```example
#let combined = json.encode((type: "FeatureCollection",
  features: json(bytes(sweden)).features
    + json(bytes(cities)).features))
#render-map(combined, (
    projection: (type: "mercator"),
    fill: "#e8e8e8", stroke: "white", stroke_width: 0.02,
    point_radius_scale: (property: "pop",
      max_radius: 1.2, min_radius: 0.1),
    point_color: "crimson", fill_opacity: 0.7,
    size_legend: (title: "population", pos: "bottom-right"),
  ), width: 55%)
```

#pagebreak()

== Density maps

When points are too many to read individually, aggregate them. Both tools operate on `Point`/`MultiPoint` features in projected space.

=== Hexagonal binning

`hexbin` bins points into a hexagonal lattice and colors each cell by its count. Set `radius` (hex size in map units), a `scheme`, and `n` color classes.

```example
#render-map(points, (
    projection: (type: "mercator", central_meridian: 15),
    hexbin: (radius: 0.2, scheme: "ylorrd", n: 6,
      stroke: "white", stroke_width: 0.004),
  ), width: 70%)
```

#pagebreak()

=== Contours

`contour` estimates a smooth density surface (a kernel density estimate) and draws it as filled isolines via marching squares. `bandwidth` controls smoothness, `n` the number of levels, `scheme` the colors, and the optional `cell_size` the grid resolution.

```example
#render-map(points, (
    projection: (type: "mercator", central_meridian: 15),
    contour: (bandwidth: 6, n: 7, scheme: "viridis",
      stroke_width: 0.02),
  ), width: 70%)
```

#pagebreak()

== Cartograms

=== Dorling

A *Dorling cartogram* replaces each feature with a circle sized by a numeric `property`, then nudges the circles apart so they don't overlap — trading exact geographic position for directly comparable symbol sizes. A `fill_scale` colors the circles; `max_radius`/`min_radius` bound their size and `iterations` the relaxation.

```example
#render-map(sweden, (
    projection: (type: "mercator", central_meridian: 16),
    fill_scale: (property: "color", type: "quantize",
      scheme: "reds", n: 5),
    dorling: (property: "l_id", max_radius: 1.0,
      stroke: "white", stroke_width: 0.02),
  ), width: 70%)
```

#pagebreak()

== Layers

`render-layers` overlays several GeoJSON sources on one shared projection and viewbox (the union of their projected bounds), each with its own config — a basemap plus data overlays and points that all line up. Layers draw back-to-front (first = bottom). Put a `graticule`, `sphere`, or `legend` on whichever layer should carry it.

```example
#render-layers((
    (data: sweden, fill: "#eef3f8", stroke: "#8fa8c8",
     stroke_width: 0.02),
    (data: cities, point_color: "crimson", point_radius: 0.18,
     stroke: "white", stroke_width: 0.04,
     label: "{name}", label_font_size: 0.3),
  ), projection: (type: "mercator", central_meridian: 16),
  width: 65%)
```

#pagebreak()

== Great-circle geometry

Two helpers build GeoJSON on the sphere, for flow maps and range rings. Each returns a `Feature`; collect several into a `FeatureCollection` and draw them (here as an overlay via `render-layers`).

=== Arcs

`geo-arc(from, to)` returns the great-circle (shortest) path between two `(lon, lat)` points, interpolated on the sphere so it curves correctly under any projection.

```example
#let hub = (18.07, 59.33) // Stockholm
#let dests = ((-74, 40.7), (139.7, 35.7), (151.2, -33.9))
#let arcs = json.encode((type: "FeatureCollection",
  features: dests.map(d => geo-arc(hub, d))))
#render-layers((
    (data: world, fill: "#eee", stroke: "#ccc", stroke_width: 0.01),
    (data: arcs, fill: "none", stroke: "crimson", stroke_width: 0.06),
  ), projection: (type: "natural_earth"), width: 95%)
```

#pagebreak()

=== Circles and range rings

`geo-circle(center, radius)` returns a polygon approximating a circle of a given *angular* radius (degrees) around a point — e.g. `500 / 111.32` for roughly 500 km.

```example
#let center = (10, 50)
#let rings = json.encode((type: "FeatureCollection",
  features: (500, 1500, 3000).map(km =>
    geo-circle(center: center, radius: km / 111.32))))
#render-layers((
    (data: world, fill: "#eef", stroke: "#ccd", stroke_width: 0.01),
    (data: rings, fill: "none", stroke: "darkred",
     stroke_width: 0.05),
  ), projection: (type: "orthographic",
    center_lat: 45, center_lon: 10), width: 55%)
```

#pagebreak()

== Clipping

=== Rectangular (`clip_extent`)

`clip_extent: (lon0, lat0, lon1, lat1)` crops the map to a geographic rectangle. The box is projected into the map's coordinate space, so it works under any projection; when no explicit `viewbox` is set, the view automatically frames (zooms to) the clipped region.

```example
#render-map(world, (
    projection: (type: "equirectangular"),
    fill: "#6fbf5f", stroke: "white", stroke_width: 0.03,
    graticule: (step: 10, color: "#ccc", opacity: 0.5),
    clip_extent: (-12, 34, 40, 72), // Europe
  ), width: 75%)
```

=== Small-circle (`clip_angle`)

On an azimuthal projection, `clip_angle` (degrees) clips to a small circle around the projection center — useful for local or hemispheric views.

```example
#render-map(world, (
    projection: (type: "azimuthal_equidistant",
      center_lat: 45, center_lon: 10),
    fill: "#6fbf5f", stroke: "#356b2c", stroke_width: 0.01,
    graticule: (step: 15, color: "#ccc", opacity: 0.5),
    clip_angle: 40,
  ), width: 55%)
```

#pagebreak()

== Rotation

`rotate: (lambda, phi, gamma)` applies a full three-axis spherical rotation (à la `d3.geoRotation`) *before* projecting, letting you re-center or tilt any non-azimuthal projection — for oblique aspects. (Azimuthal projections use `center_lat`/`center_lon` instead.)

```example
#render-map(world, (
    projection: (type: "hammer"),
    rotate: (-60, -25, 0),
    fill: "#6fbf5f", stroke: "white", stroke_width: 0.008,
    sphere: (fill: "#dceefb", stroke: "#8899bb",
      stroke_width: 0.006),
    graticule: (step: 20, color: "#fff", opacity: 0.7, width: 0.3),
  ), width: 80%)
```

#pagebreak()

== Adaptive resampling

`precision` subdivides long line/polygon segments so that straight edges follow the projection's curvature — most visible on curved projections and graticules. It is the resampling threshold in projected units; smaller is finer, `none` (the default) is off. Clipped and azimuthal paths are already re-stitched, so this mainly refines unclipped geometry.

```example
#render-map(world, (
    projection: (type: "orthographic",
      center_lat: 20, center_lon: 0),
    precision: 0.01,
    fill: "steelblue", stroke: "white", stroke_width: 0.002,
    graticule: (step: 15, color: "#fff", opacity: 0.5),
  ), width: 55%)
```

#pagebreak()

== Measurements

Two helpers return geometry in the same projected coordinate space as the rendered SVG and its `viewbox`, so you can place your own Typst content over a map or frame it precisely.

- `map-bounds(code, projection: ..)` → the projected bounds `(x, y, w, h)` of a source. `render-layers` uses it internally to align layers; it is also handy for computing a shared `viewbox` yourself.
- `map-centroids(code, projection: ..)` → an array with each feature's projected centroid `(x, y)` (or `none`), for placing custom markers or labels at feature centers.

#code-block(text(size: 7pt, raw(block: true, lang: "typst", "#let proj = (type: \"mercator\", central_meridian: 16)\n#let (x, y, w, h) = map-bounds(sweden, projection: proj)\n#let centers = map-centroids(sweden, projection: proj)\n// centers.at(0) == (cx, cy) of the first feature, in viewbox units")))

#pagebreak()

== Putting it all together

A finished thematic map combines many of these features at once: a projection with a graticule, a *choropleth* whose colors come from data joined in from an external table, a `legend` for the key, and per-feature `label`s. Here each Swedish region is colored by a `density` value (people per km², invented for the example) joined on `l_id`, with the region name drawn at its centroid.

#let density = (
  "1": (density: 360), "3": (density: 44), "4": (density: 39),
  "5": (density: 41), "6": (density: 66), "7": (density: 32),
  "8": (density: 26), "9": (density: 62), "10": (density: 51),
  "12": (density: 118), "13": (density: 66), "14": (density: 71),
  "17": (density: 36), "18": (density: 36), "19": (density: 42),
  "20": (density: 12), "21": (density: 11), "22": (density: 6),
  "23": (density: 5), "24": (density: 3), "25": (density: 3),
)

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  align: horizon,
  code-block(text(size: 6.5pt, raw(block: true, lang: "typst", ```typ
// density: a dict of l_id -> (density: n),
// e.g. a table read from a CSV.
#render-map(sweden, (
  projection: (type: "mercator",
    central_meridian: 16),
  fill_scale: (
    property: "density",
    type: "quantize",
    domain: (0, 120),
    range: ("#ffffcc", "#c2e699", "#78c679",
            "#31a354", "#006837"),
    default: "#eeeeee",
  ),
  legend: (title: "density (/km²)",
    pos: "top-right"),
  stroke: "white", stroke_width: 0.02,
  point_color: "none",
  label: (text: "{name}", font_size: 0.22),
  graticule: (step: 4, color: "#ccc",
    opacity: 0.35, width: 0.3),
), data: density, key: "l_id")
```.text))),
  render-map(sweden, (
    projection: (type: "mercator", central_meridian: 16),
    fill_scale: (
      property: "density",
      type: "quantize",
      domain: (0, 120),
      range: ("#ffffcc", "#c2e699", "#78c679", "#31a354", "#006837"),
      default: "#eeeeee",
    ),
    legend: (title: "density (/km²)", pos: "top-right"),
    stroke: "white",
    stroke_width: 0.02,
    point_color: "none",
    label: (text: "{name}", font_size: 0.22, color: "#222", font_family: "New Computer Modern"),
    graticule: (step: 4, color: "#ccc", opacity: 0.35, width: 0.3),
  ), data: density, key: "l_id", width: 100%),
)
