.text
	# hmmm a lotta these clobber register values, which
	# amy be unexpected...
	
	# Print the contents of a register.
	.macro print_reg (%reg)
		la $a0, (%reg)
		li $v0, 34 # syscall: print hex
		syscall
	.end_macro
	
	# Print an integer.
	.macro print_int (%val)
		la $a0, (%val)
		li $v0, 1 # syscall: print int
		syscall
	.end_macro
	
	# Print a character.
	.macro print_chr (%chr)
		li $v0, 11 # syscall: print chr
		li $a0, %chr
		syscall
	.end_macro
	
	# Print a string at the predefined label.
	.macro print_str (%label)
		li $v0, 4 # syscall: print str
		la $a0, %label
		syscall
	.end_macro
	
	# Read a number.
	# You could do without this
	# quite easily, but I like macros :)
	.macro read_int_to_v0 ()
		li $v0, 5 # syscall: read int
		syscall
	.end_macro
	
	# Print a string *very lazily*
	# Use when debugging, or when you
	# don't need to say things multiple times.
	.macro print_str_lazy (%str)
	.data
		lazy_str: .asciiz %str
	.text
		print_str (lazy_str)
	.end_macro
	
	# End the program with the specified exit code.
	.macro exit_with_code (%code)
		li $v0, 17 # syscall: exit
		la $a0, %code
		syscall
	.end_macro
	
	# XOR Swap two registers
	# doesn't clobber anything, it's so cool.
	.macro xor_swap (%reg_a, %reg_b)
		xor %reg_a, %reg_a, %reg_b # a = a xor b
		xor %reg_b, %reg_b, %reg_a # b = b xor a
		xor %reg_a, %reg_a, %reg_b # a = a xor b
	.end_macro
	
	# ez flow control
	
	.macro for_step (%it, %start, %end, %step, %body_macro)
		li %it, %start
	loop:
		%body_macro ()
		add %it, %it, %step
		ble %it, %end, loop
	.end_macro
	.macro for_pets (%it, %start, %end, %step, %body_macro)
		li %it, %start
	loop:
		%body_macro ()
		add %it, %it, %step
		bgt %it, %end, loop
	.end_macro
	.macro for (%it, %start, %end, %body_macro)
		for_step (%it, %start, %end, 1, %body_macro)
	.end_macro
