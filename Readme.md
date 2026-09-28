# ofxKurbo

An OpenFrameworks wrapper for [Kurbo](https://github.com/linebender/kurbo) providing a set of geometrical functions on cartesian planar 2D paths (or shapes).  
Kurbo is a modern, high-performance 2D curve geometry library in Rust, with a strong focus on Bezier paths.  

Where other libre geometric algebra libraries tend to "flatten" (sampling & approxitions) data in the process, Kurbo uses modern algorithms that resultg in way cleaner and more accurate results.

## Project history

ofxKurbo is a continuation of [ofxBezierRs](https://github.com/Daandelange/ofxBezierRs) _(with some differences)_. Bezier-rs was used in the excellent [Graphite Editor](https://editor.graphite.rs) and has mainly been replaced by Kurbo. In the migration process, some Bezier-rs algos have moved to Kurbo too and some missing parts have moved to the [Graphite::vector_types](https://github.com/GraphiteEditor/Graphite/tree/master/node-graph/libraries/vector-types). Graphite now also integrates [Linesweeper](https://radicle.network/nodes/iris.radicle.network/rad%3Az2jbcL8tV3Fqqqk1yPyGkDTjgaTfH), providing boolean operations and path cleaning. ofxKurbo glues together various functions from all these libraries to provide some of the bezier algorithms used within Graphite. It also provides some utilities to get cleaner output from it and facilitate usage with OpenFrameworks (or C++) applications.

<!-- **Demo :**  
![ofxKurbo Demo](https://raw.githubusercontent.com/Daandelange/ofxBezierRs/main/ofxBezierRs-proof-of-concept.gif)  
[Watch a video of the example](https://www.youtube.com/watch?v=NiYp7FLi8xA). -->

## Early state

Kurbo & Linesweeper provide plenty of functions. Right now, consider this addon as a proof-of-concept for using Kurbo with C++ code.  
Smooth Openframeworks integration is yet to be done.

### Path offsetting details

I made this specially for its bezier path offsetting capabilities.  
Other libraries (Livarot, Clipper2) tend to flatten the the offset curve to polygons, then eventually use curve fitting algorithms to obtain bezier data as output.
Kurbo ports the logics described in [Pomax's excellent research work](https://pomax.github.io/bezierinfo/), which is more modern and generally results in both cleaner & more precise Bezier data output.

## Requirements

Tested on `osx + OF 0.12` and `linux + OF 0.12` in `c++17`.  
Rust installed for building the library.

## Geometrical functions

- [x] Segment based vector data _(like SVG and ofPath)_
- [x] Shape offset
- [x] Shape outline
- [x] Shape rotation
- [x] Reversing winding direction
- [x] Computing the bounding box of a shape
- [x] Shape hit testing
- [x] Inflections
- [ ] Find shape self intersections
- [x] Evaluate a point on the shape (t-value: linear & euclidean)
- [x] Normal from t-value
- [x] Tangent from t-value
- [x] Curvature from t-value
- [x] Find closest point on shape (projection)
- [x] Boolean path operations.
- [ ] Morphing
- [ ] Extrusion

_Checked ones are implemented._

## Shapes

The shape object is close to the underlying one used in kurbo.  
Unlike ofxBezierRs, which is **anchor-based**, ofxKurbo is **segment-based**, more like the ofPath object & SVG commands.  
Note: Closed shape operations work better when handles are winded clockwise.

## Implementation notes

Kurbo is a library written in Rust which can build a C compatible library.

ofxKurbo relies on 3 essential components :
- `kurbo-ffi` : A rust crate to build a library that provides API for communicating with C (an `ffi` in Rust's terms).
- `kurbo-ffi.h` : C++ bindings for the compiled library, generated from the rust crate.
- `ofxKurbo.h` : Some glue to make it work better with OF.

## Usage

See the examples for more advanced usage demos.

```cpp
#include "ofxKurbo.h"

int main() {
    // Create a new path
    kurboBezPath* path = kurbo_path_create(nullptr);
    
    // Todo !
    
    // Destroy path handles when done
    kurbo_path_destroy(path);

    return 0;
}
```

### ofxImGui

There's a set of ImGui helpers available, to opt-in, define `OFXBEZRS_DEFINE_IMGUI_HELPERS`. It will automatically be enabled with the standard `ofxAddons_ENABLE_IMGUI`.

## Development

To build a new library binary for your platform, make sure that you have [Rust](https://www.rust-lang.org/tools/install) installed.
- `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

Build the library :
- `cd ./libs/kurbo-ffi/`
- `cargo build --release`
- Copy lib files to `ofxKurbo/libs/kurbo-ffi/lib/PLATFORM/libkurbo_ffi.a|dylib|so` (PLATFORM: osx, linux64, etc.)

Generate bindings :
- Install or update cbindgen : `cargo install --force cbindgen`
- `cd ./libs/kurbo-ffi/`
- Run : `cbindgen --config ./cbindgen.toml --crate kurbo-ffi --output include/kurbo-ffi.h`.
- Copy `kurbo-ffi.h` to `libs/kurbo-ffi/include`.

You can run the above instructions automatically :
- `cd ./libs/kurbo-ffi && ./Build.sh`


## License

The [kurbo crate](https://crates.io/crates/kurbo) is licensed [MIT](https://github.com/linebender/kurbo/blob/main/LICENSE-MIT) or [Apache-2.0](https://github.com/linebender/kurbo/blob/main/LICENSE-APACHE) and is made by the [LineBender](https://linebender.org) organisation. Some capabilities were contributed by the team behind [Graphite.rs](https://editor.graphite.rs) which is licensed [MIT](https://github.com/GraphiteEditor/Graphite/blob/master/LICENSE-MIT) or [Apache-2.0](https://github.com/GraphiteEditor/Graphite/blob/master/LICENSE-APACHE).  
[Linesweeper](https://radicle.network/nodes/iris.radicle.network/rad%3Az2jbcL8tV3Fqqqk1yPyGkDTjgaTfH) is licensed [MIT](https://radicle.network/nodes/iris.radicle.network/rad:z2jbcL8tV3Fqqqk1yPyGkDTjgaTfH/tree/LICENSE-MIT) or [Apache-2.0](https://radicle.network/nodes/iris.radicle.network/rad:z2jbcL8tV3Fqqqk1yPyGkDTjgaTfH/tree/LICENSE-APACHE).  
`ofxKurbo` and `kurbo-ffi` are [MIT](https://github.com/Daandelange/ofxKurbo/blob/main/License.md) and made by [Daan de Lange](https://daandelange.com/).

