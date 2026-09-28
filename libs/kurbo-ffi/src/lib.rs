// Kurbo + Linesweeper + Graphite FFI Library for geometry operations.
// Kurbo wrapper made for usage from c++. Written in rust, it transpiles (using cbindgen) to c++ headers, the ones used for the resulting lib.
//
// Note:
// Comments that start with 3 slashes (`///`) end up in the generated C header file.

// Principle : Create some common/shared data structures (bezierShape and bezierPoint/Handle), then let rust do operations on them.
// This library also provides some glue & extra passes for obtaining clean offsets.
// Also provided : Some legacy bezier-rs compatibility functions (euclidean / linear evaluation).

// Used Libraries
// - Kurbo : for bezier geometry algebra
// - Linesweeper : for boolean operations & path cleaning
// - Graphite::vector_types : for curve offsetting

// Useful ressources :
// - Using a rust lib from within C++ : https://docs.rust-embedded.org/book/interoperability/rust-with-c.html
// - FFI principles : https://doc.rust-lang.org/nomicon/ffi.html
// - FFI mistakes explained : https://rust-unofficial.github.io/patterns/idioms/ffi/errors.html
// - The Rust FFI Omnibus : http://jakegoulding.com/rust-ffi-omnibus/objects/
// - CBindgen docs : https://docs.rs/crate/cbindgen/latest
// - A WASM FFI for bezier-rs : https://github.com/GraphiteEditor/Graphite/blob/master/website/other/bezier-rs-demos/wasm/src/subpath.rs
// - RUST FII with complex data types : http://kmdouglass.github.io/posts/complex-data-types-and-the-rust-ffi/
// - C++ Rust types : https://locka99.gitbooks.io/a-guide-to-porting-c-to-rust/content/features_of_rust/types.html

// Good to know : (for C++ programmers)
// - Box::new() : Rust handled memory passed to c++
// - #[repr(C)] : Use same layout in Rust & C
// - #[no_mangle] : No name mangling for C linker compatibility
// - extern "C" : Use C ABI instead of Rust ABI (calling convention).


use std::slice;
use std::cell::RefCell;
use std::ffi::{c_ulong, CString};
use std::os::raw::c_char;
use std::panic::{self, AssertUnwindSafe};
use std::ptr;
use kurbo::{BezPath, PathEl, Point, Rect, Shape, Stroke, Join, Cap, Line, QuadBez, CubicBez, PathSeg, ParamCurve, ParamCurveDeriv, ParamCurveNearest, ParamCurveArclen, ParamCurveExtrema};

type SizeTC = c_ulong;


/// - - - -
/// Error / panic-safety boundary stuff

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KurboStatus {
    Ok = 0,
    /// A required pointer argument was null.
    NullPointer = 1,
    /// An argument was structurally invalid (e.g. a negative length).
    InvalidArgument = 2,
    /// The Rust side RustPanick; the operation did not complete.
    RustPanick = 3,
}

thread_local! {
    // One slot per OS thread. If you call into kurbo-ffi from more than one thread, each thread sees only the error from its own last call.
    static LAST_ERROR: RefCell<Option<CString>> = RefCell::new(None);
}

fn set_last_error(msg: impl Into<Vec<u8>>) {
    let c = CString::new(msg)
        .unwrap_or_else(|_| CString::new("<error message contained an interior NUL byte>").unwrap());
    LAST_ERROR.with(|slot| *slot.borrow_mut() = Some(c));
}

/// Returns a pointer to the last error message set on the calling thread,
/// Returns null when no errors happened in the last command.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_last_error_message() -> *const c_char {
    LAST_ERROR.with(|slot| match &*slot.borrow() {
        Some(c) => c.as_ptr(),
        None => ptr::null(),
    })
}

fn report_status(out_status: *mut KurboStatus, status: KurboStatus) {
    if !out_status.is_null() {
        unsafe { *out_status = status; }
    }
}

/// FFI error catching helper. Runs `f` callback, catching panics, and writes a status code to `*out_status` (if not null)
/// Returns `default` if `f` fails, returning `Err` and setting the last error message for debugging.
fn ffi_call<R>(
    out_status: *mut KurboStatus,
    default: R,
    f: impl FnOnce() -> Result<R, (KurboStatus, String)>,
) -> R where R: std::panic::UnwindSafe {
    match panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(value)) => { report_status(out_status, KurboStatus::Ok); value }
        Ok(Err((status, msg))) => { set_last_error(msg); report_status(out_status, status); default }
        Err(payload) => {
            let msg = payload.downcast_ref::<&str>().map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "kurbo-ffi: unknown panic payload".to_string());
            set_last_error(msg);
            report_status(out_status, KurboStatus::RustPanick);
            default
        }
    }
}

// - - - -

// Helper for merging Vec[BezPath] to a single BezPath
fn merge_bezpath_vec(vec: &[BezPath]) -> BezPath {
    let mut combined = BezPath::new();
    for path in vec {
        for el in path {
            combined.push(el.clone());
        }
    }
    return combined;
}

/// - - - -
/// Shared data structs
/// Some common minimal data types to be shared between c++ and Rust, used to pass and retrieve data in-bewteen both.
/// - - - -

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum kurboJoinType {
    Miter,
    Round,
    Bevel,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum kurboCapType {
    Butt,
    Round,
    Square,
}

fn parse_join(join: kurboJoinType) -> Join {
    match join {
        kurboJoinType::Miter => Join::Miter,
        kurboJoinType::Round => Join::Round,
        kurboJoinType::Bevel => Join::Bevel,
    }
}

fn parse_cap(cap: kurboCapType) -> Cap {
    match cap {
        kurboCapType::Butt => Cap::Butt,
        kurboCapType::Round => Cap::Round,
        kurboCapType::Square => Cap::Square,
    }
}

// Shared position structure
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kurboPos {
    pub x: f64,
    pub y: f64,
}

impl kurboPos {
    pub fn new(x: f64, y: f64) -> Self { kurboPos { x, y } }
    pub fn to_point(&self) -> Point { Point::new(self.x, self.y) }
    pub fn from_point(p: Point) -> Self { kurboPos { x: p.x, y: p.y } }
}

/// Rectangle with min & max corners.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kurboRect {
    pub x0: f64, pub y0: f64, pub x1: f64, pub y1: f64,
}

impl kurboRect {
    pub fn to_rect(&self) -> Rect { Rect::new(self.x0, self.y0, self.x1, self.y1) }
    pub fn from_rect(r: Rect) -> Self { kurboRect { x0: r.x0, y0: r.y0, x1: r.x1, y1: r.y1 } }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KurboPathElType {
    MoveTo, LineTo, QuadTo, CurveTo, ClosePath,
}

/// Minimal custom SVG-like-path-command representation
//  Note: because `PathEl` ain't C-compatible, `KurboPathEl` mirrors this.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KurboPathEl {
    pub tag: KurboPathElType,
    pub p0: kurboPos, pub p1: kurboPos, pub p2: kurboPos,
}

// Copy-conversion helpers
impl KurboPathEl {
    pub fn to_path_el(&self) -> PathEl {
        match self.tag {
            KurboPathElType::MoveTo => PathEl::MoveTo(self.p0.to_point()),
            KurboPathElType::LineTo => PathEl::LineTo(self.p0.to_point()),
            KurboPathElType::QuadTo => PathEl::QuadTo(self.p0.to_point(), self.p1.to_point()),
            KurboPathElType::CurveTo => PathEl::CurveTo(self.p0.to_point(), self.p1.to_point(), self.p2.to_point()),
            KurboPathElType::ClosePath => PathEl::ClosePath,
        }
    }
    pub fn from_path_el(el: &PathEl) -> Self {
        match el {
            PathEl::MoveTo(p0) => KurboPathEl { tag: KurboPathElType::MoveTo, p0: kurboPos::from_point(*p0), p1: kurboPos::new(0.0, 0.0), p2: kurboPos::new(0.0, 0.0) },
            PathEl::LineTo(p0) => KurboPathEl { tag: KurboPathElType::LineTo, p0: kurboPos::from_point(*p0), p1: kurboPos::new(0.0, 0.0), p2: kurboPos::new(0.0, 0.0) },
            PathEl::QuadTo(p0, p1) => KurboPathEl { tag: KurboPathElType::QuadTo, p0: kurboPos::from_point(*p0), p1: kurboPos::from_point(*p1), p2: kurboPos::new(0.0, 0.0) },
            PathEl::CurveTo(p0, p1, p2) => KurboPathEl { tag: KurboPathElType::CurveTo, p0: kurboPos::from_point(*p0), p1: kurboPos::from_point(*p1), p2: kurboPos::from_point(*p2) },
            PathEl::ClosePath => KurboPathEl { tag: KurboPathElType::ClosePath, p0: kurboPos::new(0.0, 0.0), p1: kurboPos::new(0.0, 0.0), p2: kurboPos::new(0.0, 0.0) },
        }
    }
}

/// Minimal custom SVG-like-path representation shared between C++ and Rust
#[repr(C)]
pub struct KurboPathRaw {
    pub data: *const KurboPathEl,
    pub len: SizeTC,
}

/// Internal bezier data handle (Rust-owned)
// Opaque handle in C++, required for the FFI boundary and because a plain BezPath has no C-repr
pub struct KurboBezPathInternal {
    pub(crate) path: BezPath,
}

impl KurboBezPathInternal {
    fn new(path: BezPath) -> Self {
       KurboBezPathInternal { path }
   }
}

/// Creates a rust-owned internal handle from a KurboPathRaw (or an empty one if none passed)
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_create(elements_opt: Option<&KurboPathRaw>) -> *mut KurboBezPathInternal {
    if let Some(elements_raw) = elements_opt {
        if !elements_raw.data.is_null() {
            let elements_slice = unsafe { slice::from_raw_parts(elements_raw.data as *const KurboPathEl, elements_raw.len as usize) };
            let path: BezPath = elements_slice.iter().map(|el| el.to_path_el()).collect();
            return Box::into_raw(Box::new(KurboBezPathInternal::new(path)));
        }
    }
    Box::into_raw(Box::new(KurboBezPathInternal::new(BezPath::new())))
}

/// Helper for creating an internal handle from C++ arrays/vectors instead of a `KurboPathRaw`. (Used by `ofxKurboPath`)
/// Bulk-uploads `len` contiguous `KurboPathEl` values in one call and returns a freshly-owned handle.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_create_from_elements(
    data: *const KurboPathEl,
    len: SizeTC,
    out_status: *mut KurboStatus,
) -> *mut KurboBezPathInternal {
    ffi_call(out_status, ptr::null_mut(), || {
        if data.is_null() && len != 0 {
            return Err((KurboStatus::NullPointer, "kurbo_path_create_from_elements: data is null but len != 0".to_string()));
        }
        let path: BezPath = if data.is_null() {
            BezPath::new()
        } else {
            let elements_slice = unsafe { slice::from_raw_parts(data, len as usize) };
            elements_slice.iter().map(|el| el.to_path_el()).collect()
        };
        Ok(Box::into_raw(Box::new(KurboBezPathInternal::new(path))))
    })
}

/// Deep-clones a path, producing an entirely independent handle with its own Rust-owned BezPath.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_clone(path: *const KurboBezPathInternal, out_status: *mut KurboStatus) -> *mut KurboBezPathInternal {
    ffi_call(out_status, ptr::null_mut(), || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_clone: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        Ok(Box::into_raw(Box::new(KurboBezPathInternal::new(p_obj.path.clone()))))
    })
}

/// Destroys a handle created by any kurbo_path_* constructor.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_destroy(path: *mut KurboBezPathInternal) {
    if path.is_null() { return; }
    let _ = panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        let _ = Box::from_raw(path);
    }));
}

/// Bulk-reads every element of `path` into a Rust-owned buffer.
/// The caller MUST pass the returned KurboPathRaw to `kurbo_elements_free` once done.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_get_elements(path: *const KurboBezPathInternal, out_status: *mut KurboStatus) -> KurboPathRaw {
    ffi_call(out_status, KurboPathRaw { data: ptr::null(), len: 0 }, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_get_elements: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let mut elements: Vec<KurboPathEl> = p_obj.path.elements().iter().map(KurboPathEl::from_path_el).collect();
        elements.shrink_to_fit();
        let raw = KurboPathRaw { data: elements.as_ptr(), len: elements.len() as SizeTC };
        std::mem::forget(elements); // ownership transferred to the caller; reclaimed in kurbo_elements_free
        Ok(raw)
    })
}

/// Frees a buffer previously returned by kurbo_path_get_elements.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_elements_free(raw: KurboPathRaw) {
    if raw.data.is_null() || raw.len == 0 { return; }
    let _ = panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        // Reconstruct the exact Vec<KurboPathEl> so Rust's allocator frees it correctly.
        let _ = Vec::from_raw_parts(raw.data as *mut KurboPathEl, raw.len as usize, raw.len as usize);
    }));
}

// Retuns inner size for downloading path elements.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_get_elements_size(path: *const KurboBezPathInternal) -> SizeTC {
    if path.is_null() {
        return 0;
    }
    let p_obj = unsafe { &*path };
    let count = p_obj.path.elements().len();
    return count as SizeTC;
}

/// Writes the elements of `path` into a caller-owned buffer, writing at most `out_capacity` elements.
/// Returns the success state. No need to call `kurbo_elements_free`.
///   SizeTC needed = kurbo_path_get_elements_size(path);
///   std::vector<KurboPathEl> buf(needed);
///   kurbo_path_write_elements(path, buf.data(), buf.size(), &status);
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_write_elements(
    path: *const KurboBezPathInternal,
    out_data: *mut KurboPathEl,
    out_capacity: SizeTC,
    out_status: *mut KurboStatus,
) -> bool {
    ffi_call(out_status, false, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_write_elements: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let count = p_obj.path.elements().len();

        if count > 0 && out_capacity as usize >= count {
            if out_data.is_null() {
                return Err((KurboStatus::NullPointer, "kurbo_path_write_elements: out_data is null but out_capacity > 0".to_string()));
            }
            let out_slice = unsafe { slice::from_raw_parts_mut(out_data, count) };
            for (dst, el) in out_slice.iter_mut().zip(p_obj.path.elements().iter()) {
                *dst = KurboPathEl::from_path_el(el);
            }
            return Ok(true);
        }

        return Ok(false);
    })
}

/// Returns a stroked-outline
/// Non-mutating: `path` is left untouched.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_stroke(
    path: *const KurboBezPathInternal,
    width: f64,
    start_cap: kurboCapType,
    end_cap: kurboCapType,
    options: *const KurboOffsetOptions,
    out_status: *mut KurboStatus,
) -> *mut KurboBezPathInternal {
    ffi_call(out_status, ptr::null_mut(), || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_stroke: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let opts = if options.is_null() { KurboOffsetOptions::defaults() } else { unsafe { *options } };
        let join = parse_join(opts.join);

        let stroke = Stroke::new(width.abs()) // Undefined (&wrong) behaviour with negative width !
            .with_join(join)
            .with_miter_limit(opts.miter_limit)
            .with_start_cap(parse_cap(start_cap))
            .with_end_cap(parse_cap(end_cap));
        let stroked_path = kurbo::stroke(p_obj.path.clone(), &stroke, &kurbo::StrokeOpts::default(), opts.stroke_tolerance);

        // Same cleanup pass the outset branch of kurbo_path_offset uses:
        let contours = graphite_boolean_cleanup(&stroked_path, opts.topology_epsilon, opts.quantize_epsilon);
        let combined = merge_bezpath_vec(&contours);
        Ok(Box::into_raw(Box::new(KurboBezPathInternal::new(combined))))
    })
}

/// Reverses the winding of the path
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_reverse(path: *mut KurboBezPathInternal, out_status: *mut KurboStatus) {
    ffi_call(out_status, (), || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_reverse: path is null".to_string()));
        }
        let p_obj = unsafe { &mut *path };
        p_obj.path = p_obj.path.reverse_subpaths();
        Ok(())
    })
}

/// Returns a new handle rotated by `angle` radians around `center`.
/// Non-mutating: `path` is left untouched.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_rotated(
    path: *const KurboBezPathInternal, angle: f64, center: kurboPos, out_status: *mut KurboStatus,
) -> *mut KurboBezPathInternal {
    ffi_call(out_status, ptr::null_mut(), || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_rotated: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let affine = kurbo::Affine::rotate_about(angle, kurbo::Point::new(center.x, center.y));
        let mut rotated = p_obj.path.clone();
        rotated.apply_affine(affine);
        Ok(Box::into_raw(Box::new(KurboBezPathInternal::new(rotated))))
    })
}

/// Returns the rectangle that contains the path
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_boundingbox(path: *const KurboBezPathInternal, out_status: *mut KurboStatus) -> kurboRect {
    ffi_call(out_status, kurboRect { x0: 0.0, y0: 0.0, x1: 0.0, y1: 0.0 }, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_boundingbox: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        Ok(kurboRect::from_rect(p_obj.path.bounding_box()))
    })
}

/// Hit testing
/// - On open paths : evaluates if the target is on the path
/// - On closed shapes : evaluates if the target is within the shape.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_contains_point(path: *const KurboBezPathInternal, pos: kurboPos, out_status: *mut KurboStatus) -> bool {
    ffi_call(out_status, false, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_contains_point: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        Ok(p_obj.path.contains(pos.to_point()))
    })
}

/// Data structure for retrieving data and handling possible errors.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kurboEvalResult {
    pub pos: kurboPos, pub tangent: kurboPos, pub normal: kurboPos, pub curvature: f64,
}

// Segment evaluation helper
fn eval_seg(seg: kurbo::PathSeg, t: f64) -> kurboEvalResult {
    match seg {
        kurbo::PathSeg::Line(line) => {
            let p = line.eval(t);
            // Manually calculate derivative to avoid ConstPoint type issues
            let d = kurbo::Point::new(line.p1.x - line.p0.x, line.p1.y - line.p0.y);
            kurboEvalResult {
                pos: kurboPos::from_point(p),
                tangent: kurboPos::from_point(d),
                normal: kurboPos::from_point(kurbo::Point::new(-d.y, d.x)),
                curvature: 0.0,
            }
        }
        kurbo::PathSeg::Quad(quad) => {
            let p = quad.eval(t);
            let d1 = quad.deriv().eval(t);
            let d2 = quad.deriv().deriv().eval(t);
            let num = d1.x * d2.y - d1.y * d2.x;
            let den = (d1.x * d1.x + d1.y * d1.y).powf(1.5);
            let curvature = if den > 1e-6 { num / den } else { 0.0 };
            kurboEvalResult {
                pos: kurboPos::from_point(p),
                tangent: kurboPos::from_point(d1),
                normal: kurboPos::from_point(kurbo::Point::new(-d1.y, d1.x)),
                curvature,
            }
        }
        kurbo::PathSeg::Cubic(cubic) => {
            let p = cubic.eval(t);
            let d1 = cubic.deriv().eval(t);
            let d2 = cubic.deriv().deriv().eval(t);
            let num = d1.x * d2.y - d1.y * d2.x;
            let den = (d1.x * d1.x + d1.y * d1.y).powf(1.5);
            let curvature = if den > 1e-6 { num / den } else { 0.0 };
            kurboEvalResult {
                pos: kurboPos::from_point(p),
                tangent: kurboPos::from_point(d1),
                normal: kurboPos::from_point(kurbo::Point::new(-d1.y, d1.x)),
                curvature,
            }
        }
    }
}

/// Linear path evaluation
/// For converting t-values to positions
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_evaluate(path: *const KurboBezPathInternal, t: f64, out_status: *mut KurboStatus) -> kurboEvalResult {
    let zero = kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 };
    ffi_call(out_status, zero, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_evaluate: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let elements = p_obj.path.elements();
        if elements.is_empty() {
            return Ok(zero);
        }

        // Count drawable segments (skip zero-length elements)
        let mut seg_count = 0;
        let mut last_point = Point::new(0.0, 0.0);
        let mut start_point = Point::new(0.0, 0.0);

        for el in elements {
            match el {
                PathEl::MoveTo(p) => {
                    last_point = *p;
                    start_point = *p;
                }
                PathEl::LineTo(p) => {
                    seg_count += 1;
                    last_point = *p;
                }
                PathEl::QuadTo(_p1, p2) => {
                    seg_count += 1;
                    last_point = *p2;
                }
                PathEl::CurveTo(_p1, _p2, p3) => {
                    seg_count += 1;
                    last_point = *p3;
                }
                PathEl::ClosePath => {
                    // Only count ClosePath if it adds a non-zero-length segment
                    if last_point != start_point {
                        seg_count += 1;
                    }
                    last_point = start_point;
                }
            }
        }

        if seg_count == 0 {
            return Ok(zero);
        }

        let clamped_t = t.max(0.0).min(1.0);
        let seg_index = if clamped_t >= 1.0 { seg_count - 1 } else { (clamped_t * (seg_count as f64)).floor() as usize };
        let local_t = if clamped_t >= 1.0 { 1.0 } else { ((clamped_t * (seg_count as f64)) - (seg_index as f64)).max(0.0).min(1.0) };

        // Find and evaluate the target segment
        let mut current_seg = 0;
        last_point = Point::new(0.0, 0.0);
        start_point = Point::new(0.0, 0.0);

        for el in elements {
            match el {
                PathEl::MoveTo(p) => {
                    last_point = *p;
                    start_point = *p;
                }
                PathEl::LineTo(p) => {
                    if current_seg == seg_index {
                        return Ok(eval_seg(PathSeg::Line(Line::new(last_point, *p)), local_t));
                    }
                    current_seg += 1;
                    last_point = *p;
                }
                PathEl::QuadTo(p1, p2) => {
                    if current_seg == seg_index {
                        return Ok(eval_seg(PathSeg::Quad(QuadBez::new(last_point, *p1, *p2)), local_t));
                    }
                    current_seg += 1;
                    last_point = *p2;
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    if current_seg == seg_index {
                        return Ok(eval_seg(PathSeg::Cubic(CubicBez::new(last_point, *p1, *p2, *p3)), local_t));
                    }
                    current_seg += 1;
                    last_point = *p3;
                }
                PathEl::ClosePath => {
                    // Only process ClosePath if it's a non-zero-length segment
                    if last_point != start_point {
                        if current_seg == seg_index {
                            return Ok(eval_seg(PathSeg::Line(Line::new(last_point, start_point)), local_t));
                        }
                        current_seg += 1;
                    }
                    last_point = start_point;
                }
            }
        }

        Ok(zero)
    })
}


/// Euclidean evaluation
/// Takes into account the path length
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_evaluate_euclidean(path: *const KurboBezPathInternal, t: f64, accuracy: f64, out_status: *mut KurboStatus) -> kurboEvalResult {
    let zero = kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 };
    ffi_call(out_status, zero, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_evaluate_euclidean: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let elements = p_obj.path.elements();
        if elements.is_empty() {
            return Ok(zero);
        }

        // Compute arc length of each segment
        let mut segments: Vec<PathSeg> = Vec::new();
        let mut last_point = Point::new(0.0, 0.0);
        let mut start_point = Point::new(0.0, 0.0);

        for el in elements {
            match el {
                PathEl::MoveTo(p) => {
                    last_point = *p;
                    start_point = *p;
                }
                PathEl::LineTo(p) => {
                    segments.push(PathSeg::Line(Line::new(last_point, *p)));
                    last_point = *p;
                }
                PathEl::QuadTo(p1, p2) => {
                    segments.push(PathSeg::Quad(QuadBez::new(last_point, *p1, *p2)));
                    last_point = *p2;
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    segments.push(PathSeg::Cubic(CubicBez::new(last_point, *p1, *p2, *p3)));
                    last_point = *p3;
                }
                PathEl::ClosePath => {
                    segments.push(PathSeg::Line(Line::new(last_point, start_point)));
                    last_point = start_point;
                }
            }
        }

        if segments.is_empty() {
            return Ok(zero);
        }

        // Compute arc lengths and cumulative lengths
        let arc_lengths: Vec<f64> = segments.iter().map(|seg| seg.arclen(accuracy)).collect();
        let total_length: f64 = arc_lengths.iter().sum();

        if total_length <= 0.0 {
            return Ok(zero);
        }

        let clamped_t = t.max(0.0).min(1.0);
        let target_length = clamped_t * total_length;

        // Find which segment contains the target length
        let mut cumulative = 0.0;
        for (i, &seg_len) in arc_lengths.iter().enumerate() {
            if cumulative + seg_len >= target_length || i == segments.len() - 1 {
                // Found the segment
                let local_length = target_length - cumulative;
                let local_t = if seg_len > 0.0 {
                    segments[i].inv_arclen(local_length, accuracy)
                } else {
                    0.0
                };
                return Ok(eval_seg(segments[i], local_t));
            }
            cumulative += seg_len;
        }

        Ok(zero)
    })
}

/// Data structure for retrieving floats
#[repr(C)]
pub struct kurboFloatsRaw { 
    pub data: *const f64, 
    pub len: SizeTC 
}

/// Frees a kurboFloatsRaw buffer
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_floats_free(raw: kurboFloatsRaw) {
    if raw.data.is_null() || raw.len == 0 { return; }
    let _ = panic::catch_unwind(AssertUnwindSafe(|| unsafe {
        let _ = Vec::from_raw_parts(raw.data as *mut f64, raw.len as usize, raw.len as usize);
    }));
}

// Helper: finds special points (inflections OR extrema) on a path, returning global t-values.
// Uses a closure to extract local t-values per segment.
fn find_special_points<F>(path: &KurboBezPathInternal, extract_local_ts: F) -> Vec<f64>
where
    F: Fn(&kurbo::PathSeg) -> Vec<f64>,
{
    let elements = path.path.elements();
    
    // Count drawable segments (matching kurbo_path_evaluate logic)
    let mut seg_count = 0;
    let mut last_point = kurbo::Point::new(0.0, 0.0);
    let mut start_point = kurbo::Point::new(0.0, 0.0);
    
    for el in elements {
        match el {
            kurbo::PathEl::MoveTo(p) => { last_point = *p; start_point = *p; }
            kurbo::PathEl::LineTo(p) => { seg_count += 1; last_point = *p; }
            kurbo::PathEl::QuadTo(_p1, p2) => { seg_count += 1; last_point = *p2; }
            kurbo::PathEl::CurveTo(_p1, _p2, p3) => { seg_count += 1; last_point = *p3; }
            kurbo::PathEl::ClosePath => {
                if last_point != start_point { seg_count += 1; }
                last_point = start_point;
            }
        }
    }
    
    if seg_count == 0 {
        return Vec::new();
    }
    
    // Find special points and convert to global t
    let mut result: Vec<f64> = Vec::new();
    let mut current_seg: usize = 0;
    last_point = kurbo::Point::new(0.0, 0.0);
    start_point = kurbo::Point::new(0.0, 0.0);
    
    for el in elements {
        match el {
            kurbo::PathEl::MoveTo(p) => { last_point = *p; start_point = *p; }
            kurbo::PathEl::LineTo(p) => {
                let seg = kurbo::PathSeg::Line(kurbo::Line::new(last_point, *p));
                for local_t in extract_local_ts(&seg) {
                    result.push((current_seg as f64 + local_t) / seg_count as f64);
                }
                current_seg += 1;
                last_point = *p;
            }
            kurbo::PathEl::QuadTo(p1, p2) => {
                let seg = kurbo::PathSeg::Quad(kurbo::QuadBez::new(last_point, *p1, *p2));
                for local_t in extract_local_ts(&seg) {
                    result.push((current_seg as f64 + local_t) / seg_count as f64);
                }
                current_seg += 1;
                last_point = *p2;
            }
            kurbo::PathEl::CurveTo(p1, p2, p3) => {
                let seg = kurbo::PathSeg::Cubic(kurbo::CubicBez::new(last_point, *p1, *p2, *p3));
                for local_t in extract_local_ts(&seg) {
                    result.push((current_seg as f64 + local_t) / seg_count as f64);
                }
                current_seg += 1;
                last_point = *p3;
            }
            kurbo::PathEl::ClosePath => {
                if last_point != start_point {
                    let seg = kurbo::PathSeg::Line(kurbo::Line::new(last_point, start_point));
                    for local_t in extract_local_ts(&seg) {
                        result.push((current_seg as f64 + local_t) / seg_count as f64);
                    }
                    current_seg += 1;
                }
                last_point = start_point;
            }
        }
    }
    
    result
}

// Hands ownership of `vec` to the caller as a kurboFloatsRaw
// To be released with exactly one call to kurbo_floats_free.
fn floats_into_raw(mut vec: Vec<f64>) -> kurboFloatsRaw {
    if vec.is_empty() {
        return kurboFloatsRaw { data: ptr::null(), len: 0 };
    }
    vec.shrink_to_fit();
    let raw = kurboFloatsRaw { data: vec.as_ptr(), len: vec.len() as SizeTC };
    std::mem::forget(vec);
    raw
}

/// Find path inflection points
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_inflections(path: *const KurboBezPathInternal, out_status: *mut KurboStatus) -> kurboFloatsRaw {
    ffi_call(out_status, kurboFloatsRaw { data: ptr::null(), len: 0 }, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_inflections: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        // Closure: only CubicBez has inflections
        let result = find_special_points(p_obj, |seg| match seg {
            kurbo::PathSeg::Cubic(cubic) => cubic.inflections().to_vec(),
            _ => Vec::new(),
        });
        Ok(floats_into_raw(result))
    })
}

/// Find path extremas
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_extrema(path: *const KurboBezPathInternal, out_status: *mut KurboStatus) -> kurboFloatsRaw {
    ffi_call(out_status, kurboFloatsRaw { data: ptr::null(), len: 0 }, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_extrema: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        // Closure: QuadBez and CubicBez have extrema
        let result = find_special_points(p_obj, |seg| match seg {
            kurbo::PathSeg::Quad(quad) => quad.extrema().to_vec(),
            kurbo::PathSeg::Cubic(cubic) => cubic.extrema().to_vec(),
            _ => Vec::new(),
        });
        Ok(floats_into_raw(result))
    })
}

// ---------------------------------------------------------------------
// Offset & Stroke helpers (improved Kurbo algos)
// Kurbo's offset and stroking algos are "naive", made for rendering : no path cleaning is done and results might contain self-overlapping shapes.
// These helper functions below allow easily reproducing the "desired" behaviours as in Inkscape's offset & stroke algorithms.
// ---------------------------------------------------------------------

/// Options used for offsetting & stroking paths
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KurboOffsetOptions {
    /// Join used both by the stroke & offset construction
    pub join: kurboJoinType,
    /// Only used when `join` is Miter.
    pub miter_limit: f64,
    /// Flattening tolerance for the stroke (only used for inset cleaning)
    pub stroke_tolerance: f64,
    /// Epsilon passed to linesweeper's binary_op_with_eps (only used for inset cleaning)
    pub boolean_epsilon: f64,
    /// Coordinate-quantization grid size applied to detect near-coincident points (only used for outset cleaning)
    pub quantize_epsilon: f64,
    /// Epsilon passed to linesweeper's Topology cleanup pass (only used for outset cleaning)
    pub topology_epsilon: f64,
    /// Do a cleanup stage (set to false for rendering-only = faster)
    pub cleanup_stage: bool,
}

impl KurboOffsetOptions {
    fn defaults() -> Self {
        KurboOffsetOptions {
            join: kurboJoinType::Miter,
            miter_limit: 4.0,
            stroke_tolerance: 0.25,
            boolean_epsilon: 1e-5,
            quantize_epsilon: 1e-8,
            topology_epsilon: 1e-5,
            cleanup_stage: true,
        }
    }
}

/// Returns the default options for stroking and offsetting
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_offset_options_default() -> KurboOffsetOptions {
    KurboOffsetOptions::defaults()
}

fn offset_join_miter_limit(join: Join, limit: f64) -> Option<f64> {
    match join {
        Join::Miter => Some(limit),
        _ => None,
    }
}

// Reduce the precision of a point (aligning it on a grid)
fn quantize_point(point: Point, epsilon: f64) -> Point {
    Point::new(
        (point.x / epsilon).round() * epsilon,
        (point.y / epsilon).round() * epsilon,
    )
}

/// Reproduces the coordinate quantization Graphite's own boolean node applies before a topology pass
/// Aka: reduce the precision of the pointing float vector data
fn graphite_quantize_path(path: &BezPath, epsilon: f64) -> BezPath {
    let mut out = BezPath::new();
    let mut has_subpath = false;

    for el in path.elements() {
        match *el {
            PathEl::MoveTo(point) => {
                if has_subpath {
                    out.close_path();
                }
                out.move_to(quantize_point(point, epsilon));
                has_subpath = true;
            }
            PathEl::LineTo(point) => {
                out.line_to(quantize_point(point, epsilon));
            }
            PathEl::QuadTo(p1, p2) => {
                out.quad_to(quantize_point(p1, epsilon), quantize_point(p2, epsilon));
            }
            PathEl::CurveTo(p1, p2, p3) => {
                out.curve_to(
                    quantize_point(p1, epsilon),
                    quantize_point(p2, epsilon),
                    quantize_point(p3, epsilon),
                );
            }
            PathEl::ClosePath => {
                out.close_path();
                has_subpath = false;
            }
        }
    }

    if has_subpath {
        out.close_path();
    }

    out
}

// Single-path topology pass that removes the self-overlaps, keeping only non-zero-winding contours.
// Used for cleaning a raw outset construction.
fn graphite_boolean_cleanup(path: &BezPath, topology_epsilon: f64, quantize_epsilon: f64) -> Vec<BezPath> {
    // This algo is sensitive to exact coincidence : snap together nearly-identical float coordinates.
    let quantized = graphite_quantize_path(path, quantize_epsilon);

    if quantized.elements().is_empty() {
        return Vec::new();
    }

    let topology = match linesweeper::topology::Topology::<i32>::from_paths(
        std::iter::once((&quantized, ())),
        topology_epsilon,
    ) {
        Ok(topology) => topology,
        Err(_) => return Vec::new(),
    };

    topology
        .contours(|winding| *winding != 0)
        .contours()
        .map(|contour| {
            let mut path = contour.path.clone();
            if !matches!(path.elements().last(), Some(PathEl::ClosePath)) {
                path.close_path();
            }
            path
        })
        .collect()
}

// Boolean operation on paths
// Also useful for cleaning self-overlapping paths, looking at its winding directions.
fn graphite_boolean_binary(a: &BezPath, b: &BezPath, op: linesweeper::BinaryOp, epsilon: f64) -> Vec<BezPath> {
    match linesweeper::binary_op_with_eps(a, b, linesweeper::FillRule::NonZero, op, epsilon) {
        Ok(contours) => contours.contours().map(|c| c.path.clone()).collect(),
        Err(_) => Vec::new(),
    }
}

// Offset a given path and cleanup the result
// Note: A single path can return zero, one or more contours.
// To reproduce this behaviour in Graphite & visualise the cleaning steps,
// See repo file : doc/Path_Offset_Minimal.graphite
fn graphite_offset_path(path: &BezPath, offset: f64, opts: &KurboOffsetOptions) -> Vec<BezPath> {
    if !offset.is_finite() || offset == 0.0 {
        return vec![path.clone()];
    }

    let join = parse_join(opts.join);

    // Positive offset (outset) : vector-types offset + self-overlap cleanup
    if (offset > 0.0) || (opts.cleanup_stage==false) {
        // vector-types' offset_bezpath uses the negative distance
        // convention for an outset — this matches the reproducer exactly,
        // it is not a sign error.
        let offset_path = vector_types::vector::algorithms::offset_bezpath::offset_bezpath(
            path,
            -offset,
            join, // fixme : mitter doesn't seem to work....
            offset_join_miter_limit(join, opts.miter_limit),
        );
        if opts.cleanup_stage {
            return graphite_boolean_cleanup(&offset_path, opts.topology_epsilon, opts.quantize_epsilon);
        }
        else {
            return vec![offset_path];
        }
    }
    // Negative offset (inset) : stroke + boolean subtraction
    else {
        let inset = offset.abs(); // secures: algos crash when negative value

        // This is a trick that seems to work well : stroke + boolop
        let stroke = Stroke::new(2.0 * inset) // double because stroke is centered
            .with_caps(Cap::Butt) // todo : allow setting cap for 
            .with_join(join)
            .with_miter_limit(opts.miter_limit);
        let stroked = kurbo::stroke(path.clone(), &stroke, &kurbo::StrokeOpts::default(), opts.stroke_tolerance);
        return graphite_boolean_binary(path, &stroked, linesweeper::BinaryOp::Difference, opts.boolean_epsilon);
    }
}

/// Offset a given path and cleanup the result
/// Note: A single path can return zero, one or more contours.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_offset(
    path: *const KurboBezPathInternal,
    distance: f64,
    options: *const KurboOffsetOptions,
    out_status: *mut KurboStatus,
) -> *mut KurboBezPathInternal {
    ffi_call(out_status, ptr::null_mut(), || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_offset: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let opts = if options.is_null() { KurboOffsetOptions::defaults() } else { unsafe { *options } };

        let contours = graphite_offset_path(&p_obj.path, distance, &opts);

        // Merge eventual multiple paths into one
        let combined = merge_bezpath_vec(&contours);
        Ok(Box::into_raw(Box::new(KurboBezPathInternal::new(combined))))
    })
}


/// Rotates a path (in degrees) around a center point
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_rotate(path: *mut KurboBezPathInternal, angle: f64, center: kurboPos, out_status: *mut KurboStatus) {
    ffi_call(out_status, (), || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_rotate: path is null".to_string()));
        }
        let p_obj = unsafe { &mut *path };
        let center_point = kurbo::Point::new(center.x, center.y);
        let affine = kurbo::Affine::rotate_about(angle, center_point);
        p_obj.path.apply_affine(affine);
        Ok(())
    })
}

/// Projects a point on a path
/// Returns the closest point on shape.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_project(path: *const KurboBezPathInternal, pos: kurboPos, accuracy: f64, out_status: *mut KurboStatus) -> kurboPos {
    ffi_call(out_status, pos, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_project: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let point = kurbo::Point::new(pos.x, pos.y);

        let mut best_dist_sq = f64::MAX;
        let mut best_point = point;

        for seg in p_obj.path.segments() {
            // ParamCurveNearest is implemented for Line, QuadBez, CubicBez, and PathSeg
            let nearest = seg.nearest(point, accuracy);
            if nearest.distance_sq < best_dist_sq {
                best_dist_sq = nearest.distance_sq;
                best_point = seg.eval(nearest.t);
            }
        }

        Ok(kurboPos::from_point(best_point))
    })
}

// Recursively subdivide two curves to find intersection points
// /!\ Experimental AI implementation !
fn find_intersections_subdivide(
    seg1: PathSeg,
    seg2: PathSeg,
    t1_range: (f64, f64),
    t2_range: (f64, f64),
    tolerance: f64,
    depth: usize,
    results: &mut Vec<(f64, f64)>,
) {
    const MAX_DEPTH: usize = 12;
    
    // Get bounding boxes
    // let bb1 = Shape::bounding_box(&seg1);
    // let bb2 = Shape::bounding_box(&seg2);
    let bb1 = ParamCurveExtrema::bounding_box(&seg1);
    let bb2 = ParamCurveExtrema::bounding_box(&seg2);

    
    // If bounding boxes don't overlap, no intersection possible
    if !bb1.overlaps(bb2) {
        return;
    }
    
    // If we've reached max depth or boxes are small enough, record intersection
    if depth >= MAX_DEPTH || (bb1.width().max(bb1.height()) < tolerance && bb2.width().max(bb2.height()) < tolerance) {
        let t1_mid = (t1_range.0 + t1_range.1) * 0.5;
        let t2_mid = (t2_range.0 + t2_range.1) * 0.5;
        results.push((t1_mid, t2_mid));
        return;
    }
    
    // Subdivide both curves
    let (seg1_a, seg1_b) = seg1.subdivide();
    let (seg2_a, seg2_b) = seg2.subdivide();
    
    let t1_mid = (t1_range.0 + t1_range.1) * 0.5;
    let t2_mid = (t2_range.0 + t2_range.1) * 0.5;
    
    // Recursively check all 4 combinations
    find_intersections_subdivide(seg1_a, seg2_a, (t1_range.0, t1_mid), (t2_range.0, t2_mid), tolerance, depth + 1, results);
    find_intersections_subdivide(seg1_a, seg2_b, (t1_range.0, t1_mid), (t2_mid, t2_range.1), tolerance, depth + 1, results);
    find_intersections_subdivide(seg1_b, seg2_a, (t1_mid, t1_range.1), (t2_range.0, t2_mid), tolerance, depth + 1, results);
    find_intersections_subdivide(seg1_b, seg2_b, (t1_mid, t1_range.1), (t2_mid, t2_range.1), tolerance, depth + 1, results);
}

/// Finds path self intersections, returned as linear t-values
/// /!\ Experimental AI implementation !
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_self_intersections(
    path: *const KurboBezPathInternal,
    error_threshold: f64,
    _min_dist_param: f64,
    out_status: *mut KurboStatus,
) -> kurboFloatsRaw {
    ffi_call(out_status, kurboFloatsRaw { data: ptr::null(), len: 0 }, || {
        if path.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_self_intersections: path is null".to_string()));
        }
        let p_obj = unsafe { &*path };
        let mut last_vec: Vec<f64> = Vec::new();

        let elements = p_obj.path.elements();
        let mut segs: Vec<(usize, PathSeg)> = Vec::new();
        let mut last_pt = Point::new(0.0, 0.0);
        let mut start_pt = Point::new(0.0, 0.0);
        let mut seg_idx = 0;
        let mut is_closed = false;

        // Collect all drawable segments
        for el in elements {
            match el {
                PathEl::MoveTo(p) => { last_pt = *p; start_pt = *p; }
                PathEl::LineTo(p) => {
                    segs.push((seg_idx, PathSeg::Line(Line::new(last_pt, *p))));
                    last_pt = *p; seg_idx += 1;
                }
                PathEl::QuadTo(p1, p2) => {
                    segs.push((seg_idx, PathSeg::Quad(QuadBez::new(last_pt, *p1, *p2))));
                    last_pt = *p2; seg_idx += 1;
                }
                PathEl::CurveTo(p1, p2, p3) => {
                    segs.push((seg_idx, PathSeg::Cubic(CubicBez::new(last_pt, *p1, *p2, *p3))));
                    last_pt = *p3; seg_idx += 1;
                }
                PathEl::ClosePath => {
                    if last_pt != start_pt {
                        segs.push((seg_idx, PathSeg::Line(Line::new(last_pt, start_pt))));
                        seg_idx += 1;
                    }
                    is_closed = true;
                    last_pt = start_pt;
                }
            }
        }

        let total_segs = seg_idx as f64;
        if total_segs == 0.0 {
            return Ok(floats_into_raw(last_vec));
        }

        // Check all pairs of segments using subdivision
        for i in 0..segs.len() {
            for j in (i + 1)..segs.len() {
                let is_adjacent = (j == i + 1) || (is_closed && i == 0 && j == segs.len() - 1);
                let (idx1, seg1) = segs[i];
                let (idx2, seg2) = segs[j];

                // Fast bounding box rejection
                if !ParamCurveExtrema::bounding_box(&seg1).overlaps(ParamCurveExtrema::bounding_box(&seg2)) {
                    continue;
                }

                // Find intersections using subdivision
                let mut local_intersections: Vec<(f64, f64)> = Vec::new();
                find_intersections_subdivide(seg1, seg2, (0.0, 1.0), (0.0, 1.0), error_threshold, 0, &mut local_intersections);

                // Process found intersections
                for (local_t1, local_t2) in local_intersections {
                    // Filter junction zones for adjacent segments
                    if is_adjacent && local_t1 > 0.9 && local_t2 < 0.1 {
                        continue;
                    }

                    // Convert to global t-values (linear distribution)
                    let global_t1 = (idx1 as f64 + local_t1) / total_segs;
                    let global_t2 = (idx2 as f64 + local_t2) / total_segs;

                    last_vec.push(global_t1);
                    last_vec.push(global_t2);
                }
            }
        }

        Ok(floats_into_raw(last_vec))
    })
}


/// Boolean operation selector for kurbo_path_boolean.
#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KurboBooleanOp {
    Union = 0,
    Intersection = 1,
    Difference = 2,
    Xor = 3,
}

impl KurboBooleanOp {
    fn to_linesweeper(self) -> linesweeper::BinaryOp {
        match self {
            KurboBooleanOp::Union        => linesweeper::BinaryOp::Union,
            KurboBooleanOp::Intersection => linesweeper::BinaryOp::Intersection,
            KurboBooleanOp::Difference   => linesweeper::BinaryOp::Difference,
            KurboBooleanOp::Xor          => linesweeper::BinaryOp::Xor,
        }
    }
}

/// Non-mutating: `a` and `b` are read-only and stay valid and reusable.
/// Returns a new handle holding every result contour as a subpath.
#[unsafe(no_mangle)]
pub extern "C" fn kurbo_path_boolean(
    a: *const KurboBezPathInternal,
    b: *const KurboBezPathInternal,
    op: KurboBooleanOp,
    epsilon: f64,
    out_status: *mut KurboStatus,
) -> *mut KurboBezPathInternal {
    ffi_call(out_status, ptr::null_mut(), || {
        if a.is_null() || b.is_null() {
            return Err((KurboStatus::NullPointer, "kurbo_path_boolean: a or b is null".to_string()));
        }
        let (a_obj, b_obj) = unsafe { (&*a, &*b) };
        let eps = if epsilon.is_finite() && epsilon > 0.0 { epsilon } else { 1e-5 };

        let contours = graphite_boolean_binary(&a_obj.path, &b_obj.path, op.to_linesweeper(), eps);

        let combined = merge_bezpath_vec(&contours);
        Ok(Box::into_raw(Box::new(KurboBezPathInternal::new(combined))))
    })
}