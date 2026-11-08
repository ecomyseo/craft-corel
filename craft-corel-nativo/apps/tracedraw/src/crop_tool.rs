//! The Crop tool. A drag draws the crop area; it stays on the page with
//! eight handles until it is applied (a double-click inside it, Enter, or
//! the bar's Crop button) or cleared (Escape, a click outside it, Clear
//! Crop Area). A drag inside the area moves it, a drag on a handle
//! resizes it, a drag outside it draws a new one.
//!
//! Cropping keeps what lies inside the area, of the selection or (with
//! nothing selected) of every object on the page's unlocked, visible
//! layers. Closed outlines are intersected with the area and open curves
//! are cut where they cross its edge, keeping the pieces inside; text
//! becomes its outlines first. Bitmaps, tables and symbols are put in a
//! clip frame of the area, so their content stays whole. Groups and clip
//! frames are cropped member by member (a clip frame's frame only).
//! Objects wholly inside are left as they are; objects wholly outside are
//! deleted. One undo step.
//!
//! With the bar's "Crop picture pixels" on, an upright picture (not
//! turned or slanted) loses its pixels outside the area instead and stays
//! a free picture (#1913). Bitmaps > Crop to Frame does the same to a
//! picture already in a clip frame: the pixels outside the frame go and a
//! rectangular frame lets the picture go free.

use crate::app::App;
use tracedraw_core::{
    document::{Shape, ShapeKind},
    geometry::{Affine, BezPath, ParamCurve, PathEl, Point, Rect, Shape as _, Vec2},
    shaping, Command, Fill, ShapeId,
};

/// A drag of the Crop tool under way.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CropDrag {
    /// A new area from `start`.
    New { start: Point },
    /// The area moved from where it was.
    Move { start: Point, from: Rect },
    /// Handle `which` (0 to 7: corners and side middles, counterclockwise
    /// from the bottom left) dragged.
    Handle { which: usize, from: Rect },
}

/// What cropping does to one object.
#[derive(Debug, Clone, PartialEq)]
pub enum Cropped {
    Keep,
    Gone,
    Into(Box<Shape>),
}

/// The eight handles of an area: corners and side middles,
/// counterclockwise from the bottom left.
pub fn handles(r: Rect) -> [Point; 8] {
    let c = r.center();
    [
        Point::new(r.x0, r.y0),
        Point::new(c.x, r.y0),
        Point::new(r.x1, r.y0),
        Point::new(r.x1, c.y),
        Point::new(r.x1, r.y1),
        Point::new(c.x, r.y1),
        Point::new(r.x0, r.y1),
        Point::new(r.x0, c.y),
    ]
}

/// `from` with handle `which` moved to `p`.
pub fn resized(from: Rect, which: usize, p: Point) -> Rect {
    let (mut x0, mut y0, mut x1, mut y1) = (from.x0, from.y0, from.x1, from.y1);
    match which {
        0 => (x0, y0) = (p.x, p.y),
        1 => y0 = p.y,
        2 => (x1, y0) = (p.x, p.y),
        3 => x1 = p.x,
        4 => (x1, y1) = (p.x, p.y),
        5 => y1 = p.y,
        6 => (x0, y1) = (p.x, p.y),
        _ => x0 = p.x,
    }
    Rect::new(x0, y0, x1, y1).abs()
}

fn finite_rect(r: Rect) -> bool {
    r.x0.is_finite() && r.y0.is_finite() && r.x1.is_finite() && r.y1.is_finite()
}

/// Is the subpath closed (a closing element, or it ends where it started)?
fn closes(sub: &BezPath, closed: bool) -> bool {
    if closed {
        return true;
    }
    let els = sub.elements();
    let start = match els.first() {
        Some(PathEl::MoveTo(p)) => *p,
        _ => return false,
    };
    let end = match els.last() {
        Some(PathEl::LineTo(p)) | Some(PathEl::QuadTo(_, p)) | Some(PathEl::CurveTo(_, _, p)) => *p,
        _ => return false,
    };
    els.len() > 2 && (end - start).hypot() < 1e-6
}

/// The part of `page` (page space) inside `rect`, and whether any of it is
/// closed.
fn clip_path(page: &BezPath, rect: Rect) -> (BezPath, bool) {
    let mut closed_parts = BezPath::new();
    let mut out = BezPath::new();
    let edge = [
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x1, rect.y0),
        Point::new(rect.x1, rect.y1),
        Point::new(rect.x0, rect.y1),
        Point::new(rect.x0, rect.y0),
    ];
    for (sub, closed) in tracedraw_core::effects::subpaths(page) {
        if closes(&sub, closed) {
            closed_parts.extend(sub.elements().iter().copied());
            if !closed {
                closed_parts.close_path();
            }
            continue;
        }
        let cuts = crate::knife::crossings(&sub, &edge);
        let pieces = if cuts.is_empty() {
            vec![sub]
        } else {
            crate::knife::split_open(&sub, &cuts)
        };
        for piece in pieces {
            // A piece lies wholly inside or outside: its middle tells.
            let segs: Vec<_> = piece.segments().collect();
            let Some(mid) = segs.get(segs.len() / 2).map(|s| s.eval(0.5)) else {
                continue;
            };
            if rect.contains(mid) {
                out.extend(piece.elements().iter().copied());
            }
        }
    }
    let mut any_closed = false;
    if !closed_parts.elements().is_empty() {
        let clipped = shaping::overlay(&closed_parts, &rect.to_path(0.01), shaping::Op::Intersect);
        any_closed = !clipped.elements().is_empty();
        out.extend(clipped.elements().iter().copied());
    }
    (out, any_closed)
}

/// The copy of `s` (a picture, table or symbol) that goes inside a new
/// clip frame in its place, with the id `id`. It keeps the object's
/// transparency and opacity: a gradient transparency then stays on the
/// picture's own box, where it lies on the page, however the frame is
/// cropped later (#1491), and `.cdr` files keep it on the picture.
pub(crate) fn framed_contents(s: &Shape, id: ShapeId) -> Shape {
    let mut inner = s.clone();
    inner.id = id;
    inner.transform = Affine::IDENTITY;
    inner
}

/// The commands that leave the object `s`, now a clip frame holding
/// [`framed_contents`] of it, without its own transparency and opacity:
/// the copy inside has them, and the clip frame would lay them on a
/// second time, its gradient stretched across the frame's box (#1491).
pub(crate) fn frame_drops_transparency(s: &Shape) -> Vec<Command> {
    let transparency = |e: &tracedraw_core::live::Effect| {
        matches!(e, tracedraw_core::live::Effect::Transparency { .. })
    };
    let mut cmds = Vec::new();
    if s.effects.iter().any(transparency) {
        cmds.push(Command::SetEffects {
            shape: s.id,
            effects: s
                .effects
                .iter()
                .filter(|e| !transparency(e))
                .cloned()
                .collect(),
        });
    }
    if s.opacity < 1.0 {
        cmds.push(Command::SetOpacity {
            shapes: vec![s.id],
            opacity: 1.0,
        });
    }
    cmds
}

/// The whole pixels of a `width` by `height` picture laid on `rect` (its
/// own space, rows from the top) that `area` (the same space) touches:
/// columns `x0..x1` and rows `y0..y1`; `None` when it touches none.
pub fn pixel_window(
    rect: Rect,
    width: u32,
    height: u32,
    area: Rect,
) -> Option<(u32, u32, u32, u32)> {
    if rect.width() <= 0.0 || rect.height() <= 0.0 || width == 0 || height == 0 {
        return None;
    }
    let (w, h) = (f64::from(width), f64::from(height));
    // A hair's tolerance, so an area on a pixel edge keeps no extra
    // column of pixels.
    let lo = |v: f64| (v + 1e-6).floor();
    let hi = |v: f64| (v - 1e-6).ceil();
    let x0 = lo((area.x0 - rect.x0) / rect.width() * w).clamp(0.0, w);
    let x1 = hi((area.x1 - rect.x0) / rect.width() * w).clamp(0.0, w);
    let y0 = lo((rect.y1 - area.y1) / rect.height() * h).clamp(0.0, h);
    let y1 = hi((rect.y1 - area.y0) / rect.height() * h).clamp(0.0, h);
    if !(x0 < x1 && y0 < y1) {
        return None;
    }
    Some((x0 as u32, y0 as u32, x1 as u32, y1 as u32))
}

/// The picture `s` (a bitmap) with only its pixels inside `area` (a
/// rectangle in the picture's own space, the space of its `rect`) kept,
/// on its whole pixels: its new kind and its ink channels cut alike
/// (#1913). The effects stack's original is cut the same way when it
/// lies on the same pixels, else the effects stay in the pixels. `None`
/// when the area keeps every pixel or none, or the picture cannot be
/// read.
pub fn crop_picture(
    s: &Shape,
    area: Rect,
) -> Option<(
    ShapeKind,
    Option<Box<tracedraw_core::picture_inks::PictureInks>>,
)> {
    let ShapeKind::Bitmap {
        rect,
        width_px,
        height_px,
        png,
        fx,
    } = &s.kind
    else {
        return None;
    };
    let rect = rect.abs();
    let (x0, y0, x1, y1) = pixel_window(rect, *width_px, *height_px, area)?;
    if (x0, y0, x1, y1) == (0, 0, *width_px, *height_px) {
        return None;
    }
    let cut = |png: &[u8]| -> Option<Vec<u8>> {
        let img = crate::bitmap_fx::decode(png)?;
        if img.width() != *width_px || img.height() != *height_px {
            return None;
        }
        let part = image::imageops::crop_imm(&img, x0, y0, x1 - x0, y1 - y0).to_image();
        if crate::bitmap_fx::png_is_gray(png) {
            crate::bitmap_fx::encode_gray(&part)
        } else {
            crate::bitmap_fx::encode(&part)
        }
    };
    let new_png = cut(png)?;
    let (w, h) = (f64::from(*width_px), f64::from(*height_px));
    let new_rect = Rect::new(
        rect.x0 + f64::from(x0) / w * rect.width(),
        rect.y1 - f64::from(y1) / h * rect.height(),
        rect.x0 + f64::from(x1) / w * rect.width(),
        rect.y1 - f64::from(y0) / h * rect.height(),
    );
    let new_fx = fx.as_ref().and_then(|f| {
        let same = f.width_px == *width_px
            && f.height_px == *height_px
            && (f.rect.abs().x0 - rect.x0).abs() < 1e-6
            && (f.rect.abs().y0 - rect.y0).abs() < 1e-6
            && (f.rect.abs().x1 - rect.x1).abs() < 1e-6
            && (f.rect.abs().y1 - rect.y1).abs() < 1e-6;
        let png = cut(&f.png).filter(|_| same)?;
        let mut f = f.clone();
        f.rect = new_rect;
        f.width_px = x1 - x0;
        f.height_px = y1 - y0;
        f.png = png.into();
        Some(f)
    });
    // The ink planes, one under the other, each cut like the pixels.
    let inks = s
        .picture_inks
        .as_deref()
        .filter(|i| i.fits(png.as_slice()))
        .and_then(|i| {
            let planes = image::load_from_memory(&i.png).ok()?.to_luma8();
            let n = i.inks.len() as u32;
            if n == 0 || planes.width() != *width_px || planes.height() != height_px * n {
                return None;
            }
            let (cw, ch) = (x1 - x0, y1 - y0);
            let mut out = image::GrayImage::new(cw, ch * n);
            for k in 0..n {
                let part =
                    image::imageops::crop_imm(&planes, x0, k * height_px + y0, cw, ch).to_image();
                image::imageops::replace(&mut out, &part, 0, i64::from(k * ch));
            }
            let mut bytes = std::io::Cursor::new(Vec::new());
            out.write_to(&mut bytes, image::ImageFormat::Png).ok()?;
            Some(Box::new(tracedraw_core::picture_inks::PictureInks {
                inks: i.inks.clone(),
                png: bytes.into_inner().into(),
                of: tracedraw_core::picture_inks::fingerprint(&new_png),
                profile: i.profile.clone(),
            }))
        });
    Some((
        ShapeKind::Bitmap {
            rect: new_rect,
            width_px: x1 - x0,
            height_px: y1 - y0,
            png: new_png.into(),
            fx: new_fx,
        },
        inks,
    ))
}

/// Whether `m` keeps a picture upright: no turn or slant, so its
/// rectangle stays a rectangle along the page's axes.
fn upright(m: Affine) -> bool {
    let [a, b, c, d, _, _] = m.as_coeffs();
    b.abs() < 1e-9 && c.abs() < 1e-9 && a.abs() > 1e-12 && d.abs() > 1e-12
}

/// Crop `s` (whose parent maps to the page by `parent`) to `rect` (page
/// space); `new_id` gives ids for the parts it adds. Pictures go in a
/// clip frame of the area; see [`crop_shape_with`] to cut their pixels.
pub fn crop_shape(
    s: &Shape,
    parent: Affine,
    rect: Rect,
    new_id: &mut dyn FnMut() -> ShapeId,
) -> Cropped {
    crop_shape_with(s, parent, rect, new_id, false)
}

/// [`crop_shape`]; with `pixels` an upright picture loses its pixels
/// outside the area instead of going in a clip frame (#1913).
pub fn crop_shape_with(
    s: &Shape,
    parent: Affine,
    rect: Rect,
    new_id: &mut dyn FnMut() -> ShapeId,
    pixels: bool,
) -> Cropped {
    let to_page = parent * s.transform;
    match &s.kind {
        ShapeKind::Group { children } => {
            let mut out = Vec::with_capacity(children.len());
            let mut changed = false;
            for c in children {
                match crop_shape_with(c, to_page, rect, new_id, pixels) {
                    Cropped::Keep => out.push(c.clone()),
                    Cropped::Gone => changed = true,
                    Cropped::Into(n) => {
                        changed = true;
                        out.push(*n);
                    }
                }
            }
            if out.is_empty() {
                Cropped::Gone
            } else if !changed {
                Cropped::Keep
            } else {
                let mut g = s.clone();
                g.kind = ShapeKind::Group { children: out };
                Cropped::Into(Box::new(g))
            }
        }
        ShapeKind::ClipFrame { frame, contents } => {
            // The frame clips the contents: cropping it is enough.
            match crop_shape(frame, to_page, rect, new_id) {
                Cropped::Keep => Cropped::Keep,
                Cropped::Gone => Cropped::Gone,
                Cropped::Into(f) => {
                    let mut c = s.clone();
                    c.kind = ShapeKind::ClipFrame {
                        frame: f,
                        contents: contents.clone(),
                    };
                    Cropped::Into(Box::new(c))
                }
            }
        }
        _ => {
            if to_page.determinant().abs() < 1e-12 {
                return Cropped::Keep;
            }
            let local = s.local_path();
            let page = to_page * local;
            let b = page.bounding_box();
            if !finite_rect(b) {
                return Cropped::Keep;
            }
            if rect.contains(Point::new(b.x0, b.y0)) && rect.contains(Point::new(b.x1, b.y1)) {
                return Cropped::Keep;
            }
            if b.intersect(rect).area() <= 0.0 && (b.width() > 0.0 && b.height() > 0.0) {
                return Cropped::Gone;
            }
            if pixels && upright(to_page) && matches!(s.kind, ShapeKind::Bitmap { .. }) {
                let area = to_page.inverse().transform_rect_bbox(rect);
                if let Some((kind, inks)) = crop_picture(s, area) {
                    let mut n = s.clone();
                    n.kind = kind;
                    n.picture_inks = inks;
                    return Cropped::Into(Box::new(n));
                }
            }
            match &s.kind {
                ShapeKind::Bitmap { .. }
                | ShapeKind::Table(_)
                | ShapeKind::SymbolInstance { .. } => {
                    // A clip frame of the area, in the object's own space.
                    let mut frame = Shape::new(
                        s.id,
                        ShapeKind::Path {
                            path: to_page.inverse() * rect.to_path(0.01),
                            closed: true,
                        },
                    );
                    frame.fill = Fill::None;
                    frame.stroke = None;
                    let inner = framed_contents(s, new_id());
                    let mut clip = s.clone();
                    clip.effects.clear();
                    clip.kind = ShapeKind::ClipFrame {
                        frame: Box::new(frame),
                        contents: vec![inner],
                    };
                    Cropped::Into(Box::new(clip))
                }
                _ => {
                    let (clipped, closed) = clip_path(&page, rect);
                    if clipped.elements().is_empty() {
                        return Cropped::Gone;
                    }
                    let mut n = s.clone();
                    n.kind = ShapeKind::Path {
                        path: to_page.inverse() * clipped,
                        closed,
                    };
                    Cropped::Into(Box::new(n))
                }
            }
        }
    }
}

impl App {
    /// Crop to the area and clear it.
    pub fn apply_crop(&mut self) {
        if let Some(r) = self.crop_area.take() {
            self.crop_to(r);
        }
        self.crop_drag = None;
    }

    /// Keep what lies inside `rect` (page space) of the selection, or of
    /// everything on the page's unlocked, visible layers.
    pub fn crop_to(&mut self, rect: Rect) {
        if !finite_rect(rect) || rect.width() < 0.01 || rect.height() < 0.01 {
            return;
        }
        let targets: Vec<Shape> = if self.selection.is_empty() {
            self.doc()
                .page(self.page)
                .map(|p| {
                    p.layers
                        .iter()
                        .filter(|l| !l.locked && l.visible)
                        .flat_map(|l| l.shapes.iter().filter(|s| !s.locked).cloned())
                        .collect()
                })
                .unwrap_or_default()
        } else {
            self.selected_shapes()
        };
        let mut cmds = Vec::new();
        let mut removed = Vec::new();
        for s in targets {
            let parent = self
                .doc()
                .parent_transform(s.id)
                .unwrap_or(Affine::IDENTITY);
            let engine = &mut self.engine;
            let mut new_id = || engine.new_shape_id();
            let pixels = self.settings.crop_picture_pixels;
            match crop_shape_with(&s, parent, rect, &mut new_id, pixels) {
                Cropped::Keep => {}
                Cropped::Gone => removed.push(s.id),
                Cropped::Into(n) => {
                    // A picture newly put in a clip frame: the picture
                    // inside keeps the transparency, the frame drops it.
                    let framed = matches!(n.kind, ShapeKind::ClipFrame { .. })
                        && !matches!(s.kind, ShapeKind::ClipFrame { .. });
                    // A picture whose pixels were cut keeps its inks,
                    // cut alike.
                    let inks_cut = matches!(n.kind, ShapeKind::Bitmap { .. })
                        && n.picture_inks != s.picture_inks;
                    cmds.push(Command::SetShapeKind {
                        shape: s.id,
                        kind: n.kind,
                    });
                    if inks_cut {
                        cmds.push(Command::SetPictureInks {
                            shape: s.id,
                            cmyk: n.cmyk_bitmap,
                            inks: n.picture_inks,
                        });
                    }
                    if framed {
                        cmds.extend(frame_drops_transparency(&s));
                    }
                }
            }
        }
        if !removed.is_empty() {
            cmds.push(Command::DeleteShapes {
                shapes: removed.clone(),
            });
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Crop", &cmds) {
            self.status = e.to_string();
            return;
        }
        self.selection.retain(|id| !removed.contains(id));
    }

    /// The selected clip frames that hold one picture alone: what
    /// Bitmaps > Crop to Frame works on.
    pub fn framed_pictures(&self) -> Vec<ShapeId> {
        self.selected_shapes()
            .iter()
            .filter(|s| match &s.kind {
                ShapeKind::ClipFrame { contents, .. } => {
                    matches!(contents.as_slice(), [c] if matches!(c.kind, ShapeKind::Bitmap { .. }))
                }
                _ => false,
            })
            .map(|s| s.id)
            .collect()
    }

    /// Bitmaps > Crop to Frame (#1913): each selected clip frame holding a
    /// picture alone loses the picture's pixels outside the frame; a
    /// frame that is a rectangle along the picture's sides then lets the
    /// picture go, a free picture in the frame's place (its transparency
    /// and opacity, which the picture inside keeps, with it). Another
    /// frame stays, holding the smaller picture. One undo step.
    pub fn crop_pictures_to_frames(&mut self) {
        let mut cmds = Vec::new();
        for id in self.framed_pictures() {
            let Some(s) = self.doc().find_shape(id).cloned() else {
                continue;
            };
            let ShapeKind::ClipFrame { frame, contents } = &s.kind else {
                continue;
            };
            let [pic] = contents.as_slice() else {
                continue;
            };
            // The frame in the picture's own space.
            let m = pic.transform;
            if m.determinant().abs() < 1e-12 {
                continue;
            }
            let path = m.inverse() * (frame.transform * frame.local_path());
            let area = path.bounding_box();
            if !finite_rect(area) {
                continue;
            }
            let rectangular =
                (path.area().abs() - area.area()).abs() <= 1e-6 * area.area().max(1e-9);
            let cut = crop_picture(pic, area);
            let mut inner = pic.clone();
            if let Some((kind, inks)) = cut {
                inner.kind = kind;
                inner.picture_inks = inks;
            } else if !rectangular {
                continue;
            }
            if rectangular {
                // The picture in the frame's place, free.
                let free = inner;
                cmds.push(Command::SetShapeKind {
                    shape: s.id,
                    kind: free.kind.clone(),
                });
                // The picture's own placement in the frame, laid on the
                // object (a page-space transform: the parent's too).
                let parent = self
                    .doc()
                    .parent_transform(s.id)
                    .unwrap_or(Affine::IDENTITY);
                let was = parent * s.transform;
                cmds.push(Command::TransformShapes {
                    shapes: vec![s.id],
                    transform: was * m * was.inverse(),
                });
                cmds.push(Command::SetPictureInks {
                    shape: s.id,
                    cmyk: free.cmyk_bitmap,
                    inks: free.picture_inks.clone(),
                });
                cmds.push(Command::SetEffects {
                    shape: s.id,
                    effects: free.effects.clone(),
                });
                cmds.push(Command::SetOpacity {
                    shapes: vec![s.id],
                    opacity: free.opacity,
                });
            } else {
                let mut kept = s.clone();
                kept.kind = ShapeKind::ClipFrame {
                    frame: frame.clone(),
                    contents: vec![inner],
                };
                cmds.push(Command::SetShapeKind {
                    shape: s.id,
                    kind: kept.kind,
                });
            }
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Crop to Frame", &cmds) {
            self.status = e.to_string();
        }
    }

    /// The handle of the crop area near `p` (page space), if any.
    fn crop_handle_at(&self, p: Point) -> Option<usize> {
        let r = self.crop_area?;
        let tol = 6.0 / self.view.zoom.max(1e-6) as f64;
        handles(r).iter().position(|h| (*h - p).hypot() <= tol)
    }

    /// The Crop tool's clicks and drags.
    pub fn crop_input(&mut self, response: &egui::Response, p: Point) {
        use egui::PointerButton;
        if response.drag_started_by(PointerButton::Primary) {
            let origin = response
                .ctx
                .input(|i| i.pointer.press_origin())
                .map(|o| self.view.to_page(o))
                .unwrap_or(p);
            self.crop_drag = Some(match (self.crop_handle_at(origin), self.crop_area) {
                (Some(which), Some(from)) => CropDrag::Handle { which, from },
                (None, Some(from)) if from.contains(origin) => CropDrag::Move {
                    start: origin,
                    from,
                },
                _ => CropDrag::New {
                    start: self.snap_point(origin),
                },
            });
        }
        if response.dragged_by(PointerButton::Primary) {
            self.crop_drag_to(p);
        }
        if response.drag_stopped() {
            self.crop_drag = None;
            if self
                .crop_area
                .is_some_and(|r| r.width() < 0.1 || r.height() < 0.1)
            {
                self.crop_area = None;
            }
        }
        if response.double_clicked_by(PointerButton::Primary) {
            if self.crop_area.is_some_and(|r| r.contains(p)) {
                self.apply_crop();
            }
        } else if response.clicked_by(PointerButton::Primary)
            && self.crop_area.is_some_and(|r| !r.contains(p))
        {
            self.crop_area = None;
        }
    }

    /// The pointer moved to `p` (page space) during a drag.
    pub fn crop_drag_to(&mut self, p: Point) {
        let Some(d) = self.crop_drag else {
            return;
        };
        let r = match d {
            CropDrag::New { start } => Rect::from_points(start, self.snap_point(p)),
            CropDrag::Move { start, from } => from + (p - start),
            CropDrag::Handle { which, from } => resized(from, which, self.snap_point(p)),
        };
        if finite_rect(r) {
            self.crop_area = Some(r);
        }
    }
}

/// `r` moved by `v`, for the bar's fields.
pub fn moved(r: Rect, v: Vec2) -> Rect {
    r + v
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::tools::Tool;
    use tracedraw_core::Color;

    /// A red picture over (10, 10) to (50, 40), half see-through and with
    /// a left-to-right gradient transparency (#1491).
    pub(crate) fn see_through_picture(app: &mut App) -> ShapeId {
        use tracedraw_core::live::Effect;
        use tracedraw_core::style::{Fountain, FountainKind};
        let png = tiny_skia::Pixmap::new(2, 2)
            .and_then(|mut p| {
                p.fill(tiny_skia::Color::from_rgba8(255, 0, 0, 255));
                p.encode_png().ok()
            })
            .unwrap_or_default();
        let id = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(10.0, 10.0, 50.0, 40.0),
                width_px: 2,
                height_px: 2,
                png: png.into(),
                fx: None,
            })
            .expect("picture");
        app.run(Command::SetOpacity {
            shapes: vec![id],
            opacity: 0.5,
        });
        app.run(Command::SetEffects {
            shape: id,
            effects: vec![Effect::Transparency {
                mask: Fill::Fountain(Fountain::two(
                    FountainKind::Linear,
                    Color::BLACK,
                    Color::WHITE,
                    0.0,
                )),
                merge: Default::default(),
                target: 2,
            }],
        });
        id
    }

    /// How much green the page shows, at 10 pixels a millimetre, half way
    /// down the drawn (not white) pixels of the column at `x` mm: lower
    /// is more of the red picture.
    pub(crate) fn green_at(app: &App, x: f64) -> u8 {
        let page = app.doc().pages[0].id;
        let img = tracedraw_render::render_page_image(app.doc(), page, 254.0).expect("page");
        let column: Vec<u8> = (0..img.height())
            .filter_map(|y| img.pixel((x * 10.0) as u32, y))
            .map(|c| c.demultiply().green())
            .filter(|g| *g < 250)
            .collect();
        column.get(column.len() / 2).copied().expect("drawn")
    }

    /// #1491: cropping a picture with a gradient transparency and a
    /// uniform one leaves what is kept looking as it did: the copy of the
    /// picture inside the clip frame does not add them a second time.
    #[test]
    fn cropping_a_see_through_picture_keeps_its_look() {
        let mut app = App::headless();
        let id = see_through_picture(&mut app);
        let before = [green_at(&app, 35.0), green_at(&app, 45.0)];
        app.select(vec![id]);
        app.crop_to(Rect::new(30.0, 0.0, 60.0, 60.0));
        let s = app.doc().find_shape(id).expect("clip frame");
        let ShapeKind::ClipFrame { contents, .. } = &s.kind else {
            panic!("not a clip frame: {:?}", s.kind);
        };
        // The picture inside holds the transparency (files keep it there),
        // the frame none.
        assert!(s.effects.is_empty() && s.opacity == 1.0);
        assert!(contents[0].opacity == 0.5 && contents[0].effects.len() == 1);
        let after = [green_at(&app, 35.0), green_at(&app, 45.0)];
        for (b, a) in before.iter().zip(&after) {
            assert!(b.abs_diff(*a) <= 2, "before {before:?}, after {after:?}");
        }
    }

    /// A 4 by 2 picture on (0, 0) to (40, 20): each pixel 10 mm, its
    /// columns red, green, blue and white, the top row opaque and the
    /// bottom half see-through; CMYK inks that tell the columns apart.
    fn striped_picture(app: &mut App) -> ShapeId {
        let cols = [[255, 0, 0], [0, 255, 0], [0, 0, 255], [255, 255, 255]];
        let img = image::RgbaImage::from_fn(4, 2, |x, y| {
            let [r, g, b] = cols[x as usize];
            image::Rgba([r, g, b, if y == 0 { 255 } else { 128 }])
        });
        let png = crate::bitmap_fx::encode(&img).expect("png");
        // Planes C, M, Y, K one under the other: column x has x * 60 of
        // each ink, plus the plane's number.
        let planes = image::GrayImage::from_fn(4, 8, |x, y| image::Luma([(x * 60 + y / 2) as u8]));
        let mut bytes = std::io::Cursor::new(Vec::new());
        planes
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("planes");
        let inks = tracedraw_core::picture_inks::PictureInks::cmyk(&png, bytes.into_inner().into());
        let id = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(0.0, 0.0, 40.0, 20.0),
                width_px: 4,
                height_px: 2,
                png: png.into(),
                fx: None,
            })
            .expect("picture");
        app.run(Command::SetPictureInks {
            shape: id,
            cmyk: true,
            inks: Some(Box::new(inks)),
        });
        id
    }

    fn pixels_of(s: &Shape) -> (Rect, image::RgbaImage) {
        let ShapeKind::Bitmap {
            rect,
            width_px,
            height_px,
            png,
            ..
        } = &s.kind
        else {
            panic!("not a picture: {:?}", s.kind);
        };
        let img = crate::bitmap_fx::decode(png).expect("pixels");
        assert_eq!((img.width(), img.height()), (*width_px, *height_px));
        (*rect, img)
    }

    fn planes_of(s: &Shape) -> Vec<u8> {
        let inks = s.picture_inks.as_deref().expect("inks");
        let ShapeKind::Bitmap { png, .. } = &s.kind else {
            panic!("picture");
        };
        assert!(inks.fits(png.as_slice()), "the inks fit the pixels");
        image::load_from_memory(&inks.png)
            .expect("planes")
            .to_luma8()
            .into_raw()
    }

    /// #1913: with Crop picture pixels on, the Crop tool cuts a picture's
    /// pixels to the area (on whole pixels) and the picture stays free;
    /// its transparency and its CMYK inks are cut alike. Off, it goes in
    /// a clip frame as before. Undo puts the whole picture back.
    #[test]
    fn the_crop_tool_cuts_a_pictures_pixels() {
        let mut app = App::headless();
        let id = striped_picture(&mut app);
        let whole = app.doc().find_shape(id).cloned().expect("picture");
        app.settings.crop_picture_pixels = true;
        app.select(vec![id]);
        // Columns 1 and 2, the top row: the area's edges are inside pixels.
        app.crop_to(Rect::new(12.0, 11.0, 28.0, 30.0));
        let s = app.doc().find_shape(id).cloned().expect("cropped");
        let (rect, img) = pixels_of(&s);
        assert_eq!(rect, Rect::new(10.0, 10.0, 30.0, 20.0), "whole pixels");
        assert_eq!((img.width(), img.height()), (2, 1));
        assert_eq!(img.get_pixel(0, 0).0, [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(1, 0).0, [0, 0, 255, 255]);
        assert!(s.cmyk_bitmap);
        // C, M, Y, K of columns 1 and 2, top row.
        assert_eq!(planes_of(&s), [60, 120, 61, 121, 62, 122, 63, 123]);
        // The bottom row keeps its transparency.
        app.engine.undo().expect("undo");
        assert_eq!(app.doc().find_shape(id), Some(&whole));
        app.crop_to(Rect::new(0.0, 0.0, 10.0, 10.0));
        let (rect, img) = pixels_of(app.doc().find_shape(id).expect("cropped"));
        assert_eq!(rect, Rect::new(0.0, 0.0, 10.0, 10.0));
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 128]);

        // Off: a clip frame, every pixel kept.
        app.engine.undo().expect("undo");
        app.settings.crop_picture_pixels = false;
        app.crop_to(Rect::new(12.0, 11.0, 28.0, 30.0));
        let s = app.doc().find_shape(id).expect("framed");
        assert!(matches!(s.kind, ShapeKind::ClipFrame { .. }));
    }

    /// A turned picture is not cut (its pixels do not line up with the
    /// area): it goes in a clip frame even with the option on.
    #[test]
    fn a_turned_picture_still_goes_in_a_frame() {
        let mut app = App::headless();
        let id = striped_picture(&mut app);
        app.run(Command::TransformShapes {
            shapes: vec![id],
            transform: Affine::rotate(0.3),
        });
        app.settings.crop_picture_pixels = true;
        app.select(vec![id]);
        app.crop_to(Rect::new(10.0, 5.0, 25.0, 20.0));
        let s = app.doc().find_shape(id).expect("framed");
        assert!(matches!(s.kind, ShapeKind::ClipFrame { .. }));
    }

    /// #1913: Bitmaps > Crop to Frame cuts a framed picture's pixels to
    /// the frame and, the frame being a rectangle, the picture goes free
    /// where it was, with its transparency; a frame of another shape
    /// stays, holding the smaller picture. One undo step.
    #[test]
    fn crop_to_frame_frees_the_picture() {
        let mut app = App::headless();
        let id = striped_picture(&mut app);
        app.select(vec![id]);
        app.crop_to(Rect::new(12.0, 0.0, 40.0, 20.0));
        app.run(Command::SetOpacity {
            shapes: vec![id],
            opacity: 1.0,
        });
        // Moved after framing: the free picture follows.
        app.run(Command::TransformShapes {
            shapes: vec![id],
            transform: Affine::translate((100.0, 50.0)),
        });
        assert_eq!(app.framed_pictures(), vec![id]);
        app.crop_pictures_to_frames();
        let s = app.doc().find_shape(id).cloned().expect("free");
        let (rect, img) = pixels_of(&s);
        assert_eq!((img.width(), img.height()), (3, 2));
        assert_eq!(img.get_pixel(0, 0).0, [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 1).0, [0, 255, 0, 128]);
        let b = s.transform.transform_rect_bbox(rect);
        assert!(
            (b.x0 - 110.0).abs() < 1e-9 && (b.x1 - 140.0).abs() < 1e-9,
            "where the frame showed it: {b:?}"
        );
        assert!((b.y0 - 50.0).abs() < 1e-9 && (b.y1 - 70.0).abs() < 1e-9);
        assert_eq!(planes_of(&s).len(), 3 * 2 * 4);
        assert!(app.framed_pictures().is_empty());
        app.engine.undo().expect("undo");
        assert!(matches!(
            app.doc().find_shape(id).expect("frame").kind,
            ShapeKind::ClipFrame { .. }
        ));

        // An oval frame: the pixels outside its box go, the frame stays.
        let pic = striped_picture(&mut app);
        let oval = app
            .new_shape(ShapeKind::Ellipse {
                rect: Rect::new(10.0, 0.0, 30.0, 20.0),
                arc: None,
            })
            .expect("oval");
        app.place_inside(vec![pic], oval);
        let frame = oval;
        app.select(vec![frame]);
        assert_eq!(app.framed_pictures(), vec![frame]);
        app.crop_pictures_to_frames();
        let s = app.doc().find_shape(frame).expect("frame");
        let ShapeKind::ClipFrame { contents, .. } = &s.kind else {
            panic!("still a frame");
        };
        let (_, img) = pixels_of(&contents[0]);
        assert_eq!((img.width(), img.height()), (2, 2));
    }

    fn rect_shape(app: &mut App, r: Rect) -> ShapeId {
        let id = app
            .new_shape(ShapeKind::Rect {
                rect: r,
                radius: 0.0,
                corners: None,
            })
            .expect("rectangle");
        app.run(Command::SetFill {
            shapes: vec![id],
            fill: Fill::Solid(Color::BLACK),
        });
        id
    }

    /// #1491: cropping an object with a gradient transparency keeps the
    /// part of the transparency over what is left, as cropping a picture
    /// does: the ramp is cut, not started again across the new box.
    #[test]
    fn crop_cuts_the_gradient_transparency() {
        use tracedraw_core::live::Effect;
        use tracedraw_core::style::{Fountain, FountainKind};
        let mut app = App::headless();
        let id = rect_shape(&mut app, Rect::new(0.0, 0.0, 100.0, 50.0));
        let ramp = Fill::Fountain(Fountain::two(
            FountainKind::Linear,
            Color::BLACK,
            Color::WHITE,
            0.0,
        ));
        app.run(Command::SetEffects {
            shape: id,
            effects: vec![Effect::Transparency {
                mask: ramp,
                merge: Default::default(),
                target: 2,
            }],
        });
        app.select(vec![id]);
        app.crop_to(Rect::new(60.0, 10.0, 90.0, 40.0));
        let s = app.doc().find_shape(id).cloned().expect("cropped");
        let b = s.bounds();
        assert!(
            (b.x0 - 60.0).abs() < 1e-6 && (b.x1 - 90.0).abs() < 1e-6,
            "{b:?}"
        );
        let Some(Effect::Transparency {
            mask: Fill::Fountain(f),
            ..
        }) = s.effects.first()
        else {
            panic!("a gradient transparency");
        };
        // Where the ramp was 0.6 and 0.9 of the way, it still is.
        for x in [60.0, 75.0, 90.0] {
            let t = f.param_at(b, Point::new(x, 25.0)).expect("placed");
            assert!((t - x / 100.0).abs() < 1e-9, "{x}: {t}");
        }
    }

    #[test]
    fn the_area_waits_and_moves_until_it_is_applied() {
        let mut app = App::headless();
        app.tool = Tool::Crop;
        let id = rect_shape(&mut app, Rect::new(0.0, 0.0, 40.0, 40.0));
        // A drag draws the area; nothing is cropped yet.
        app.crop_drag = Some(CropDrag::New {
            start: Point::new(10.0, 10.0),
        });
        app.crop_drag_to(Point::new(30.0, 30.0));
        app.crop_drag = None;
        assert_eq!(app.crop_area, Some(Rect::new(10.0, 10.0, 30.0, 30.0)));
        assert!(matches!(
            app.doc().find_shape(id).map(|s| &s.kind),
            Some(ShapeKind::Rect { .. })
        ));
        // Moved, then a handle dragged.
        let from = app.crop_area.expect("area");
        app.crop_drag = Some(CropDrag::Move {
            start: Point::new(20.0, 20.0),
            from,
        });
        app.crop_drag_to(Point::new(25.0, 20.0));
        app.crop_drag = Some(CropDrag::Handle {
            which: 4,
            from: app.crop_area.expect("area"),
        });
        app.crop_drag_to(Point::new(50.0, 32.0));
        assert_eq!(app.crop_area, Some(Rect::new(15.0, 10.0, 50.0, 32.0)));
        assert_eq!(
            handles(Rect::new(0.0, 0.0, 2.0, 2.0))[3],
            Point::new(2.0, 1.0)
        );
        // Applied: the square is cut to the area, one undo step.
        let steps = app.engine.history_labels().0.len();
        app.apply_crop();
        assert!(app.crop_area.is_none());
        assert_eq!(app.engine.history_labels().0.len(), steps + 1);
        let b = app.doc().find_shape(id).expect("square").bounds();
        assert!(
            (b.x0 - 15.0).abs() < 1e-6 && (b.x1 - 40.0).abs() < 1e-6,
            "{b:?}"
        );
        assert!(
            (b.y0 - 10.0).abs() < 1e-6 && (b.y1 - 32.0).abs() < 1e-6,
            "{b:?}"
        );
    }

    #[test]
    fn open_curves_groups_bitmaps_and_text() {
        let mut app = App::headless();
        // An open line across the area: the part inside stays, open.
        let mut line = BezPath::new();
        line.move_to((-10.0, 5.0));
        line.line_to((30.0, 5.0));
        let lid = app
            .new_shape(ShapeKind::Path {
                path: line,
                closed: false,
            })
            .expect("line");
        // A group of a square inside and one outside the area.
        let a = rect_shape(&mut app, Rect::new(1.0, 1.0, 4.0, 4.0));
        let b = rect_shape(&mut app, Rect::new(50.0, 50.0, 60.0, 60.0));
        app.run(Command::Group { shapes: vec![a, b] });
        let gid = app
            .doc()
            .all_layers()
            .flat_map(|l| &l.shapes)
            .find(|s| matches!(s.kind, ShapeKind::Group { .. }))
            .map(|s| s.id)
            .expect("group");
        // A bitmap half in the area.
        let bid = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(10.0, -10.0, 30.0, 10.0),
                width_px: 2,
                height_px: 2,
                png: Vec::new().into(),
                fx: None,
            })
            .expect("bitmap");
        app.crop_to(Rect::new(0.0, 0.0, 20.0, 20.0));
        let s = app.doc().find_shape(lid).expect("line");
        let ShapeKind::Path { path, closed } = &s.kind else {
            panic!("a path");
        };
        assert!(!closed);
        let lb = path.bounding_box();
        assert!(
            (lb.x0 - 0.0).abs() < 1e-6 && (lb.x1 - 20.0).abs() < 1e-6,
            "{lb:?}"
        );
        // The group keeps the square inside, as it was.
        let Some(ShapeKind::Group { children }) = app.doc().find_shape(gid).map(|s| s.kind.clone())
        else {
            panic!("the group");
        };
        assert_eq!(children.len(), 1);
        assert!(matches!(children[0].kind, ShapeKind::Rect { .. }));
        // The bitmap is in a clip frame of the area, whole.
        let Some(ShapeKind::ClipFrame { frame, contents }) =
            app.doc().find_shape(bid).map(|s| s.kind.clone())
        else {
            panic!("a clip frame");
        };
        assert!(matches!(contents[0].kind, ShapeKind::Bitmap { .. }));
        let fb = frame.local_path().bounding_box();
        assert!(
            (fb.x1 - 20.0).abs() < 1e-6 && (fb.y0 - 0.0).abs() < 1e-6,
            "{fb:?}"
        );
        // Text half in the area becomes its outlines, cut (when a font
        // gives outlines here).
        let tid = app
            .new_shape(ShapeKind::Text {
                spans: vec![tracedraw_core::TextSpan::new(
                    "Crop me".to_string(),
                    app.text_font.clone(),
                    24.0,
                )],
                origin: Point::new(10.0, 5.0),
                frame: None,
                align: tracedraw_core::TextAlign::Left,
                para: Default::default(),
                on_path: None,
            })
            .expect("text");
        let outlines = app
            .doc()
            .find_shape(tid)
            .map(|s| !s.local_path().elements().is_empty())
            .unwrap_or(false);
        app.select(vec![tid]);
        app.crop_to(Rect::new(0.0, 0.0, 20.0, 20.0));
        if outlines {
            let t = app.doc().find_shape(tid).expect("cropped text");
            assert!(matches!(t.kind, ShapeKind::Path { closed: true, .. }));
            let tb = t.bounds();
            assert!(tb.x1 <= 20.0 + 1e-6, "{tb:?}");
        }
        // Wholly outside: deleted.
        let far = rect_shape(&mut app, Rect::new(100.0, 100.0, 110.0, 110.0));
        app.select(vec![far]);
        app.crop_to(Rect::new(0.0, 0.0, 20.0, 20.0));
        assert!(app.doc().find_shape(far).is_none());
        assert!(app.selection.is_empty());
    }

    #[test]
    fn handles_and_resizing_stay_sane() {
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        // Dragging a corner past the opposite one flips the area round.
        assert_eq!(
            resized(r, 0, Point::new(20.0, 15.0)),
            Rect::new(10.0, 10.0, 20.0, 15.0)
        );
        assert_eq!(
            moved(r, Vec2::new(1.0, 2.0)),
            Rect::new(1.0, 2.0, 11.0, 12.0)
        );
        // Odd areas crop nothing.
        let mut app = App::headless();
        let id = rect_shape(&mut app, r);
        app.crop_to(Rect::new(f64::NAN, 0.0, 1.0, 1.0));
        app.crop_to(Rect::new(0.0, 0.0, 0.0, 5.0));
        assert!(matches!(
            app.doc().find_shape(id).map(|s| &s.kind),
            Some(ShapeKind::Rect { .. })
        ));
    }
}
