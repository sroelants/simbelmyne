EXE ?= Simbelmyne

openbench:
	cargo rustc --release -p simbelmyne -- -C target-cpu=native --emit link=$(EXE)
