<p align="center">
  <img src="https://raw.githubusercontent.com/bernsteining/mercator/main/docs/logo.svg" alt="mercator" width="160">
</p>

<h1 align="center">mercator</h1>

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
| `stroke_width` | float/string | `0.05` | Stroke width. May be a `{property_name}` template for data-driven line weight. |
| `fill` | string | `"white"` | Fill color. Supports `{property_name}` interpolation. |
| `fill_opacity` | float/string | `1.0` | Fill opacity. May be a `{property_name}` template. |
| `fill_pattern` | string | none | `"hatched"`, `"crosshatched"`, or `"dotted"`. Supports `{property_name}`. |
| `point_radius` | float | `stroke_width * 5` | Radius for Point/MultiPoint geometries |
| `point_color` | string | same as `fill` | Point fill color. `"none"` hides points. Supports `{property_name}`. |
| `point_shape` | string | `"circle"` | Marker shape for Point/MultiPoint: `circle`, `square`, `diamond`, `triangle`, `cross`, or `star`. Supports `{property_name}`. |
| `stroke_dash` | string | none | SVG `stroke-dasharray` for line/polygon borders, e.g. `"0.25 0.15"` (map units). Supports `{property_name}`. |
| `label_halo` | string | none | Halo/outline color drawn behind label text (legibility over busy maps) |
| `label_halo_width` | float | `0.12 × font size` | Halo width in map units |
| `label_collide` | bool | `false` | Drop labels whose bounding box overlaps an already-placed label (first wins) |
| `viewbox` | array | auto | Manual viewbox as `(x, y, width, height)` in projected units — zoom/frame a specific region of the map |
| `viewbox_padding` | float | `0.15` | Padding fraction around auto-computed viewbox |
| `label` | string or array | none | Label template: `"{name}"` or array of `{text, font_size, color, font_family}` objects |
| `label_color` | string | `"black"` | Default label color |
| `label_font_size` | float | `0.3` | Default label font size |
| `label_font_family` | string | `"Arial"` | Default label font family |
| `projection` | object | equirectangular | Projection config (see below) |
| `rotate` | array | none | Spherical pre-rotation `[lambda, phi, gamma]` in degrees (d3.geoRotation) — recenter/tilt/roll a non-azimuthal projection to an oblique aspect (azimuthal projections use `center_lat`/`center_lon` instead) |
| `precision` | float | none | Adaptive-resampling tolerance in projected units. When set, straight lon/lat segments are subdivided so they follow the projection's curve (useful for coarse geometry on curved projections). Off when omitted. |
| `graticule` | object | none | Graticule overlay config (see below) |
| `tissot` | object | none | Tissot's indicatrix overlay config (see below) |
| `sphere` | object | none | Filled globe/ocean disc for azimuthal projections (see below); hemisphere clipping is automatic and needs no config |
| `clip_extent` | array | none | Crop the map to a geographic rectangle `(lon0, lat0, lon1, lat1)`. Projected into map space (works under any projection); with no explicit `viewbox`, the view auto-frames the cropped region. |
| `clip_angle` | float | none | Clip an azimuthal map to a small circle of this angular radius (degrees) |
| `antimeridian` | bool | `false` | Clip polygons at the antimeridian (cylindrical projections) so seam-crossing shapes (e.g. Antarctica) close cleanly instead of streaking (see below) |
| `hexbin` | object | none | Aggregate Point/MultiPoint features into a hexagonal density grid — `(radius, scheme?, n?, stroke?, stroke_width?)`. Cells are colored by point count. |
| `dorling` | object | none | Dorling cartogram: replace each feature with a circle sized by a numeric property, placed near its centroid with collision repulsion — `(property, max_radius, min_radius?, domain?, iterations?, stroke?, stroke_width?)`. Circle fill comes from the resolved `fill` (so `fill_scale` colors them). |
| `contour` | object | none | Density contours of Point/MultiPoint features (KDE + marching squares) — `(cell_size?, bandwidth?, n?, scheme?, stroke_width?)`. Iso-lines colored by level. |
| `filter` | object | none | Render only features whose `property` matches — `(property, eq?, ne?, in?, gt?, lt?, gte?, lte?)`; conditions combine with logical AND. |
| `fit` | object | none | Frame the view to the subset of features matching a selector (all features are still drawn) — like d3-geo's fitExtent/fitSize. Same conditions as `filter`, plus `padding?`. Overridden by an explicit `viewbox`. |
| `fill_scale` | object | none | Data-driven fill from a numeric property, i.e. a choropleth (see below) |
| `point_radius_scale` | object | none | Data-driven point size from a numeric property, i.e. proportional symbols (see below) |
| `legend` | object | none | Legend key for the active `fill_scale` (discrete swatches, or a gradient bar for `linear`/`diverging`) — `(title?, pos)` |
| `size_legend` | object | none | Nested-circle size key for the active `point_radius_scale` — `(title?, pos)` |

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
| `mollweide` | Pseudo-cylindrical (equal-area) | `central_meridian` |
| `sinusoidal` | Pseudo-cylindrical (equal-area) | `central_meridian` |
| `eckert4` | Pseudo-cylindrical (equal-area) | `central_meridian` |
| `eckert6` | Pseudo-cylindrical (equal-area) | `central_meridian` |
| `kavrayskiy7` | Pseudo-cylindrical (compromise) | `central_meridian` |
| `wagner6` | Pseudo-cylindrical (compromise) | `central_meridian` |
| `van_der_grinten` | Compromise (circular) | `central_meridian` |
| `miller` | Cylindrical (compromise) | `central_meridian` |
| `gall_stereographic` | Cylindrical (compromise) | `central_meridian` |
| `gall_peters` | Cylindrical (equal-area) | `central_meridian` |
| `aitoff` | Lenticular (compromise) | `central_meridian` |
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

A `sphere` config draws an ocean background behind the land, shaped to the projection: a **disc** for azimuthal globes (`orthographic`) or the projection's **frame** (ellipse / lens / rectangle) for non-azimuthal projections (Mollweide, Hammer, Aitoff, Robinson, …). Separately, azimuthal globes **always** clip geometry to the visible hemisphere (land is cut at the limb and re-stitched) — automatically, with or without a `sphere`.

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `fill` | string | `"#cfe8ff"` | Ocean fill (`"none"` draws none) |
| `stroke` | string | none | Outline color |
| `stroke_width` | float | `0.005` | Outline width |

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
| `type` | string | `"quantize"` | `"quantize"` (equal-width bins), `"quantile"` (equal-count bins), `"threshold"` (explicit breaks), `"linear"` (interpolated), `"diverging"` (interpolated around a midpoint), or `"category"` |
| `domain` | array | auto | `(min, max)` for numeric types; computed from the data when omitted |
| `range` | array | required* | Colors: discrete bins for `quantize`/`quantile`/`threshold`, gradient stops for `linear`/`diverging` (needs hex). *Optional if `scheme` is set; not used by `category`. |
| `scheme` | string | none | Named color scheme used when `range` is omitted — sequential (`blues`, `greens`, `oranges`, `reds`, `purples`, `greys`, `viridis`, `magma`, `ylgnbu`, `ylorrd`), diverging (`rdbu`, `rdylbu`, `brbg`, `piyg`, `spectral`), or categorical (`category10`/`tableau10`, `set1`, `set2`, `dark2`) |
| `n` | int | `5` | Number of classes to sample from `scheme` for `quantize`/`quantile` |
| `breaks` | array | none | `threshold` only: N break points → N+1 colors |
| `midpoint` | float | domain mid | `diverging` only: value pinned to the middle color |
| `categories` | object | none | `category` only: explicit `value → color` map |
| `palette` | array | none | `category` only: colors auto-assigned to distinct values (first-seen order) when `categories`/`scheme` are omitted |
| `default` | string | `"#cccccc"` | Color for features whose value is missing/unmatched |

`quantile` bins the data so each color holds ~the same number of features (robust to outliers); `threshold` uses your explicit `breaks`; `diverging` anchors `midpoint` to the middle color (for signed data). Any of these can take a named `scheme` instead of a hand-written `range`, e.g. `(property: "gdp", type: "quantile", scheme: "viridis", n: 6)`.

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

### range rings (geo-circle)

`geo-circle(center: (lon, lat), radius: deg, steps: 64, properties: (:))` builds a GeoJSON polygon approximating a spherical circle (a great-circle "range ring"), like `d3.geoCircle`. It returns a Feature dictionary; combine several (and optionally your basemap) into a `FeatureCollection` and `json.encode` before rendering:

```typ
#let ring = geo-circle(center: (18.07, 59.33), radius: 500 / 111.32) // ~500 km
#render-map(json.encode((type: "FeatureCollection", features: (ring,))), (
  fill: "#4488ff", fill_opacity: 0.3, stroke: "#1144aa",
))
```

### flow maps (geo-arc)

`geo-arc(from, to, steps: 48, properties: (:))` builds a GeoJSON LineString following the great-circle (shortest) path between two `(lon, lat)` points — the building block of a flow / connection map. It interpolates on the sphere, so the arc curves correctly under any projection. Combine several into a `FeatureCollection` (typically over a basemap via `render-layers`):

```typ
#let sthlm = (18.07, 59.33)
#let arcs = ((-74, 40.7), (139.7, 35.7)).map(d => geo-arc(sthlm, d))
#render-layers((
  (data: read("world.json", encoding: none), fill: "#dfe6ee", stroke: "white"),
  (data: json.encode((type: "FeatureCollection", features: arcs)), fill: "none", stroke: "crimson"),
), projection: (type: "natural_earth"))
```

```typ
#let rates = ("SE-01": (rate: 90), "SE-03": (rate: 55))

#render-map(read("regions.json", encoding: none), (
  fill_scale: (property: "rate", type: "quantize", range: (..)),
), data: rates, key: "id")
```

## layers (render-layers)

`render-map` draws one dataset. To stack several — a basemap, boundaries, a data overlay, points — onto one shared projection and viewbox (so they line up), use `render-layers`:

```typ
#render-layers((
  // bottom layer first
  (data: read("regions.json", encoding: none), fill: "#e8e8e8", stroke: "white",
   graticule: (step: 10)),
  (data: read("cities.geojson", encoding: none), point_color: "crimson",
   point_radius_scale: (property: "pop", max_radius: 1.2)),
), projection: (type: "mercator"), width: 80%)
```

Each layer is a dictionary with `data` (the GeoJSON/TopoJSON) plus that layer's config keys. `projection` is shared across all layers. The viewbox defaults to the union of every layer's projected bounds (`viewbox-padding` controls the margin); pass `viewbox: (x, y, w, h)` to set it explicitly. Remaining arguments (e.g. `width`) go to each layer's image. The companion `map-bounds(code, projection: ..)` returns a dataset's projected `(x, y, w, h)`.

## build locally

```sh
cargo build --target wasm32-unknown-unknown --release
wasm-opt -O4 --enable-simd --enable-bulk-memory --enable-sign-ext \
  --enable-nontrapping-float-to-int --enable-mutable-globals --strip-debug \
  target/wasm32-unknown-unknown/release/mercator.wasm -o mercator/mercator.wasm
```

NB: `wasm-opt` is part of [binaryen](https://github.com/WebAssembly/binaryen).
