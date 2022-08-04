# MARS 2 (arrogant name)

I bounce back and forth between "actual emulator core development" and "UI development" because [it keeps me going](https://twitter.com/V360dev/status/1540797269632483328).

TODO:

## Small Stuff

<!-- - ☑ currently i made the text representation wayy too wide, and the hexdump column currently only shows the first 4 bytes of a string. that should be fixed.
	- i can vary how many labels per row there are across representation views, that's fine to do. i just can't vary # of labels per row across individual rows.
	- my idea is to just .. humm.. yeah,  i should probably just uh have it be 4 labels for words and 16 labels for text repr. . who cares about performance in the ui? the table is lazy evaluated anyway.-->
<!-- - ☑ branch delay slot doent' exist.
	- easy to implement, actually. just a bit awkward, mainly because it.. y'know. causes execution to linger before jumping.-->
- exceptions are incomplete
	- https://courses.missouristate.edu/KenVollmar/MARS/Help/MarsExceptions.html
- work on memory paging `get_slice` - it probably needs to just become an iterator or something..
	- dummy page to read all zeros? idk..
	- is it possible to make a word (not in awkward way) from this?
- allow writes across pages with `set_slice`
- complete `MemoryPlaces` and support loading some kind of executable file that includes `.data` and `.text` and everything nice.
	- required to be inside CPU crate, as CPU needs that information of where to jump to..
		- no it doesn't
<!-- - ☑ export `INSTRUCTION_SIZE` that's just `WORD_BYTES` reexported lol-->
- "scroll to" support in new "infinite list" memory view
	- a "jump to exact address" thing too.
- oof, switching representations loses my place in the memory view.
- multiplication and division instructions
	- oops
	- in MIPS revision 6 and up, these aren't in the instruction set?? so i really need to decide on what mips i'm talking about.
		- i'm just.. gonna implement MIPS revision 5 / lower.. revision 6 has a bunch of new instruction formats i do Not want to bother with lmao
		- also if i ever wanna try extending this to run some game system, it'd def fit in the MIPS rev 1..=5 range
		- playstation is a MIPS rev 1. could target that.
			- lol actually MIPS32 rev 1 != MIPS I. confusing!!
			- playstation is MIPS I, n64 is MIPS III (& MIPS I sometimes)

## Meta Stuff

<!-- - ☑ looking at the older version of the MIPS emulator up at my v360.dev domain, the dark theme text is white instead of gray and it looks 100x better imo. need to recreate that.
	- also fuck border radius all my homies hate border radius-->
- CI/CD pipeline? i think it's free for all public repositories.
	- keeps https://v360.dev/toy_mips_emu up-to-date
	- would just imply uh..
	- see https://github.com/emilk/egui/blob/master/.github/workflows/rust.yml
	- in turn, see https://github.com/actions-rs/cargo
	- i rely on the `wasm-opt` binary in https://github.com/WebAssembly/binaryen for good performance good size good

## BIG FEATURES

- **breakpoints..**
	- again will show up "in terms of a specific CPU"
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
	- this is going to need to be broken out into its own crate because Of Course My Scope Has Expanded
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
	- https://courses.missouristate.edu/KenVollmar/MARS/Help/SyscallHelp.html
