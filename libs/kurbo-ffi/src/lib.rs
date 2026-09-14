use std::slice;
use std::ffi::c_ulong;
use kurbo::{BezPath, PathEl, Point, Rect, Shape, Stroke, Join, Cap, Line, QuadBez, CubicBez, PathSeg, ParamCurve, ParamCurveDeriv, ParamCurveNearest, ParamCurveArclen, ParamCurveExtrema};

type SizeTC = c_ulong;

#[repr(C)]
pub enum kurboJoinType {
    Miter,
    Round,
    Bevel,
}

#[repr(C)]
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

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct KurboPathEl {
    pub tag: KurboPathElType,
    pub p0: kurboPos, pub p1: kurboPos, pub p2: kurboPos,
}

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

#[repr(C)]
pub struct KurboPathRaw {
    pub data: *const KurboPathEl,
    pub len: SizeTC,
}

pub struct KurboBezPath {
    pub(crate) path: BezPath,
    pub(crate) elements: Vec<KurboPathEl>,
}

impl KurboBezPath {
    pub fn new(path: BezPath) -> Self {
        let elements: Vec<KurboPathEl> = path.elements().iter().map(KurboPathEl::from_path_el).collect();
        KurboBezPath { path, elements }
    }
}

#[no_mangle]
pub extern "C" fn kurbo_path_create(elements_opt: Option<&KurboPathRaw>) -> *mut KurboBezPath {
    if let Some(elements_raw) = elements_opt {
        if !elements_raw.data.is_null() {
            let elements_slice = unsafe { slice::from_raw_parts(elements_raw.data as *const KurboPathEl, elements_raw.len as usize) };
            let path: BezPath = elements_slice.iter().map(|el| el.to_path_el()).collect();
            return Box::into_raw(Box::new(KurboBezPath::new(path)));
        }
    }
    Box::into_raw(Box::new(KurboBezPath::new(BezPath::new())))
}

#[no_mangle]
pub extern "C" fn kurbo_path_destroy(path: *mut KurboBezPath) {
    if !path.is_null() { unsafe { let _ = Box::from_raw(path); } }
}

#[no_mangle]
pub extern "C" fn kurbo_path_return_handle_data(path: *mut KurboBezPath) -> KurboPathRaw {
    let p = unsafe { assert!(!path.is_null()); &mut *path };
    p.elements = p.path.elements().iter().map(KurboPathEl::from_path_el).collect();
    KurboPathRaw { data: p.elements.as_mut_ptr(), len: p.elements.len() as SizeTC }
}

#[no_mangle]
pub extern "C" fn kurbo_path_append_move_to(path: *mut KurboBezPath, p: kurboPos) {
    unsafe { (&mut *path).path.move_to(p.to_point()); }
}
#[no_mangle]
pub extern "C" fn kurbo_path_append_line_to(path: *mut KurboBezPath, p: kurboPos) {
    unsafe { (&mut *path).path.line_to(p.to_point()); }
}
#[no_mangle]
pub extern "C" fn kurbo_path_append_quad_to(path: *mut KurboBezPath, p1: kurboPos, p2: kurboPos) {
    unsafe { (&mut *path).path.quad_to(p1.to_point(), p2.to_point()); }
}
#[no_mangle]
pub extern "C" fn kurbo_path_append_curve_to(path: *mut KurboBezPath, p1: kurboPos, p2: kurboPos, p3: kurboPos) {
    unsafe { (&mut *path).path.curve_to(p1.to_point(), p2.to_point(), p3.to_point()); }
}
#[no_mangle]
pub extern "C" fn kurbo_path_append_close(path: *mut KurboBezPath) {
    unsafe { (&mut *path).path.close_path(); }
}

#[no_mangle]
pub extern "C" fn kurbo_path_stroke(path: *mut KurboBezPath, width: f64, join: kurboJoinType, miter_limit: f64, start_cap: kurboCapType, end_cap: kurboCapType) -> *mut KurboBezPath {
    let p_obj = unsafe { &mut *path };
    let stroke = Stroke::new(width).with_join(parse_join(join)).with_miter_limit(miter_limit).with_start_cap(parse_cap(start_cap)).with_end_cap(parse_cap(end_cap));
    let m_stroke_opts : kurbo::StrokeOpts = <kurbo::StrokeOpts as std::default::Default>::default();
    Box::into_raw(Box::new(KurboBezPath::new(kurbo::stroke(p_obj.path.clone(), &stroke, &m_stroke_opts, 0.1))))
}

#[no_mangle]
pub extern "C" fn kurbo_path_reverse(path: *mut KurboBezPath) {
    if path.is_null() {
        return;
    }
    let p_obj = unsafe { &mut *path };
    p_obj.path = p_obj.path.reverse_subpaths();
}

#[no_mangle]
pub extern "C" fn kurbo_path_boundingbox(path: *mut KurboBezPath) -> kurboRect {
    let p_obj = unsafe { &mut *path };
    kurboRect::from_rect(p_obj.path.bounding_box())
}

#[no_mangle]
pub extern "C" fn kurbo_path_contains_point(path: *mut KurboBezPath, pos: kurboPos) -> bool {
    let p_obj = unsafe { &mut *path };
    p_obj.path.contains(pos.to_point())
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct kurboEvalResult {
    pub pos: kurboPos, pub tangent: kurboPos, pub normal: kurboPos, pub curvature: f64,
}

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

#[no_mangle]
pub extern "C" fn kurbo_path_evaluate(path: *mut KurboBezPath, t: f64) -> kurboEvalResult {

    let p_obj = unsafe { &mut *path };
    let elements = p_obj.path.elements();
    if elements.is_empty() {
        return kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 };
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
        return kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 };
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
                    return eval_seg(PathSeg::Line(Line::new(last_point, *p)), local_t);
                }
                current_seg += 1;
                last_point = *p;
            }
            PathEl::QuadTo(p1, p2) => {
                if current_seg == seg_index {
                    return eval_seg(PathSeg::Quad(QuadBez::new(last_point, *p1, *p2)), local_t);
                }
                current_seg += 1;
                last_point = *p2;
            }
            PathEl::CurveTo(p1, p2, p3) => {
                if current_seg == seg_index {
                    return eval_seg(PathSeg::Cubic(CubicBez::new(last_point, *p1, *p2, *p3)), local_t);
                }
                current_seg += 1;
                last_point = *p3;
            }
            PathEl::ClosePath => {
                // Only process ClosePath if it's a non-zero-length segment
                if last_point != start_point {
                    if current_seg == seg_index {
                        return eval_seg(PathSeg::Line(Line::new(last_point, start_point)), local_t);
                    }
                    current_seg += 1;
                }
                last_point = start_point;
            }
        }
    }
    
    kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 }
}

#[no_mangle]
pub extern "C" fn kurbo_path_evaluate_euclidean(path: *mut KurboBezPath, t: f64, accuracy: f64) -> kurboEvalResult {
    let p_obj = unsafe { &mut *path };
    let elements = p_obj.path.elements();
    if elements.is_empty() {
        return kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 };
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
        return kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 };
    }
    
    // Compute arc lengths and cumulative lengths
    let arc_lengths: Vec<f64> = segments.iter().map(|seg| seg.arclen(accuracy)).collect();
    let total_length: f64 = arc_lengths.iter().sum();
    
    if total_length <= 0.0 {
        return kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 };
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
            return eval_seg(segments[i], local_t);
        }
        cumulative += seg_len;
    }
    
    kurboEvalResult { pos: kurboPos::new(0.0, 0.0), tangent: kurboPos::new(0.0, 0.0), normal: kurboPos::new(0.0, 0.0), curvature: 0.0 }
}

#[repr(C)]
pub struct kurboFloatsRaw { 
    pub data: *const f64, 
    pub len: SizeTC 
}

// A wrapper to allow mutable static state in FFI while satisfying the Sync requirement.
// SAFETY: The C++ caller is responsible for not calling this function concurrently 
// from multiple threads, or for handling the race conditions on the returned pointer.
struct FfiBuffer<T>(std::cell::UnsafeCell<T>);
unsafe impl<T> Sync for FfiBuffer<T> {}

static LAST_VEC_RAW: FfiBuffer<Vec<f64>> = FfiBuffer(std::cell::UnsafeCell::new(Vec::new()));

/// Private helper: finds special points (inflections OR extrema) on a path,
/// returning global t-values. Uses a closure to extract local t-values per segment.
fn find_special_points<F>(path: &KurboBezPath, extract_local_ts: F) -> Vec<f64>
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

#[no_mangle]
pub extern "C" fn kurbo_path_inflections(path: *mut KurboBezPath) -> kurboFloatsRaw {
    let p_obj = unsafe { &*path };
    let last_vec = unsafe { &mut *LAST_VEC_RAW.0.get() };
    last_vec.clear();
    
    // Closure: only CubicBez has inflections
    let result = find_special_points(p_obj, |seg| {
        match seg {
            kurbo::PathSeg::Cubic(cubic) => cubic.inflections().to_vec(),
            _ => Vec::new(),
        }
    });
    
    if result.is_empty() {
        return kurboFloatsRaw { data: std::ptr::null(), len: 0 };
    }
    
    last_vec.extend(result);
    kurboFloatsRaw {
        data: last_vec.as_ptr(),
        len: last_vec.len() as SizeTC,
    }
}

#[no_mangle]
pub extern "C" fn kurbo_path_extrema(path: *mut KurboBezPath) -> kurboFloatsRaw {
    let p_obj = unsafe { &*path };
    let last_vec = unsafe { &mut *LAST_VEC_RAW.0.get() };
    last_vec.clear();
    
    // Closure: QuadBez and CubicBez have extrema
    let result = find_special_points(p_obj, |seg| {
        match seg {
            kurbo::PathSeg::Quad(quad) => quad.extrema().to_vec(),
            kurbo::PathSeg::Cubic(cubic) => cubic.extrema().to_vec(),
            _ => Vec::new(),
        }
    });
    
    if result.is_empty() {
        return kurboFloatsRaw { data: std::ptr::null(), len: 0 };
    }
    
    last_vec.extend(result);
    kurboFloatsRaw {
        data: last_vec.as_ptr(),
        len: last_vec.len() as SizeTC,
    }
}

fn offset_bezpath(path: &BezPath, distance: f64, tolerance: f64) -> BezPath {
    let mut result = BezPath::new();
    let mut temp_path = BezPath::new();
    
    let mut current_subpath_segs: Vec<PathSeg> = Vec::new();
    let mut last_pt = Point::new(0.0, 0.0);
    let mut start_pt = Point::new(0.0, 0.0);
    
    for el in path.elements() {
        match el {
            PathEl::MoveTo(p) => {
                // Process previous subpath before starting a new one
                process_subpath(&current_subpath_segs, distance, tolerance, &mut result, &mut temp_path);
                current_subpath_segs.clear();
                last_pt = *p;
                start_pt = *p;
            }
            PathEl::LineTo(p) => {
                current_subpath_segs.push(PathSeg::Line(Line::new(last_pt, *p)));
                last_pt = *p;
            }
            PathEl::QuadTo(p1, p2) => {
                current_subpath_segs.push(PathSeg::Quad(QuadBez::new(last_pt, *p1, *p2)));
                last_pt = *p2;
            }
            PathEl::CurveTo(p1, p2, p3) => {
                current_subpath_segs.push(PathSeg::Cubic(CubicBez::new(last_pt, *p1, *p2, *p3)));
                last_pt = *p3;
            }
            PathEl::ClosePath => {
                // Add closing segment if it's not already closed
                if last_pt != start_pt {
                    current_subpath_segs.push(PathSeg::Line(Line::new(last_pt, start_pt)));
                }
                // Process current subpath
                process_subpath(&current_subpath_segs, distance, tolerance, &mut result, &mut temp_path);
                current_subpath_segs.clear();
                result.close_path(); // Explicitly close the offset subpath
                last_pt = start_pt;
            }
        }
    }
    
    // Process any remaining open subpath
    process_subpath(&current_subpath_segs, distance, tolerance, &mut result, &mut temp_path);
    
    result
}

fn process_subpath(
    segs: &[PathSeg],
    distance: f64,
    tolerance: f64,
    result: &mut BezPath,
    temp_path: &mut BezPath,
) {
    if segs.is_empty() {
        return;
    }
    
    for (i, seg) in segs.iter().enumerate() {
        temp_path.truncate(0);
        kurbo::offset::offset_cubic(seg.to_cubic(), distance, tolerance, temp_path);
        
        for (j, el) in temp_path.elements().iter().enumerate() {
            // Skip the MoveTo of subsequent segments in the SAME subpath to maintain continuity.
            // The first segment (i == 0) provides the correct starting MoveTo for the subpath.
            if i > 0 && j == 0 && matches!(el, PathEl::MoveTo(_)) {
                continue;
            }
            result.push(*el);
        }
    }
}

#[no_mangle]
pub extern "C" fn kurbo_path_offset(path: *mut KurboBezPath, distance: f64, tolerance: f64) -> *mut KurboBezPath {
    let p_obj = unsafe { &mut *path };
    let offset_path = offset_bezpath(&p_obj.path, distance, tolerance);
    Box::into_raw(Box::new(KurboBezPath::new(offset_path)))
}

#[no_mangle]
pub extern "C" fn kurbo_path_rotate(path: *mut KurboBezPath, angle: f64, center: kurboPos) {
    let p_obj = unsafe { &mut *path };
    let center_point = kurbo::Point::new(center.x, center.y);
    let affine = kurbo::Affine::rotate_about(angle, center_point);
    p_obj.path.apply_affine(affine);
}

#[no_mangle]
pub extern "C" fn kurbo_path_project(path: *mut KurboBezPath, pos: kurboPos, accuracy: f64) -> kurboPos {
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
    
    kurboPos::from_point(best_point)
}

/// Recursively subdivide two curves to find intersection points
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

#[no_mangle]
pub extern "C" fn kurbo_path_self_intersections(
    path: *mut KurboBezPath,
    error_threshold: f64,
    _min_dist_param: f64,
) -> kurboFloatsRaw {
    let p_obj = unsafe { &*path };
    let last_vec = unsafe { &mut *LAST_VEC_RAW.0.get() };
    last_vec.clear();

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
        return kurboFloatsRaw { data: std::ptr::null(), len: 0 };
    }

    // Check all pairs of segments using subdivision
    for i in 0..segs.len() {
        for j in (i + 1)..segs.len() {
            let is_adjacent = (j == i + 1) || (is_closed && i == 0 && j == segs.len() - 1);
            let (idx1, seg1) = segs[i];
            let (idx2, seg2) = segs[j];

            // Fast bounding box rejection
            // if !Shape::bounding_box(&seg1).overlaps(Shape::bounding_box(&seg2)) {
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

    if last_vec.is_empty() {
        return kurboFloatsRaw { data: std::ptr::null(), len: 0 };
    }

    kurboFloatsRaw {
        data: last_vec.as_ptr(),
        len: last_vec.len() as SizeTC,
    }
}