# mercator-harness

wasmi-based test harness for the mercator WASM plugin. Runs the `geo()` function outside of Typst with timing, fuel metering, and per-projection benchmarking.

## Build

```
make harness
```

## Usage

```
harness <wasm_file> <geojson_file> [config_json] [--fuel] [--bench=N]
harness <wasm_file> --doc[=data_dir] [--fuel]
```

SVG output goes to stdout, diagnostics to stderr.

## Examples

### Single call

```bash
# Default config (equirectangular, no styling)
harness/target/release/harness mercator/mercator.wasm examples/data/world.json > output.svg

# With a config file
echo '{"projection":{"type":"robinson"}}' > /tmp/cfg.json
harness/target/release/harness mercator/mercator.wasm examples/data/world.json /tmp/cfg.json > output.svg

# With instruction count
harness/target/release/harness mercator/mercator.wasm examples/data/world.json --fuel > /dev/null
# OK (675.123ms, 189802551 instructions)
```

### Benchmark all projections

```bash
# 3 iterations per projection, with instruction counts
harness/target/release/harness mercator/mercator.wasm examples/data/world.json --bench=3 --fuel
```

Output:
```
projection                       avg (ms)   min (ms)   instructions
--------------------------------------------------------------------
equirectangular                   467.722    290.755      190356340
mercator                          476.225    321.032      193614995
...
authagraph                        751.743    592.686      268107558
--------------------------------------------------------------------
TOTAL                            8783.883                3612470519
```

### Benchmark a specific config

```bash
echo '{"projection":{"type":"authagraph"},"graticule":{"step":15}}' > /tmp/cfg.json
harness/target/release/harness mercator/mercator.wasm examples/data/world.json /tmp/cfg.json --bench=5 --fuel
```

### Replay documentation.typ

Replays all 32 `geo()` calls from `examples/documentation.typ` using a single WASM instance (matching Typst's caching behavior).

```bash
harness/target/release/harness mercator/mercator.wasm --doc=examples/data --fuel
```

Default data dir is `examples/data` if omitted:

```bash
harness/target/release/harness mercator/mercator.wasm --doc --fuel
```

## Flags

| Flag | Description |
|---|---|
| `--fuel` | Report WASM instruction count (deterministic, reproducible) |
| `--bench=N` | Run N iterations, report avg/min time per projection |
| `--doc[=DIR]` | Replay all geo() calls from documentation.typ |

Without `--bench`, runs a single call and prints SVG to stdout.

Without a config file, `--bench` automatically tests all 18 projections.

## Comparing code changes

Instruction counts are deterministic — they don't vary between runs. To measure the impact of a change:

```bash
# Before
make build && harness/target/release/harness mercator/mercator.wasm examples/data/world.json --bench=1 --fuel 2> before.txt

# Make changes, rebuild
make build && harness/target/release/harness mercator/mercator.wasm examples/data/world.json --bench=1 --fuel 2> after.txt

diff before.txt after.txt
```
