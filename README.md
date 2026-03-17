# sdfer

[![CI](https://github.com/hmeyer/sdfer/actions/workflows/ci.yml/badge.svg)](https://github.com/hmeyer/sdfer/actions/workflows/ci.yml)

sdfer aims to become a script based CAD.
It runs in the browser, using WASM + WebGL + Shaders.
It is based on implicit functions as described in depth by [iq](https://iquilezles.org/articles/distfunctions/).

**[Live Demo](https://hmeyer.github.io/sdfer/)**

## Examples

During development I focused on creating platonics, archimedean and catalan solids.

![Icosidodecahedron](resources/icosidodecahedron.png)
![Pentagonalhexecontahedron edges](resources/pentagonalhexecontahedron_edges.png)

## Usage

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
trunk serve
```
