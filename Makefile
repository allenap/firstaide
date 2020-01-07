out := ../bin/$(shell uname -s).$(shell uname -m)/firstaide

$(out): target/release/firstaide
	mkdir -p $(@D) && cp -p $< $@

target/release/firstaide: FORCE
	cargo build --release

FORCE:
