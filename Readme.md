# ofxKurbo

An OpenFrameworks wrapper for [kurbo](https://github.com/linebender/kurbo) providing a set of geometrical functions on cartesian planar 2D paths and shapes.  
Kurbo is a modern, high-performance 2D curve geometry library in Rust, with a strong focus on Bezier paths.  
Like libKurbo is a continuation of libBezierRs (with some differences), ofxKurbo is a continuation of [ofxBezierRs](https://github.com/Daandelange/ofxBezierRs).

<!-- **Demo :**  
![ofxKurbo Demo](https://raw.githubusercontent.com/Daandelange/ofxBezierRs/main/ofxBezierRs-proof-of-concept.gif)  
[Watch a video of the example](https://www.youtube.com/watch?v=NiYp7FLi8xA). -->

## Early state

Kurbo provides plenty of functions. Right now, consider this addon as a proof-of-concept for using libKurbo with C++ code.  
Smooth Openframeworks integration is yet to be done.

### Path offsetting details

I made this for its bezier path offsetting capabilities.  
Other libraries (Livarot, Clipper2) tend to flatten the the offset curve to polygons, then use curve fitting algorithms to obtain bezier data as output.
Kurbo ports the logics described in [Pomax's excellent research work](https://pomax.github.io/bezierinfo/), which is more modern and generally results in cleaner Bezier data output.

## Requirements

Tested on `osx + OF 0.12` and `linux + OF 0.12` in `c++17`.

## Geometrical functions

- [x] Shape offset
- [x] Shape outline
- [x] Shape rotation
- [x] Reversing winding direction
- [x] Computing the bounding box of a shape
- [x] Shape hit testing
- [x] Inflections
- [ ] Find shape self intersections
- [x] Evaluate a point on the shape (t-value)
- [x] Normal from t-value
- [x] Tangent from t-value
- [x] Curvature from t-value
- [x] Find closest point on shape (projection)
- [ ] Boolean path operations.

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

```cpp
#include "ofxKurbo.h"

int main() {
    // Create a new path
    kurboBezPath* path = kurbo_path_create(nullptr);
    
    // Append some curve segments
    kurbo_path_append_move_to(path, to_kurboPos(glm::vec2(100, 100)));
    kurbo_path_append_curve_to(path, to_kurboPos(glm::vec2(100, 200)), to_kurboPos(glm::vec2(200, 200)), to_kurboPos(glm::vec2(200, 100)));
    kurbo_path_append_close(path);
    
    // Stroke the path (create an outline)
    kurboBezPath* stroked = kurbo_path_stroke(path, 10.0, kurboJoinType::Round, 4.0, kurboCapType::Round, kurboCapType::Round);
    
    // Retrieve resulting path elements
    kurboPathRaw rawData = kurbo_path_return_handle_data(stroked);
    
    // Use result in OpenFrameworks
    ofBeginShape();
    for (size_t i = 0; i < rawData.len; i++) {
        const KurboPathEl& el = rawData.data[i];
        if (el.tag == KurboPathElType::MoveTo) {
            ofVertex(to_glmVec2(el.p0));
        } else if (el.tag == KurboPathElType::CurveTo) {
            ofBezierVertex(to_glmVec2(el.p0), to_glmVec2(el.p1), to_glmVec2(el.p2));
        } else if (el.tag == KurboPathElType::ClosePath) {
            ofEndShape(true);
        }
    }
    ofEndShape(false);
    
    // Destroy path handles when done
    kurbo_path_destroy(path);
    kurbo_path_destroy(stroked);

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

The [kurbo crate](https://crates.io/crates/kurbo) is licensed [MIT](https://github.com/linebender/kurbo/blob/main/LICENSE-MIT) or [Apache-2.0](https://github.com/linebender/kurbo/blob/main/LICENSE-APACHE). The bezier-rs crate is made by the [LineBender](https://linebender.org) organisation and offsetting capabilities were contributed by the team behind [Graphite.rs](https://editor.graphite.rs).
`ofxKurbo` and `bezier-rs-ffi` are [MIT](https://github.com/Daandelange/ofxKurbo/blob/main/License.md) and made by [Daan de Lange](https://daandelange.com/).

