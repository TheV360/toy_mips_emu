.data
msg_usedisplay:
	.ascii  "Hello, emulator! :)\n"
	.asciiz "Random colors are being drawn at 0x010000.\n"

.text
main:
	
	# ( cd program/; ./build.sh lfsr_rainbow.asm ) && cargo run
	
	# Write introductory string.
	li $v0, 4 # syscall: print str
	la $a0, msg_usedisplay
	syscall
	
	# $t0: Start of display region. (0x01_0000)
	lui $t0, 0x01 # (the lower _0000 of it is done by this, too.)
	
	# $t1: End of display region. (0x01_0400)
	# start + 16px * 16px = 256 * 1 word per pixel = 1024 = 0x400
	addiu $t1, $t0, 0x0400
	
	# $t2: Next color, generated from LFSR.
	
	loop:
		# Make a color from the LFSR.
		jal next_lfsr
		addu $t2, $zero, $a0
		sll $t2, $t2, 16
		jal next_lfsr
		addu $t2, $t2, $a0
		
		# Write it to the display.
		sw $t2, 0($t0)
		
		# Move the display pointer forward a word.
		addiu $t0, $t0, 4
		
		# If we're past the end of the display,..
		bne $t0, $t1, skip_screen_dec
		nop # (delay slot)
		
		# ..reset the display's pointer.
		lui $t0, 0x01
	skip_screen_dec:
		# Could fall through, but also what if you didn't?
		j loop
		nop # (delay slot)
	
	# Quit
	addiu $v0, $zero, 17 # syscall: exit
	addiu $a0, $zero, 0
	syscall

next_lfsr: ##########################
# implements a 16-bit LFSR, adapted #
# from C code on Wikipedia. lol     #
#===================================#
# arguments:    ( $a0: LFSR state ) #
#-----------------------------------#
# returns:      ( $a0: LFSR state ) #
#-----------------------------------#
# clobbers: $t4, $t5                #
#####################################

	# bit = ( LFSR state
	# (original assignment is mixed into first xor instruction.)

	# ^ (LFSR state >> 2)
	srl $t5, $a0, 2
	xor $t4, $a0, $t5
	
	# ^ (LFSR state >> 3)
	srl $t5, $a0, 3
	xor $t4, $t4, $t5
	
	# ^ (LFSR state >> 5) )
	srl $t5, $a0, 5
	xor $t4, $t4, $t5
	
	# & 1;
	andi $t4, $t4, 1
	
	# new LFSR state = (LFSR state >> 1)
	srl $a0, $a0, 1
	
	# ( bit << 15 )
	sll $t4, $t4, 15
	# lhs | rhs;
	or $a0, $a0, $t4

	jr $ra
	nop
