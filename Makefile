EXE ?= Simbelmyne

openbench:
	cargo --release -p simbelmyne -- -C target-cpu=native --emit link=$(EXE)
