# cargo -q run --example mips_assembler -- program/friend.asm program/friend.text.bin
.text # <-- doesn't do anything yet.
# todo: strip directives (like .text) or better yet, parse them
	addi $t0, $t0, 1
	add $t1, $t0, $t0
	j 0x0
	add $t2, $t0, $t1 # runs if branch delay enabled
	add $t3, $t0, $t2 # should never run
