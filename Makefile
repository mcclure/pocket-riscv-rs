.PHONY: main

# So you can build this without setting nightly as your default
RUST_CHANNEL ?= +nightly

main:
	cargo $(RUST_CHANNEL) build --release
	cargo $(RUST_CHANNEL) objcopy --release -- -O binary rust.bin
