This is a repo meant to host Rust programs for agg23's [Pocket RISC-V](https://github.com/agg23/openfpga-litex) platform. While Rust *can* be built out of the openfpga-litex repo directly, this repo references openfpga-litex as a git submodule (in `external/openfpga-litex`) so that a single piece of Rust code can be easily tested with different versions of openfpga-litex.

The code in this commit is a small screen-test app. It fills the screen with a pulsing color and occasional inverting flashes; pressing the Y button will cause it to invert while the button is held down, and pressing A, B or X will cause it to flash at various frequencies. **Warning**, this application can be very unpleasant to look at in any mode.

The purpose of this is to demonstrate vsync; there is not enough time during the vertical blanking interval to finish drawing the screen (try running with `--feature speed-debug`), so the screen will exhibit vertical "tearing". This demonstrates an interesting quirk of the Pocket RISC-V core: If you back up a little in the git commit history of this repo, you will find that there's a version which does not vsync, and the no-vsync version performs subjectively *better*. In my tests the no-vsync version will tear about 20% of the time whereas the vsync version tears always.

If you wish to fork this, make sure to change the app name in Cargo.toml (it gets built into the application) and the license below (assuming do not wish to release as public domain). You may also prefer to remove the app-specific dependency "glam".

# Usage

See [run.txt](run.txt)

# Getting Started

To get started with openfpga-litex, make sure to notice the [README](external/openfpga-litex), [control.md](external/openfpga-litex/docs/control.md), and the [existing Rust examples](external/openfpga-litex/lang/rust/examples) in openfpga-litex; and the "build docs" command in [run.txt](run.txt) (most useful for the litex-pac and litex-openfpga crate docs, since litex-pac contains the Rust version of control.md).

Once you have built a `rust.bin`, you have two options for deployment: You can live upload to a running copy of the Pocket RISC-V core as described in [run.txt](run.txt), or you can create a new copy of the Pocket RISC-V core as described in the [Analogue docs](https://www.analogue.co/developer/docs/packaging-a-core) and include rust.bin as `boot.bin` in the `/Assets/.../common` directory.

# License

The Rust code in this directory is written by Andi McClure <<andi.m.mcclure@gmail.com>> (based on the openfpga-litex examples) and is intended as example code. It is available under [Creative Commons Zero](https://creativecommons.org/publicdomain/zero/1.0/legalcode), in other words, it is public domain. If you substantially reuse the code, a credit would be appreciated, but this is not legally required.

Code in submodules or crates, such as openfpga-litex, will of course have its own license.
