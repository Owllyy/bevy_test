cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --no-typescript --target web \
          --out-dir ./out/ \
          --out-name "SpaceMerge" \
          ./target/wasm32-unknown-unknown/release/SpaceMerge.wasm

rm ../github_pages/SpaceMerge.js && rm ../github_pages/SpaceMerge_bg.wasm
mv out/SpaceMerge.js ../github_pages/ && mv out/SpaceMerge_bg.wasm ../github_pages/
cd ../github_pages
git add . && git commit -m "update" && git push
cd ../bevy_test