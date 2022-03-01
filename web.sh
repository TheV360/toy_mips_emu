set -eu

echo "Building"
export RUSTFLAGS=--cfg=web_sys_unstable_apis
cargo build -p "mips_emulator_gui" --release --lib --target wasm32-unknown-unknown

echo "Generating Bindings"
wasm-bindgen target/wasm32-unknown-unknown/release/mips_emulator_gui.wasm \
	--out-dir www --no-modules --no-typescript

echo "Optimizing WASM"
~/Documents/Apps/_Utilities_/binaryen/bin/wasm-opt www/mips_emulator_gui_bg.wasm \
	-O2 --fast-math -o www/mips_emulator_gui_bg.wasm

if [ $# == 1 ] && [ "$1" == "run" ]
then
	echo "Starting Local Server"
	( cd www && python server.py ) || exit 0
fi
