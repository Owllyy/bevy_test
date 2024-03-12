cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
          --out-dir ./out/ \
          --out-name "SpaceMerge" \
          ./target/wasm32-unknown-unknown/release/SpaceMerge.wasm