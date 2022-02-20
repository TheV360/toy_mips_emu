export RUSTFLAGS=--cfg=web_sys_unstable_apis
cargo build -p "mips_emu" --release --lib --target wasm32-unknown-unknown

wasm-bindgen target/wasm32-unknown-unknown/release/mips_emu.wasm \
	--out-dir www --no-modules --no-typescript

~/Documents/Apps/_Utilities_/binaryen/bin/wasm-opt www/mips_emu_bg.wasm \
	-O2 --fast-math -o www/mips_emu_bg.wasm

( cd www && python server.py )
