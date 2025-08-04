# Mercator
Mercator is a typst plugin to render GeoJSON as SVG in typst.

## Usage

````typst

// inline

#let config = json.encode((
  "stroke": "black",
  "stroke_width": 0.02,
  "fill": "green",
  "fill_opacity": 0.5,
  "viewbox": array((10.0, -70.0, 15.0, 15.0))))

#show raw.where(lang: "geojson"): it => mercator.render-map(it.text, config, width: 400pt)

```geojson
<GeoJSON string>
```

// from file

#let sweden = read(
  "swedish_regions.json",
  encoding: "utf8",
)

#let config2 = json.encode((
  "stroke": "black",
  "stroke_width": 0.05,
  "fill": "red",
  "fill_opacity": 0.5,
  "viewbox": array((10.0, -70.0, 15.0, 15.0))))

#figure(mercator.render-map(sweden, config2, height:400pt), caption: "Swedish regions")
````

# example

Check the source of [swedish_regions.typ](../example/swedish_regions.typ) & the result [swedish_regions.pdf](../example/swedish_regions.pdf).

