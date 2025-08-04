# mercator

Mercator is a typst plugin to render GeoJSON as SVG in typst.

# build locally

```sh
cargo build --target wasm32-unknown-unknown --release 
cp target/wasm32-unknown-unknown/release/mercator.wasm mercator/
cp -r mercator/* ~/.local/share/typst/packages/local/mercator/0.1.0/
```

# usage

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


```sh
typst compile mercator/example/example.typ
```

Check the source of [example.typ](mercator/example/example.typ) & the result [swedish_regions.pdf](mercator/example/example.pdf).

# todo 
* also parse points and city labels
* auto compute viewbox from rust