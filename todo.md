# MARS 2 (arrogant name)

I bounce back and forth between "actual emulator core development" and "UI development" because [it keeps me going](https://twitter.com/V360dev/status/1540797269632483328).

TODO:

## Small Stuff

- currently i made the text representation wayy too wide, and it currently only shows the first 4 bytes of a string. that should be fixed.
	- i can vary how many labels per row there are across representation views, that's fine to do. i just can't vary # of labels per row across individual rows.
- branch delay slot doent' exist.
- exceptions are incomplete
- work on memory paging `get_slice` - it probably needs to just become an iterator or something..
- allow writes across pages with `set_slice`
- complete `MemoryPlaces` and support loading some kind of executable file that includes `.data` and `.text` and everything nice.
- ~~export `INSTRUCTION_SIZE` that's just `WORD_BYTES` reexported lol~~
- "scroll to" support in new "infinite list" memory view
	- a "jump to exact address" thing too.
- oof, switching representations loses my place in the memory view.

## BIG FEATURES

- **heat map**  
	(highlights instructions that have been executed and registers/memory as they are modified)
	- get info about current instruction (registers used, how they're used, etc.)
	- configurable things to blink
		- memory: ☐read ☑write ☑execute
		- registers: ☐read ☑write
	- configurable "decay" time / colors
	- what'd be a good representation for this?
		- honestly maybe just allocate a buffer large enough (i.e., however many instructions can be executed in the decay time)
		- that sucks and'd require some calculation. what if i just allowed 128 highlights, and the color varied based on where in the circle buffer they are (about to be overwritten: red; just executed: yellow)
	- show in virtual display?
	- a "zoomed out" view of memory could complement this.
- **step backwards**
	- CPU step could optionally return way to undo each instruction with the instruction (think "undo steps", command pattern from Game Prog. Patterns)
	<!-- - add the button to the ui. what color could it be? what's the icon it has, what animation does it do when you click on it odes it make a  sound? ?  does it blow up? [does it hurt you??](youtu.be/rvg2ZsJurNM?t=196s) ironic bikeshedding to make fun of myself -->
- **basic assembler**
	- with another example utility that does basic "jump to this label" BS
	- i just like the idea of dogfooding my computer emulator.
- **double-click to edit**
	- registers
		- no revert button for these -- they'd cause the program to deviate waaay too far from their original state to be worth "reverting" without an entire-- okay.. wait
	- instructions
		- and these *should* have a revert button, because programs don't modify themselves *too often*...
			- but i'm talking about user action causing a program to change. self-modifying programs probably won't have this "revert" feat because it'd be a PitA to develop, and the Reset button already does what you want anyway.
			- mmm. the good part of  having immediate mode GUI is that i can easily conditionally have buttons. this could just check against  the original file (that i simply keep around)
				- having an "infinite list" just give me a range for the rows i need to render is... Extremely cool design, lol.
		- this would of course hook into the "basic assembler" i mentioned earlier. (it'd be funny to allow you to also double-click the disassembly)
	- egui woes
		- it's not easy to add a right-click menu to any old thing..
		- i'm gonna have to store this "edit" buffer somewhere - maybe it'll be fine to have one per window? (can't share it because what if someone -- well i wonder if i can cancel the text entry thing if the window loses focus, and re-use that buffer for the other window)
			- oh my goddd stop overcomplicating
		- HOW THE H\*\*L DOSE EGUI WORK...
- **input/output**  
	i've really been neglecting that `syscall` at the beginning of my sample program for waaay too long. i'd like to have a terminal view like the bottom panel of MARS.
	- combined input/output like the typical terminal
	- mumble something about a version of the emulator that runs in the terminal
