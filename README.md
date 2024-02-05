This is a test app to demonstrate sprite drawing in the openfpga-litex (Pocket RISC-V) core. All drawing is done in CPU.

It will draw 2 sprites to begin with. You can select a sprite using LEFT and RIGHT d-pad and change its image with UP and DOWN d-pad. To add additional sprites, press RIGHT many times. To check the currently selected sprite, press L. To halt the selected sprite's automatic movement, press X. To move the selected sprite manually, hold L and use the d-pad. To reset the selected sprite to upper left, press A. To delete the selected sprite, press B.

This example uses some mildly fancy features to get performance:

- PNGs are decoded in build.rs and embedded into the executable.
- Drawing is double-buffered.
- Only rectangles which have changed are updated frame to frame.

In my testing, I can get about 13 sprites before I start missing frame deadlines (as measured by speed-debug, see [run.txt](run.txt)). I think I can bump that up around 2x with some optimizations.

# Usage

See [run.txt](run.txt)

# Getting Started

To get started with openfpga-litex, make sure to notice the [README](external/openfpga-litex), [control.md](external/openfpga-litex/docs/control.md), and the [existing Rust examples](external/openfpga-litex/lang/rust/examples) in openfpga-litex; and the "build docs" command in [run.txt](run.txt) (most useful for the litex-pac and litex-openfpga crate docs, since litex-pac contains the Rust version of control.md).

Once you have built a `rust.bin`, you have two options for deployment: You can live upload to a running copy of the Pocket RISC-V core as described in [run.txt](run.txt), or you can create a new copy of the Pocket RISC-V core as described in the [Analogue docs](https://www.analogue.co/developer/docs/packaging-a-core) and include rust.bin as `boot.bin` in the `/Assets/.../common` directory.

# License

The images in this repository (the contents of `resource/`) are by [Miguel Sternberg](https://spookysquid.com). They are included as examples, and you are granted **no rights** to use them.

The rest (files outside `resource/` or `external/`) is written by Andi McClure <<andi.m.mcclure@gmail.com>>. It is available under the "MIT License":

	Copyright (C) 2024 Andi McClure

	Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

	The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

	THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

If this license is too restrictive for what you are doing, feel free to contact me.

You can find my other Analogue Pocket creations at [https://pocket.runhello.com/](https://pocket.runhello.com/).
