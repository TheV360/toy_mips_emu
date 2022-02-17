.data
msg_usedisplay:
	.ascii  "Hello, emulator! :)\n"
	.asciiz "A rainbow is being drawn at 0x010000.\n"

# I wrote this by hand.
# 0 for none, +127 for halfway, -1 for all
funny_colors:
	##### BBBB  GGGG  RRRR, space
	.align 1
	.byte   +0,   +0,   -1
	.align 1
	.byte   +0, +127,   -1
	.align 1
	.byte   +0,   -1,   -1
	.align 1
	.byte   +0,   -1, +127
	.align 1
	.byte   +0,   -1,   +0
	.align 1
	.byte +127,   -1,   +0
	.align 1
	.byte   -1,   -1,   +0
	.align 1
	.byte   -1, +127,   +0
	.align 1
	.byte   -1,   +0,   +0
	.align 1
	.byte   -1,   +0, +127
	.align 1
	.byte   -1,   +0,   -1
	.align 1
	.byte +127,   +0,   -1
funny_colors_end:

.text
	.include "common_macros.asm"
	
	print_str (msg_usedisplay)
	
	# First, I want to draw a rainbow background.
	
	li $t0, 0x010000
	li $t1, 0
	la $t3, 0x010400 # start + 16px * 16px = 256 * 1 word per pixel = 1024 = 0x400
	loop:
		lw $t2, funny_colors($t1)
		sw $t2, 0($t0)
		
		# Move both pointers forward a word.
		addiu $t0, $t0, 4
		addiu $t1, $t1, 4
		
		# If we're past the end of the colors list...
		blt $t1, 48, skip_color_index_dec
		# 12 colors * 4 bytes per color = 48 bytes
		
		# ...go back to the beginning!
		# Makes a rainbow!!!
		li $t1, 0
	skip_color_index_dec:
		bne $t0, $t3, skip_screen_dec
		
		li $t0, 0x010000
	skip_screen_dec:
		j loop
	
	exit_with_code (0)
