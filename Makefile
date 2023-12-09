# So you can build this without setting nightly as your default
RUST_CHANNEL ?= +nightly
RUST_ARGS ?=

.PHONY: main
unexport RUST_CHANNEL
unexport RUST_ARGS

main:
	cargo $(RUST_CHANNEL) build $(RUST_ARGS) --release
	cargo $(RUST_CHANNEL) objcopy $(RUST_ARGS) --release -- -O binary rust.bin
