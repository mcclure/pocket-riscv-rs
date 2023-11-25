This is a repo meant to host Rust programs for agg23's [Pocket RISC-V](https://github.com/agg23/openfpga-litex) platform. While Rust *can* be built out of the openfpga-litex repo directly, this repo references openfpga-litex as a git submodule so that a single piece of Rust code can be easily tested with different versions of openfpga-litex.

This commit is sort of a dummy, in case there's some horrible reason why I need at least one commit in the repo which does not have the openfpga-litex submodule.

If you wish to fork this, make sure to change the app name in Cargo.toml (it gets built into the application) and the license below (assuming do not wish to release as public domain).

# Usage

See [run.txt](run.txt)

# License

The Rust code in this directory is written by Andi McClure <andi.m.mcclure@gmail.com> and is intended as example code. It is available under Creative Commons Zero (https://creativecommons.org/publicdomain/zero/1.0/legalcode), in other words, it is public domain. If you substantially reuse the code, a credit would be appreciated, but this is not legally required.
