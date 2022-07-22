.text # <-- doesn't do anything yet.
# todo: strip directives (like .text) or better yet, parse them
	addi $t0 $t0 1
	add $t0 $t0 $t0
	j 0x0
