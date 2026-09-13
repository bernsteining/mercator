# mercator

![Worldmap projected in orthographic projection](https://github.com/bernsteining/mercator/blob/v0.1.2/examples/data/logo.png)

Mercator is a Typst plugin to render GeoJSON and TopoJSON as SVG maps.

**[Try the live demo →](https://bernsteining.github.io/mercator/)** — pick a dataset and projection and see the SVG render in your browser (same wasm the plugin ships).

## usage

```typst
#import "@preview/mercator:0.1.2": *

#let world = read("examples/data/world.json", encoding: none)

#render-map(world, (
  projection: (
    type: "orthographic",
    center_lat: 45,
    center_lon: 10,
  ),
  graticule: (step: 15),
), width: 100%)
```

## documentation

Check [examples/documentation.pdf](https://github.com/bernsteining/mercator/blob/v0.1.2/examples/documentation.pdf), it covers all the features with examples.

## config options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `stroke` | string | `"black"` | Stroke color. Supports `{property_name}` interpolation. |
| `stroke_width` | float | `0.05` | Stroke width |
| `fill` | string | `"white"` | Fill color. Supports `{property_name}` interpolation. |
| `fill_opacity` | float | `1.0` | Fill opacity |
| `fill_pattern` | string | none | `"hatched"`, `"crosshatched"`, or `"dotted"`. Supports `{property_name}`. |
| `point_radius` | float | `stroke_width * 5` | Radius for Point/MultiPoint geometries |
| `point_color` | string | same as `fill` | Point fill color. `"none"` hides points. Supports `{property_name}`. |
| `viewbox` | array | auto | Manual viewbox as `(x, y, width, height)` |
| `viewbox_padding` | float | `0.15` | Padding fraction around auto-computed viewbox |
| `label` | string or array | none | Label template: `"{name}"` or array of `{text, font_size, color, font_family}` objects |
| `label_color` | string | `"black"` | Default label color |
| `label_font_size` | float | `0.3` | Default label font size |
| `label_font_family` | string | `"Arial"` | Default label font family |
| `projection` | object | equirectangular | Projection config (see below) |
| `graticule` | object | none | Graticule overlay config (see below) |
| `tissot` | object | none | Tissot's indicatrix overlay config (see below) |
| `sphere` | object | none | Filled globe/ocean disc for azimuthal projections (see below); hemisphere clipping is automatic and needs no config |
| `antimeridian` | bool | `false` | Clip polygons at the antimeridian (cylindrical projections) so seam-crossing shapes (e.g. Antarctica) close cleanly instead of streaking (see below) |
| `fill_scale` | object | none | Data-driven fill from a numeric property, i.e. a choropleth (see below) |
| `point_radius_scale` | object | none | Data-driven point size from a numeric property, i.e. proportional symbols (see below) |
| `legend` | object | none | Legend key for the active `fill_scale` (see below) |

### projections

| Type | Category | Parameters |
|------|----------|------------|
| `equirectangular` | Cylindrical | `central_meridian` |
| `mercator` | Cylindrical | `central_meridian` |
| `cassini` | Cylindrical | `central_meridian` |
| `lambert_conformal_conic` | Conic | `standard_parallel_1`, `standard_parallel_2`, `central_meridian`, `latitude_of_origin` |
| `albers_equal_area` | Conic | `standard_parallel_1`, `standard_parallel_2`, `central_meridian`, `latitude_of_origin` |
| `bonne` | Pseudo-conic | `standard_parallel`, `central_meridian` |
| `polyconic` | Pseudo-conic | `central_meridian` |
| `robinson` | Pseudo-cylindrical | `central_meridian` |
| `natural_earth` | Pseudo-cylindrical | `central_meridian` |
| `hammer` | Pseudo-cylindrical | `central_meridian` |
| `winkel_tripel` | Pseudo-cylindrical | `central_meridian` |
| `orthographic` | Azimuthal | `center_lat`, `center_lon` |
| `gnomonic` | Azimuthal | `center_lat`, `center_lon` |
| `lambert_azimuthal_equal_area` | Azimuthal | `center_lat`, `center_lon` |
| `azimuthal_equidistant` | Azimuthal | `center_lat`, `center_lon` |
| `wiechel` | Pseudo-azimuthal | `center_lat`, `center_lon` |
| `peirce_quincuncial` | Other | `center_lon` |
| `authagraph` | Other | _(no parameters)_ |

### graticule

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `step` | float | `15.0` | Degrees between grid lines |
| `color` | string | `"#ccc"` | Line color |
| `width` | float | `0.5` | Line width |
| `opacity` | float | `0.6` | Line opacity |

### tissot

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `step` | float | `30.0` | Degrees between indicator circles |
| `radius` | float | `5.0` | Circle radius in degrees |
| `fill` | string | `"red"` | Fill color |
| `fill_opacity` | float | `0.3` | Fill opacity |
| `stroke` | string | `"red"` | Stroke color |
| `stroke_width` | float | `0.5` | Stroke width |
| `max_lat` | float | `60.0` | Maximum latitude for indicators |

### sphere

Azimuthal globe projections (currently `orthographic`) **always** clip geometry to the visible hemisphere — polygons are cut at the limb and re-stitched along it, so land closes cleanly against the horizon instead of breaking. This happens automatically, with or without a `sphere` config. The optional `sphere` config only adds a filled ocean disc drawn behind the land.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `fill` | string | `"#cfe8ff"` | Disc fill (`"none"` draws no ocean; clipping still applies) |
| `stroke` | string | none | Disc outline color |
| `stroke_width` | float | `0.005` | Disc outline width |

```typ
#render-map(world, (
  projection: (type: "orthographic", center_lat: 25, center_lon: 10),
  sphere: (fill: "#a8d4f0", stroke: "#268"),
  fill: "#6fbf5f", stroke: "#356b2c",
  graticule: (step: 15, color: "#fff"),
))
```

### antimeridian

`antimeridian: true` clips polygons at the antimeridian for cylindrical projections (`equirectangular`, `mercator`, `cassini`). Polygons that cross the ±180° seam are cut and re-stitched along the map boundary — including around a pole — so shapes like Antarctica fill correctly instead of breaking into a streak. Uses the d3-geo clip/rejoin algorithm; a faithful port. Off by default (the previous per-vertex behavior is unchanged).

```typ
#render-map(world, (
  projection: (type: "equirectangular"),
  antimeridian: true,
  fill: "#6fbf5f", stroke: "#356b2c",
))
```

### fill_scale

Colors each feature from a numeric property (a choropleth), overriding `fill`.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `property` | string | required | Property to read from each feature |
| `type` | string | `"quantize"` | `"quantize"` (discrete bins), `"linear"` (interpolated), or `"category"` |
| `domain` | array | auto | `(min, max)` for numeric types; computed from the data when omitted |
| `range` | array | required* | Colors: discrete bins for `quantize`, gradient stops for `linear` (needs hex). *Not used by `category`. |
| `categories` | object | none | `category` only: explicit `value → color` map |
| `palette` | array | none | `category` only: colors auto-assigned to distinct values (first-seen order) when `categories` is omitted |
| `default` | string | `"#cccccc"` | Color for features whose value is missing/unmatched |

For `type: "category"` the property can be a string or number; each distinct value gets a color, either from an explicit `categories` map or auto-assigned from a `palette`:

```typ
fill_scale: (property: "party", type: "category",
  categories: ("A": "#e41a1c", "B": "#377eb8", "C": "#4daf4a"))
// or: (property: "party", type: "category", palette: ("#e41a1c", "#377eb8", "#4daf4a"))
```

```typ
#render-map(read("data.json", encoding: none), (
  fill_scale: (
    property: "gdp",
    type: "quantize",
    range: ("#eff3ff", "#c6dbef", "#9ecae1", "#6baed6", "#3182bd", "#08519c"),
  ),
  legend: (title: "GDP", pos: "bottom-right"),
))
```

### point_radius_scale

Sizes `Point`/`MultiPoint` symbols from a numeric property (a proportional-symbol / bubble map), overriding `point_radius`.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `property` | string | required | Numeric property to read from each feature |
| `type` | string | `"sqrt"` | `"sqrt"` (area ∝ value — the perceptual default) or `"linear"` (radius ∝ value) |
| `domain` | array | auto | `(min, max)`; computed from the data when omitted |
| `max_radius` | float | `1.0` | Radius for the largest value (map/projection units) |
| `min_radius` | float | `0.0` | Radius for the smallest value |

```typ
#render-map(read("cities.json", encoding: none), (
  projection: (type: "mercator"),
  point_radius_scale: (property: "population", max_radius: 1.2),
  point_color: "crimson",
))
```

### legend

Draws a swatch key for the active `fill_scale` inside a viewbox corner.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `title` | string | none | Legend heading |
| `pos` | string | `"bottom-right"` | `"top-left"`, `"top-right"`, `"bottom-left"`, or `"bottom-right"` |

### joining external data

`fill_scale` reads a property that must exist on each feature. If your data lives in a separate table instead of inside the GeoJSON, pass it to `render-map` as `data` (with a `key`) and it is merged into each feature's properties before rendering — the classic GIS "join". `data` is a dictionary keyed by the join value, or an array of records (e.g. from `csv(.., row-type: dictionary)`). The standalone `join(code, data, key: ..)` returns the merged GeoJSON.

```typ
#let rates = ("SE-01": (rate: 90), "SE-03": (rate: 55))

#render-map(read("regions.json", encoding: none), (
  fill_scale: (property: "rate", type: "quantize", range: (..)),
), data: rates, key: "id")
```

## build locally

```sh
cargo build --target wasm32-unknown-unknown --release
wasm-opt -O4 --enable-simd --enable-bulk-memory --enable-sign-ext \
  --enable-nontrapping-float-to-int --enable-mutable-globals --strip-debug \
  target/wasm32-unknown-unknown/release/mercator.wasm -o mercator/mercator.wasm
```

NB: `wasm-opt` is part of [binaryen](https://github.com/WebAssembly/binaryen).
