#!/bin/bash
set -euo pipefail

# This script works great!!! if you're me
JAVA_EXEC_PATH="java"
MARS_JAR_PATH="$HOME/Downloads/mars/Mars4_5.jar"
# Adjust these variables accordingly.

"$JAVA_EXEC_PATH" -jar "$MARS_JAR_PATH" \
	nc \
	a ae1 \
	mc CompactTextAtZero \
	dump .text Binary out.text.bin \
	dump .data Binary out.data.bin \
	"$1"

# ( cd program; ./build.sh bitmap_example_simple.asm ) && cargo run --release
