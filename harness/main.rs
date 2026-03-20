use wasmi::*;

struct HostState {
    args: Vec<u8>,
    result: Vec<u8>,
    memory: Option<Memory>,
}

const PROJECTIONS: &[&str] = &[
    r#"{"projection":{"type":"equirectangular"}}"#,
    r#"{"projection":{"type":"mercator"}}"#,
    r#"{"projection":{"type":"lambert_conformal_conic"}}"#,
    r#"{"projection":{"type":"albers_equal_area"}}"#,
    r#"{"projection":{"type":"robinson"}}"#,
    r#"{"projection":{"type":"orthographic"}}"#,
    r#"{"projection":{"type":"natural_earth"}}"#,
    r#"{"projection":{"type":"lambert_azimuthal_equal_area"}}"#,
    r#"{"projection":{"type":"gnomonic"}}"#,
    r#"{"projection":{"type":"wiechel"}}"#,
    r#"{"projection":{"type":"peirce_quincuncial"}}"#,
    r#"{"projection":{"type":"cassini"}}"#,
    r#"{"projection":{"type":"bonne"}}"#,
    r#"{"projection":{"type":"polyconic"}}"#,
    r#"{"projection":{"type":"azimuthal_equidistant"}}"#,
    r#"{"projection":{"type":"hammer"}}"#,
    r#"{"projection":{"type":"winkel_tripel"}}"#,
    r#"{"projection":{"type":"authagraph"}}"#,
];

const PROJECTION_NAMES: &[&str] = &[
    "equirectangular",
    "mercator",
    "lambert_conformal_conic",
    "albers_equal_area",
    "robinson",
    "orthographic",
    "natural_earth",
    "lambert_azimuthal_equal_area",
    "gnomonic",
    "wiechel",
    "peirce_quincuncial",
    "cassini",
    "bonne",
    "polyconic",
    "azimuthal_equidistant",
    "hammer",
    "winkel_tripel",
    "authagraph",
];

// --- Doc mode: replay all geo() calls from documentation.typ ---

const SMILEY: &[u8] = br#"{"type":"GeometryCollection","geometries":[{"type":"Polygon","coordinates":[[[9.5,5.0],[9.46,5.59],[9.35,6.16],[9.16,6.72],[8.9,7.25],[8.57,7.74],[8.18,8.18],[7.74,8.57],[7.25,8.9],[6.72,9.16],[6.16,9.35],[5.59,9.46],[5.0,9.5],[4.41,9.46],[3.84,9.35],[3.28,9.16],[2.75,8.9],[2.26,8.57],[1.82,8.18],[1.43,7.74],[1.1,7.25],[0.84,6.72],[0.65,6.16],[0.54,5.59],[0.5,5.0],[0.54,4.41],[0.65,3.84],[0.84,3.28],[1.1,2.75],[1.43,2.26],[1.82,1.82],[2.26,1.43],[2.75,1.1],[3.28,0.84],[3.84,0.65],[4.41,0.54],[5.0,0.5],[5.59,0.54],[6.16,0.65],[6.72,0.84],[7.25,1.1],[7.74,1.43],[8.18,1.82],[8.57,2.26],[8.9,2.75],[9.16,3.28],[9.35,3.84],[9.46,4.41],[9.5,5.0]]]},{"type":"Polygon","coordinates":[[[3.85,6.2],[3.81,6.41],[3.69,6.59],[3.51,6.71],[3.3,6.75],[3.09,6.71],[2.91,6.59],[2.79,6.41],[2.75,6.2],[2.79,5.99],[2.91,5.81],[3.09,5.69],[3.3,5.65],[3.51,5.69],[3.69,5.81],[3.81,5.99],[3.85,6.2]]]},{"type":"Polygon","coordinates":[[[7.25,6.2],[7.21,6.41],[7.09,6.59],[6.91,6.71],[6.7,6.75],[6.49,6.71],[6.31,6.59],[6.19,6.41],[6.15,6.2],[6.19,5.99],[6.31,5.81],[6.49,5.69],[6.7,5.65],[6.91,5.69],[7.09,5.81],[7.21,5.99],[7.25,6.2]]]},{"type":"LineString","coordinates":[[2.86,3.4],[3.06,3.18],[3.3,2.98],[3.55,2.81],[3.82,2.66],[4.1,2.55],[4.39,2.47],[4.7,2.42],[5.0,2.4],[5.3,2.42],[5.61,2.47],[5.9,2.55],[6.18,2.66],[6.45,2.81],[6.7,2.98],[6.94,3.18],[7.14,3.4]]}]}"#;

// (name, data_key, config_json)
// data_key: "world", "world_no_ant", "sweden", "smiley"
const DOC_CALLS: &[(&str, &str, &str)] = &[
    // Hero
    ("hero", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"orthographic","center_lat":45,"center_lon":10},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    // Quick start
    ("quick_start", "sweden", "{}"),
    // Inline smiley
    ("smiley", "smiley", "{}"),
    // Styling
    ("styling_teal", "sweden", r##"{"stroke":"white","stroke_width":0.01,"fill":"teal","fill_opacity":0.8,"point_color":"none"}"##),
    ("styling_yellow", "sweden", r##"{"stroke":"#333","stroke_width":0.08,"fill":"#f7fc0f","fill_opacity":0.2,"point_color":"none"}"##),
    // Viewbox
    ("viewbox", "sweden", r##"{"stroke":"black","stroke_width":0.02,"fill":"grey","fill_opacity":0.5,"viewbox":[15.0,-69.4,10.0,6.0],"point_color":"none"}"##),
    // Labels
    ("labels", "sweden", r##"{"stroke":"white","stroke_width":0.03,"fill":"steelblue","fill_opacity":0.8,"point_color":"none","label":"{name}","label_color":"black","label_font_size":0.25}"##),
    ("multi_labels", "sweden", r##"{"stroke":"white","stroke_width":0.03,"fill":"steelblue","fill_opacity":0.8,"point_color":"none","label":[{"text":"{name}","font_size":0.25,"color":"black"},{"text":"#{l_id}","font_size":0.15,"color":"red"}]}"##),
    // Points
    ("points", "sweden", r##"{"stroke":"black","stroke_width":0.02,"fill":"white","point_radius":0.15,"point_color":"red","label":"{point}","label_color":"red","label_font_size":0.6}"##),
    // Per-feature styling
    ("per_feature", "sweden", r##"{"stroke":"black","stroke_width":0.02,"fill":"{fill_color}","fill_opacity":0.9,"fill_pattern":"{pattern}","point_color":"none"}"##),
    // Graticule comparison
    ("ortho_no_grat", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"orthographic","center_lat":45,"center_lon":10}}"##),
    ("ortho_grat", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"orthographic","center_lat":45,"center_lon":10},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    // Tissot
    ("tissot", "world", r##"{"projection":{"type":"mercator"},"stroke":"#aaa","stroke_width":0.01,"fill":"none","graticule":{"step":30,"color":"#ddd","opacity":0.4,"width":0.2},"tissot":{"step":30,"radius":5,"fill":"red","fill_opacity":0.4,"stroke":"darkred","stroke_width":0.3}}"##),
    // All projections with graticule
    ("equirectangular", "world", r##"{"stroke":"white","stroke_width":0.05,"fill":"steelblue","fill_opacity":0.85,"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("mercator", "world", r##"{"stroke":"white","stroke_width":0.05,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"mercator"},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("cassini", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"cassini","central_meridian":0},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("lcc", "world_no_ant", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"lambert_conformal_conic","standard_parallel_1":30,"standard_parallel_2":60,"central_meridian":10},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("albers", "world_no_ant", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"albers_equal_area","standard_parallel_1":30,"standard_parallel_2":60,"central_meridian":10,"latitude_of_origin":40},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("bonne", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"viewbox":[-2.5,-2.5,5,5.5],"projection":{"type":"bonne","standard_parallel":45,"central_meridian":10},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("polyconic", "world_no_ant", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"viewbox":[-4,-3.5,8,7],"projection":{"type":"polyconic","central_meridian":10},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("robinson", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"robinson"},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("natural_earth", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"natural_earth"},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("hammer", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"hammer"},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("winkel_tripel", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"winkel_tripel"},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("laea", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"viewbox_padding":0.25,"projection":{"type":"lambert_azimuthal_equal_area","center_lat":45,"center_lon":10},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("gnomonic", "world_no_ant", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"gnomonic","center_lat":90,"center_lon":0},"graticule":{"step":15,"color":"red","opacity":0.5},"viewbox":[-3,-3,6,6]}"##),
    ("orthographic", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"orthographic","center_lat":45,"center_lon":10},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("azimuthal_equidistant", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"azimuthal_equidistant","center_lat":-90,"center_lon":0},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("wiechel", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"wiechel","center_lat":90,"center_lon":0},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("peirce_quincuncial", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"peirce_quincuncial"},"graticule":{"step":15,"color":"red","opacity":0.5}}"##),
    ("authagraph", "world", r##"{"stroke":"white","stroke_width":0.001,"fill":"steelblue","fill_opacity":0.85,"projection":{"type":"authagraph"},"graticule":{"step":10,"color":"red","opacity":0.5}}"##),
    // Combined example
    ("combined", "sweden", r##"{"stroke":"white","stroke_width":0.01,"fill":"{fill_color}","fill_opacity":0.8,"fill_pattern":"{pattern}","point_radius":0.15,"point_color":"magenta","label":[{"text":"{point}","font_size":0.4,"color":"black","font_family":"New Computer Modern"},{"text":"id: {l_id}","font_size":0.12,"color":"red"}],"projection":{"type":"mercator","central_meridian":16},"viewbox":[-6.4,-74.4,10,10],"graticule":{"step":2,"color":"#ccc","opacity":0.4,"width":0.3},"tissot":{"step":3.3,"radius":0.5,"fill":"red","fill_opacity":0.2,"max_lat":80}}"##),
];

fn setup(wasm_bytes: &[u8], fuel: bool) -> (Store<HostState>, Instance, TypedFunc<(i32, i32), i32>) {
    let mut config = Config::default();
    if fuel {
        config.consume_fuel(true);
    }
    let engine = Engine::new(&config);
    let module = Module::new(&engine, wasm_bytes).expect("failed to parse WASM module");

    let mut store = Store::new(
        &engine,
        HostState {
            args: Vec::new(),
            result: Vec::new(),
            memory: None,
        },
    );

    if fuel {
        store.set_fuel(u64::MAX).unwrap();
    }

    let mut linker = Linker::<HostState>::new(&engine);

    linker
        .func_wrap(
            "typst_env",
            "wasm_minimal_protocol_write_args_to_buffer",
            |mut caller: Caller<'_, HostState>, ptr: i32| {
                let mem = caller.data().memory.expect("memory not set");
                let data = caller.data().args.clone();
                mem.write(&mut caller, ptr as usize, &data)
                    .expect("failed to write args to guest memory");
            },
        )
        .unwrap();

    linker
        .func_wrap(
            "typst_env",
            "wasm_minimal_protocol_send_result_to_host",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                let mem = caller.data().memory.expect("memory not set");
                let mut buf = vec![0u8; len as usize];
                mem.read(&caller, ptr as usize, &mut buf)
                    .expect("failed to read result from guest memory");
                caller.data_mut().result = buf;
            },
        )
        .unwrap();

    let instance = linker
        .instantiate_and_start(&mut store, &module)
        .expect("failed to instantiate");

    let memory = instance
        .get_memory(&store, "memory")
        .expect("missing 'memory' export");
    store.data_mut().memory = Some(memory);

    let geo_func = instance
        .get_typed_func::<(i32, i32), i32>(&store, "geo")
        .expect("missing 'geo' export");

    (store, instance, geo_func)
}

fn call_geo(
    store: &mut Store<HostState>,
    geo_func: &TypedFunc<(i32, i32), i32>,
    geojson: &[u8],
    config: &[u8],
) -> Result<Vec<u8>, String> {
    {
        let state = store.data_mut();
        state.args.clear();
        state.args.extend_from_slice(geojson);
        state.args.extend_from_slice(config);
    }

    let code = geo_func
        .call(&mut *store, (geojson.len() as i32, config.len() as i32))
        .expect("WASM function trapped");

    let result = store.data().result.clone();
    if code == 0 {
        Ok(result)
    } else {
        Err(String::from_utf8_lossy(&result).into_owned())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut bench_iters: Option<usize> = None;
    let mut use_fuel = false;
    let mut doc_dir: Option<String> = None;
    let mut positional = Vec::new();

    for arg in args.iter().skip(1) {
        if arg == "--fuel" {
            use_fuel = true;
        } else if arg.starts_with("--bench") {
            if let Some(eq) = arg.find('=') {
                bench_iters = Some(arg[eq + 1..].parse().expect("--bench=N requires a number"));
            } else {
                bench_iters = Some(1);
            }
        } else if arg.starts_with("--doc") {
            if let Some(eq) = arg.find('=') {
                doc_dir = Some(arg[eq + 1..].to_string());
            } else {
                doc_dir = Some("examples/data".to_string());
            }
        } else {
            positional.push(arg.as_str());
        }
    }

    if doc_dir.is_some() && positional.is_empty() {
        eprintln!("Usage: harness <wasm_file> --doc[=data_dir] [--fuel]");
        std::process::exit(1);
    }

    if doc_dir.is_none() && positional.len() < 2 {
        eprintln!("Usage: harness <wasm_file> <geojson_file> [config_json] [--fuel] [--bench=N]");
        eprintln!("       harness <wasm_file> --doc[=data_dir] [--fuel]");
        eprintln!();
        eprintln!("  --fuel        Report WASM instruction count (deterministic)");
        eprintln!("  --bench=N     Run N iterations, report avg time");
        eprintln!("                Without config: benchmarks all 18 projections");
        eprintln!("  --doc[=DIR]   Replay all geo() calls from documentation.typ");
        eprintln!("                Default data dir: examples/data");
        std::process::exit(1);
    }

    let wasm_bytes = std::fs::read(positional[0]).expect("failed to read WASM file");

    if let Some(dir) = doc_dir {
        run_doc(&wasm_bytes, &dir, use_fuel);
        return;
    }

    let geojson_bytes = std::fs::read(positional[1]).expect("failed to read GeoJSON file");
    let explicit_config = positional.get(2).map(|p| std::fs::read(p).expect("failed to read config file"));

    if let Some(iters) = bench_iters {
        run_bench(&wasm_bytes, &geojson_bytes, explicit_config.as_deref(), iters, use_fuel);
    } else {
        run_single(&wasm_bytes, &geojson_bytes, explicit_config.as_deref(), use_fuel);
    }
}

fn run_single(wasm_bytes: &[u8], geojson: &[u8], config: Option<&[u8]>, fuel: bool) {
    let config = config.unwrap_or(b"{}");
    let (mut store, _instance, geo_func) = setup(wasm_bytes, fuel);

    let fuel_before = if fuel { store.get_fuel().unwrap() } else { 0 };
    let start = std::time::Instant::now();
    let result = call_geo(&mut store, &geo_func, geojson, config);
    let elapsed = start.elapsed();
    let fuel_used = if fuel { fuel_before - store.get_fuel().unwrap() } else { 0 };

    match result {
        Ok(svg) => {
            let out = String::from_utf8_lossy(&svg);
            println!("{out}");
            if fuel {
                eprintln!("OK ({:.3}ms, {} instructions)", elapsed.as_secs_f64() * 1000.0, fuel_used);
            } else {
                eprintln!("OK ({:.3}ms)", elapsed.as_secs_f64() * 1000.0);
            }
        }
        Err(err) => {
            eprintln!("ERROR: {err}");
            std::process::exit(1);
        }
    }
}

fn run_bench(wasm_bytes: &[u8], geojson: &[u8], config: Option<&[u8]>, iters: usize, fuel: bool) {
    let configs: Vec<(&str, Vec<u8>)> = if let Some(cfg) = config {
        vec![("custom", cfg.to_vec())]
    } else {
        PROJECTION_NAMES
            .iter()
            .zip(PROJECTIONS.iter())
            .map(|(name, cfg)| (*name, cfg.as_bytes().to_vec()))
            .collect()
    };

    let iters = iters.max(1);

    eprintln!("{:<30} {:>10} {:>10} {:>14}", "projection", "avg (ms)", "min (ms)", "instructions");
    eprintln!("{}", "-".repeat(68));

    let mut total_time = 0.0f64;
    let mut total_fuel = 0u64;

    for (name, cfg) in &configs {
        let (mut store, _instance, geo_func) = setup(wasm_bytes, fuel);

        let mut times = Vec::with_capacity(iters);
        let mut fuel_used = 0u64;

        for i in 0..iters {
            if fuel {
                store.set_fuel(u64::MAX).unwrap();
            }
            let fuel_before = if fuel { store.get_fuel().unwrap() } else { 0 };

            let start = std::time::Instant::now();
            let result = call_geo(&mut store, &geo_func, geojson, cfg);
            let elapsed = start.elapsed();

            if fuel && i == 0 {
                fuel_used = fuel_before - store.get_fuel().unwrap();
            }

            if let Err(err) = result {
                eprintln!("{:<30} ERROR: {}", name, err);
                break;
            }
            times.push(elapsed.as_secs_f64() * 1000.0);
        }

        if times.is_empty() {
            continue;
        }

        let avg = times.iter().sum::<f64>() / times.len() as f64;
        let min = times.iter().cloned().fold(f64::INFINITY, f64::min);
        total_time += avg;
        total_fuel += fuel_used;

        if fuel {
            eprintln!("{:<30} {:>10.3} {:>10.3} {:>14}", name, avg, min, fuel_used);
        } else {
            eprintln!("{:<30} {:>10.3} {:>10.3} {:>14}", name, avg, min, "-");
        }
    }

    eprintln!("{}", "-".repeat(68));
    if fuel {
        eprintln!("{:<30} {:>10.3} {:>10} {:>14}", "TOTAL", total_time, "", total_fuel);
    } else {
        eprintln!("{:<30} {:>10.3}", "TOTAL", total_time);
    }
}

fn run_doc(wasm_bytes: &[u8], data_dir: &str, fuel: bool) {
    let world = std::fs::read(format!("{data_dir}/world.json"))
        .expect("failed to read world.json");
    let world_no_ant = std::fs::read(format!("{data_dir}/world_no_antartica.json"))
        .expect("failed to read world_no_antartica.json");
    let sweden = std::fs::read(format!("{data_dir}/swedish_regions.json"))
        .expect("failed to read swedish_regions.json");

    // Single instance — matches Typst behavior (GeoJSON cache persists across calls)
    let (mut store, _instance, geo_func) = setup(wasm_bytes, fuel);

    eprintln!("{:<30} {:>10} {:>14}", "call", "ms", "instructions");
    eprintln!("{}", "-".repeat(58));

    let mut total_time = 0.0f64;
    let mut total_fuel = 0u64;

    for (name, data_key, config) in DOC_CALLS {
        let data: &[u8] = match *data_key {
            "world" => &world,
            "world_no_ant" => &world_no_ant,
            "sweden" => &sweden,
            "smiley" => SMILEY,
            _ => panic!("unknown data key: {data_key}"),
        };

        if fuel {
            store.set_fuel(u64::MAX).unwrap();
        }
        let fuel_before = if fuel { store.get_fuel().unwrap() } else { 0 };

        let start = std::time::Instant::now();
        let result = call_geo(&mut store, &geo_func, data, config.as_bytes());
        let elapsed = start.elapsed();

        let fu = if fuel { fuel_before - store.get_fuel().unwrap() } else { 0 };
        let ms = elapsed.as_secs_f64() * 1000.0;
        total_time += ms;
        total_fuel += fu;

        match result {
            Ok(_) => {
                if fuel {
                    eprintln!("{:<30} {:>10.3} {:>14}", name, ms, fu);
                } else {
                    eprintln!("{:<30} {:>10.3}", name, ms);
                }
            }
            Err(err) => {
                eprintln!("{:<30} ERROR: {}", name, err);
            }
        }
    }

    eprintln!("{}", "-".repeat(58));
    if fuel {
        eprintln!("{:<30} {:>10.3} {:>14}", format!("TOTAL ({} calls)", DOC_CALLS.len()), total_time, total_fuel);
    } else {
        eprintln!("{:<30} {:>10.3}", format!("TOTAL ({} calls)", DOC_CALLS.len()), total_time);
    }
}
