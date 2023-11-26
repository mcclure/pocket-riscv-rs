This is the source of a Rust program that makes glitchy and (hopefully) compellingly abrasive sounds and animations. It is distributed as "ot3" and is meant to run on agg23's [Pocket RISC-V](https://github.com/agg23/openfpga-litex) platform. It uses every button on the Analogue Pocket. **Warning:** This application is very loud, and can cause the entire screen to flash.

# Usage

See [run.txt](run.txt)

# License

This program is made available under the MIT license:

> (C) 2023 Andi McClure
>
> Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the “Software”), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
>
> The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
>
>
> THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

It makes use of hilbert_2d by Humberto Duarte and openfpga-litex by Adam Gastineau, which have MIT licenses in their submodules in [external/](external/).
