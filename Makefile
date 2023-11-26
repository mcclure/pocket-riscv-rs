.PHONY: main

# So you can build this without setting nightly as your default
RUST_CHANNEL ?= +nightly
RUST_FLAGS ?=

main:
	cargo $(RUST_CHANNEL) build $(RUST_FLAGS) --release
	cargo $(RUST_CHANNEL) objcopy $(RUST_FLAGS) --release -- -O binary rust.bin
