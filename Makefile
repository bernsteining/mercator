WASM_TARGET = target/wasm32-unknown-unknown/release/mercator.wasm
WASM_OUT = mercator/mercator.wasm
WASM_PKG = $(HOME)/.local/share/typst/packages/local/mercator/0.1.1/mercator.wasm

.PHONY: build doc harness clean

build:
	cargo build --target wasm32-unknown-unknown --release
	wasm-opt -O3 --enable-simd --enable-bulk-memory --enable-sign-ext --enable-nontrapping-float-to-int --enable-mutable-globals --enable-multivalue --traps-never-happen --fast-math --closed-world --directize --inline-functions-with-loops --converge $(WASM_TARGET) -o $(WASM_OUT)
	cp $(WASM_OUT) $(WASM_PKG)
	@ls -lh $(WASM_OUT)

doc: build
	typst compile examples/documentation.typ examples/documentation.pdf --root .

harness:
	cargo build --release --manifest-path harness/Cargo.toml

clean:
	cargo clean
	cargo clean --manifest-path harness/Cargo.toml
