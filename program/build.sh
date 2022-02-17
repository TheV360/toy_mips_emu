# This script works great!!! if you're me
JAVA_EXEC_PATH="/c/Users/V360/Documents/Apps/jdk-17/bin/java"
MARS_JAR_PATH="/c/Users/V360/Documents/Apps/MARS/MARS_4.5_v.jar"
# Adjust these variables accordingly.

$JAVA_EXEC_PATH -jar $MARS_JAR_PATH \
	nc \
	a ae1 \
	mc CompactTextAtZero \
	dump .text Binary out.text.bin \
	dump .data Binary out.data.bin \
	$1

# ( cd program; ./build.sh bitmap_example_simple.asm ) && cargo run --release
