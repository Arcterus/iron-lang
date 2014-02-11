RUSTC      := rustc
RUSTCFLAGS := --opt-level=3

all: iron

iron: src/iron.rs src/*.rs
	$(RUSTC) $(RUSTCFLAGS) -o $@ $<

clean:
	rm -rf iron

