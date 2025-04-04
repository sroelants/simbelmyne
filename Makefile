EXE ?= Simbelmyne

openbench:
	/home/sam/.cargo/bin/cargo rustc --release -p simbelmyne -- -C target-cpu=native --emit link=$(EXE)
