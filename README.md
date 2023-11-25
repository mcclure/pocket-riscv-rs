This is a repo meant to host Rust programs for agg23's [Pocket RISC-V](https://github.com/agg23/openfpga-litex) platform. While Rust *can* be built out of the openfpga-litex repo directly, this repo references openfpga-litex as a git submodule (in `external/openfpga-litex`) so that a single piece of Rust code can be easily tested with different versions of openfpga-litex.

The code in this commit is a small brick breaking game ("minibreak") that shows off basic capabilities of the core: It has controls (left and right d-pad, select to pause), reads the system timer (for RNG), generates sound, and draws in the framebuffer.

If you wish to fork this, make sure to change the app name in Cargo.toml (it gets built into the application) and the license below (assuming do not wish to release as public domain). You may also prefer to remove the app-specific dependency "glam".

# Usage

See [run.txt](run.txt)

# Getting Started

To get started with openfpga-litex, make sure to notice the [README](external/openfpga-litex), [control.md](external/openfpga-litex/docs/control.md), and the [existing Rust examples](external/openfpga-litex/lang/rust/examples) in openfpga-litex; and the "build docs" command in [run.txt](run.txt) (most useful for the litex-pac and litex-openfpga crate docs, since litex-pac contains the Rust version of control.md).

Once you have built a `rust.bin`, you have two options for deployment: You can live upload to a running copy of the Pocket RISC-V core as described in [run.txt](run.txt), or you can create a new copy of the Pocket RISC-V core as described in the [Analogue docs](https://www.analogue.co/developer/docs/packaging-a-core) and include rust.bin as `boot.bin` in the `/Assets/.../common` directory.

# License

The Rust code in this directory is written by Andi McClure <<andi.m.mcclure@gmail.com>> (based on the openfpga-litex examples) and is intended as example code. It is available under [Creative Commons Zero](https://creativecommons.org/publicdomain/zero/1.0/legalcode), in other words, it is public domain. If you substantially reuse the code, a credit would be appreciated, but this is not legally required.

Code in submodules or crates, such as openfpga-litex, will of course have its own license.
