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
#show heading.where(level: 3): set text(size: 1.17em)
#show heading.where(level: 4): set text(size: 1.05em)

// --- Data ---

#let sweden = read("data/swedish_regions.json", encoding: "utf8")
#let world = read("data/world.json", encoding: "utf8")
#let world_no_ant = read("data/world_no_antartica.json", encoding: "utf8")

// --- Example helper ---
// Show rule that displays a code block and executes it.

#let doc-scope = ("render-map": render-map, sweden: sweden, world: world)
#let code-block(body) = block(
  width: 100%, inset: 8pt, radius: 3pt,
  fill: luma(245), stroke: 0.5pt + luma(200), body,
)
#show raw.where(lang: "example"): it => {
  code-block(text(size: 7pt, raw(block: true, lang: "typst", it.text)))
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
#let sweden = read("data/swedish_regions.json", encoding: "utf8")
#render-map(sweden, width: 80%)
```

The `render-map` function takes two positional arguments:
- `data` — GeoJSON or TopoJSON string
- `config` — JSON-encoded configuration string (optional)

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

All rendering options are passed as a JSON-encoded dictionary. Every field is optional and defaults to the value shown below. Fields marked with `{property}` support per-feature interpolation from GeoJSON properties. (cf. per-feature styling section)

```json
{
  // --- Appearance ---
  "stroke":       "black", // string – CSS color name or hex
  "stroke_width": 0.05,    // float – border thickness
  "fill":         "white", // string – CSS color name or hex
  "fill_opacity": 1.0,     // float – 0.0 to 1.0 (transparent⟶opaque)
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
  "viewbox_padding": 0.15,  // float – padding fraction around auto viewbox

  "projection": null,  // object, see Projections section
  "graticule": null,   // object, see Graticule section
  "tissot": null       // object, see Tissot's Indicatrix section
}
```

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
  code-block(text(size: 7pt, raw(block: true, lang: "typst", "#render-map(sweden, json.encode((\n  stroke: \"black\",\n  stroke_width: 0.02,\n  fill: \"grey\",\n  fill_opacity: 0.5,\n  viewbox: array((15.0, -69.4, 10.0, 6.0)),\n  point_color: \"none\",\n)), width: 70%)"))),
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
    label_color: "red",
    label_font_size: 0.6,
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
  )), width: 80%)
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
    #render-map(world, json.encode(ortho_base), width: 100%)
    #align(center, text(size: 8pt)[Without graticule])
  ],
  [
    #render-map(world, json.encode((..ortho_base,
      graticule: (step: 15, color: "red", opacity: 0.5),
    )), width: 100%)
    #align(center, text(size: 8pt)[With graticule])
  ],
)

#code-block(text(size: 7pt, raw(block: true, lang: "typst", "graticule: (\n  step: 15,      // degrees between lines (default: 15)\n  color: \"red\",  // line color (default: \"#ccc\")\n  opacity: 0.5,  // line opacity (default: 0.6)\n  width: 0.5,    // line width (default: 0.5)\n)")))

The following examples of projections will use a graticule to visually represent the distortions due to the projections.

#pagebreak()

=== Tissot's Indicatrix

#link("https://en.wikipedia.org/wiki/Tissot%27s_indicatrix")[Tissot's indicatrix] places small circles at regular grid points. After projection, these circles deform into ellipses that reveal how the projection distorts shapes and areas.

On a *conformal* projection (like Mercator), circles stay circular but grow near the poles. On an *equal-area* projection (like Albers), circles keep the same area but get squished. On a *compromise* projection (like Robinson), both shape and area change.

#align(center, render-map(world, json.encode((
  projection: (type: "mercator"),
  stroke: "#aaa", stroke_width: 0.01, fill: "none",
  graticule: (step: 30, color: "#ddd", opacity: 0.4, width: 0.2),
  tissot: (step: 30, radius: 5, fill: "red", fill_opacity: 0.4, stroke: "darkred", stroke_width: 0.3),
)), width: 80%))

#code-block(text(size: 7pt, raw(block: true, lang: "typst", "tissot: (\n  step: 30,          // degrees between circles (default: 30)\n  radius: 5,         // circle radius in degrees (default: 5)\n  fill: \"red\",       // fill color (default: \"red\")\n  fill_opacity: 0.3, // fill opacity (default: 0.3)\n  stroke: \"red\",     // stroke color (default: \"red\")\n  stroke_width: 0.5, // stroke width (default: 0.5)\n  max_lat: 60,       // maximum latitude in degrees (default: 60)\n)")))

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
      code: "#let world = read(\"data/world.json\", encoding: \"utf8\")\n#render-map(world, json.encode((\n  graticule: (step: 15),\n)))",
    ),
    (
      category: "Cylindrical",
      config: (..world_config, projection: (type: "mercator")),
      grat_config: (..world_config, projection: (type: "mercator"), graticule: graticule),
      name: "Mercator",
      url: "https://en.wikipedia.org/wiki/Mercator_projection",
      desc: "Introduced by Gerardus Mercator in 1569 for nautical navigation. Conformal: preserves local angles and shapes, so any straight line is a constant-bearing rhumb line. Extreme area distortion near the poles.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"mercator\",\n    central_meridian: 0,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"cassini\",\n    central_meridian: 0,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"lambert_conformal_conic\",\n    standard_parallel_1: 30,\n    standard_parallel_2: 60,\n    central_meridian: 10,\n    latitude_of_origin: 0,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"albers_equal_area\",\n    standard_parallel_1: 30,\n    standard_parallel_2: 60,\n    central_meridian: 10,\n    latitude_of_origin: 40,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"bonne\",\n    standard_parallel: 45,\n    central_meridian: 10,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"polyconic\",\n    central_meridian: 10,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"robinson\",\n    central_meridian: 0,\n  ),\n)))",
    ),
    (
      category: "Pseudo-cylindrical",
      config: (..conic_config, projection: (type: "natural_earth")),
      grat_config: (..conic_config, projection: (type: "natural_earth"), graticule: graticule),
      name: "Natural Earth",
      url: "https://en.wikipedia.org/wiki/Natural_Earth_projection",
      desc: "Designed by Tom Patterson in 2008 for the Natural Earth dataset. A compromise projection similar to Robinson but with smoother, rounder corners. Adopted by many open-source mapping tools as a default world view.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"natural_earth\",\n    central_meridian: 0,\n  ),\n)))",
    ),
    (
      category: "Pseudo-cylindrical",
      config: (..conic_config, projection: (type: "hammer")),
      grat_config: (..conic_config, projection: (type: "hammer"), graticule: graticule),
      name: "Hammer",
      url: "https://en.wikipedia.org/wiki/Hammer_projection",
      desc: "Developed by Ernst Hammer in 1892 as a modification of the Aitoff projection. Equal-area: maps the entire globe into an ellipse with a 2:1 axis ratio. Meridians are curved, equally spaced along the equator. Widely used for whole-world maps in atlases.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"hammer\",\n    central_meridian: 0,\n  ),\n)))",
    ),
    (
      category: "Pseudo-cylindrical",
      config: (..conic_config, projection: (type: "winkel_tripel")),
      grat_config: (..conic_config, projection: (type: "winkel_tripel"), graticule: graticule),
      name: "Winkel Tripel",
      url: "https://en.wikipedia.org/wiki/Winkel_tripel_projection",
      desc: "Created by Oswald Winkel in 1921. A compromise projection computed as the arithmetic mean of equirectangular and Aitoff projections. Minimizes the sum of distortions in area, direction, and distance --- hence \"tripel\" (German for triple). Adopted by the National Geographic Society in 1998 as their standard world map projection.",
      params: ((name: "central_meridian", typ: "float", default: "0"),),
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"winkel_tripel\",\n    central_meridian: 0,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"lambert_azimuthal_equal_area\",\n    center_lat: 45,\n    center_lon: 10,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"gnomonic\",\n    center_lat: 45,\n    center_lon: 10,\n  ),\n)))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"orthographic\",\n    center_lat: 45,\n    center_lon: 10))))",
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
      code: "#render-map(world, json.encode((\n  projection: (\n    type: \"azimuthal_equidistant\",\n    center_lat: 45,\n    center_lon: 10,\n  ),\n)))",
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
      [==== #proj.name]
      text(size: 8.5pt, style: "italic")[#proj.desc #link(proj.url)[\[Wikipedia\]]]
      align(center, render-map(geo_data, json.encode(proj.grat_config), width: w))
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
  [==== Wiechel]
  text(size: 8.5pt, style: "italic")[Invented by H. Wiechel in 1879. An equal-area azimuthal projection with a distinctive swirl: each meridian is a circular arc, giving the map a pinwheel-like appearance. One of the few projections that is both equal-area and visually striking. #link("https://en.wikipedia.org/wiki/Wiechel_projection")[\[Wikipedia\]]]
  align(center, render-map(world, json.encode((..conic_config, projection: (
    type: "wiechel", center_lat: 90, center_lon: 0,
  ), graticule: graticule)), width: 92%))
  text(size: 8pt, weight: "bold")[Parameters]
  [- #text(size: 8pt)[#raw("center_lat") _(float, default: 0)_]]
  [- #text(size: 8pt)[#raw("center_lon") _(float, default: 0)_]]
  block(
    width: 100%, inset: 8pt, radius: 3pt, fill: luma(245), stroke: 0.5pt + luma(200),
    text(size: 7pt, raw(block: true, lang: "typst", "#render-map(world, json.encode((\n  projection: (\n    type: \"wiechel\",\n    center_lat: 90,\n    center_lon: 0,\n  ),\n)), width: 100%)"))
  )

  pagebreak()
  [=== Other]
  [==== Peirce Quincuncial]
  text(size: 8.5pt, style: "italic")[Published by Charles Sanders Peirce in 1879. Conformal: maps the entire globe onto a square using elliptic integrals. The north pole sits at the center, the south pole is split across the four corners, and the equator forms a diamond. Tessellates the plane. #link("https://en.wikipedia.org/wiki/Peirce_quincuncial_projection")[\[Wikipedia\]]]
  align(center, render-map(world, json.encode((..conic_config, projection: (
    type: "peirce_quincuncial",
  ), graticule: graticule)), width: 92%))
  text(size: 8pt, weight: "bold")[Parameters]
  [- #text(size: 8pt)[#raw("center_lon") _(float, default: 0)_]]
  block(
    width: 100%, inset: 8pt, radius: 3pt, fill: luma(245), stroke: 0.5pt + luma(200),
    text(size: 7pt, raw(block: true, lang: "typst", "#render-map(world, json.encode((\n  projection: (\n    type: \"peirce_quincuncial\",\n    center_lon: 0,\n  ),\n)))"))
  )

  pagebreak()
  [==== AuthaGraph]
  text(size: 8.5pt, style: "italic")[Invented by Hajime Narukawa in 1999. Maps the sphere onto a tetrahedron, then unfolds it into a rectangle. Nearly equal-area with minimal shape distortion, distributing errors at four oceanic points. The original formulas are proprietary; this is Kunimune's open-source approximation. #link("https://en.wikipedia.org/wiki/AuthaGraph_projection")[\[Wikipedia\]]]
  align(center)[
    #render-map(world, json.encode((..conic_config, projection: (
      type: "authagraph",
    ), graticule: (step: 10, color: "red", opacity: 0.5))), width: 100%)
  ]
  block(
    width: 100%, inset: 8pt, radius: 3pt, fill: luma(245), stroke: 0.5pt + luma(200),
    text(size: 7pt, raw(block: true, lang: "typst", "#render-map(world, json.encode((\n  projection: (\n    type: \"authagraph\",\n  ),\n))"))
  )
}

#pagebreak()

== Putting it all together

Combining projection, graticule, Tissot's indicatrix, per-feature styling, fill patterns, and multi-line labels on a single map.

#grid(
  columns: (1fr, 1fr),
  gutter: 1em,
  code-block(text(size: 7pt, raw(block: true, lang: "typst", "#render-map(sweden, json.encode((\n  stroke: \"white\",\n  stroke_width: 0.01,\n  fill: \"{fill_color}\",\n  fill_opacity: 0.8,\n  fill_pattern: \"{pattern}\",\n  point_radius: 0.15,\n  point_color: \"magenta\",\n  label: (\n    (text: \"{point}\", font_size: 0.40,\n     color: \"black\",\n     font_family: \"New Computer Modern\"),\n    (text: \"id: {l_id}\", font_size: 0.12,\n     color: \"red\"),\n  ),\n  projection: (\n    type: \"mercator\",\n    central_meridian: 16,\n  ),\n  viewbox: (-6.4, -74.4, 10, 10),\n  graticule: (step: 2, color: \"#ccc\",\n    opacity: 0.4, width: 0.3),\n  tissot: (step: 3.3, radius: 0.5,\n    fill: \"red\", fill_opacity: 0.2,\n    max_lat: 80),\n)), width: 100%)"))),
  render-map(sweden, json.encode((
    stroke: "white",
    stroke_width: 0.01,
    fill: "{fill_color}",
    fill_opacity: 0.8,
    fill_pattern: "{pattern}",
    point_radius: 0.15,
    point_color: "magenta",
    label: (
      (text: "{point}", font_size: 0.40, color: "black", font_family: "New Computer Modern"),
      (text: "id: {l_id}", font_size: 0.12, color: "red"),
    ),
    projection: (
      type: "mercator",
      central_meridian: 16,
    ),
    viewbox: (-6.4, -74.4, 10, 10),
    graticule: (step: 2, color: "#ccc", opacity: 0.4, width: 0.3),
    tissot: (step: 3.3, radius: 0.5, fill: "red", fill_opacity: 0.2, max_lat: 80),
  )), width: 100%),
)
