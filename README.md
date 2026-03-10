# mercator

![logo](examples/data/logo.png)

Mercator is a Typst plugin to render GeoJSON and TopoJSON as SVG maps.

# usage

```typst
#import "@preview/mercator:0.1.2": *

#let world = read("examples/data/worldmap.json", encoding: "utf8")

#render-map(world, json.encode((
  projection: (
    type: "orthographic",
    center_lat: 45,
    center_lon: 10,
  ),
  graticule: (step: 15),
)), width: 100%)
```


# documentation

Check [examples/documentation.pdf](examples/documentation.pdf), it covers all the features with examples.


# config options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `stroke` | string | `"black"` | Stroke color. Supports `{property_name}` interpolation. |
| `stroke_width` | float | `0.05` | Stroke width |
| `fill` | string | `"red"` | Fill color. Supports `{property_name}` interpolation. |
| `fill_opacity` | float | `0.5` | Fill opacity |
| `fill_pattern` | string | none | `"hatched"`, `"crosshatched"`, or `"dotted"`. Supports `{property_name}`. |
| `point_radius` | float | `stroke_width * 5` | Radius for Point/MultiPoint geometries |
| `point_color` | string | same as `fill` | Point fill color. `"none"` hides points. Supports `{property_name}`. |
| `viewbox` | array | auto | Manual viewbox as `(x, y, width, height)` |
| `viewbox_padding` | float | `0.1` | Padding fraction around auto-computed viewbox |
| `label` | string or array | none | Label template: `"{name}"` or array of `{text, font_size, color}` objects |
| `label_color` | string | `"black"` | Default label color |
| `label_font_size` | float | `0.3` | Default label font size |
| `label_font_family` | string | `"Arial"` | Default label font family |
| `projection` | object | equirectangular | Projection config (see below) |
| `graticule` | object | none | Graticule overlay config (see below) |

## projections

| Type | Parameters |
|------|------------|
| `equirectangular` | _(default, no parameters)_ |
| `mercator` | `central_meridian` |
| `lambert_conformal_conic` | `standard_parallel_1`, `standard_parallel_2`, `central_meridian`, `latitude_of_origin` |
| `albers_equal_area` | `standard_parallel_1`, `standard_parallel_2`, `central_meridian`, `latitude_of_origin` |
| `robinson` | `central_meridian` |
| `orthographic` | `center_lat`, `center_lon` |
| `natural_earth` | `central_meridian` |

## graticule

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `step` | float | `15.0` | Degrees between grid lines |
| `color` | string | `"#ccc"` | Line color |
| `width` | float | `0.5` | Line width |
| `opacity` | float | `0.6` | Line opacity |

# build locally

```sh
cargo build --target wasm32-unknown-unknown --release
wasm-opt -O4 --enable-bulk-memory --strip-debug \
target/wasm32-unknown-unknown/release/mercator.wasm -o mercator/mercator.wasm
```
