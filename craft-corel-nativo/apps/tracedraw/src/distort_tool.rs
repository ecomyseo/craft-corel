//! The Distort tool: Push and Pull, Zipper and Twister as a live effect
//! (`Effect::Distort`, formulas in `tracedraw_core::distort`).
//!
//! A drag on an object, or anywhere with one object selected, distorts
//! it about the point the drag started on (off the object too, #1442):
//! Push and Pull by the drag's length, a full half width of the object
//! being 100, pushed when the pointer is right of the start and pulled
//! when left; Zipper with teeth a quarter of the drag's length high,
//! leaning the way the drag goes from the outline's normal (#1671);
//! Twister by the angle the pointer turned about the start, whole turns
//! included. The handle line goes from the start to the pointer in any
//! direction, not along an axis (#1442, `HandleLine`). A selected
//! distortion shows its handles: the centre (a diamond, dragged to move
//! it), a line to the end handle (a square whose distance from the
//! centre is the Push and Pull amount or the Zipper amplitude, the
//! Zipper's line leaning with its teeth; a circle whose angle about the
//! centre is the Twister angle, whole turns counted while it is dragged
//! round) and, for the Zipper, a slider on the line for the frequency
//! (#1122). Each drag is one undo step; a
//! drag off the handles adds a new distortion over the ones the object
//! has, the handles and the bar edit the newest (#1442).
//!
//! The property bar shows the selected object's distortion (or the
//! tool's settings) and edits the selected distortions live.

use crate::app::App;
use serde::{Deserialize, Serialize};
use tracedraw_core::{
    distort::{Distort, MAX_FREQUENCY, MAX_TILT},
    geometry::{BezPath, Point, Shape as _, Vec2},
    live::Effect,
    Command, Shape, ShapeId, ShapeKind,
};

/// The kind of distortion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DistortMode {
    #[default]
    PushPull,
    Zipper,
    Twister,
}

impl DistortMode {
    pub const ALL: [DistortMode; 3] = [
        DistortMode::PushPull,
        DistortMode::Zipper,
        DistortMode::Twister,
    ];

    pub fn key(self) -> &'static str {
        match self {
            DistortMode::PushPull => "toolbar.push_pull",
            DistortMode::Zipper => "toolbar.zipper",
            DistortMode::Twister => "toolbar.twister",
        }
    }

    pub fn of(d: &Distort) -> DistortMode {
        match d {
            Distort::PushPull { .. } => DistortMode::PushPull,
            Distort::Zipper { .. } => DistortMode::Zipper,
            Distort::Twister { .. } => DistortMode::Twister,
        }
    }
}

/// The Distort tool's values, one set per kind.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DistortSettings {
    pub mode: DistortMode,
    /// Push and Pull, percent of the nodes' distance (-200 to 200).
    pub amount: f64,
    /// Zipper: the teeth's height (mm), how many on each segment, its
    /// options and the teeth's lean from the outline's normal (degrees,
    /// -180 to 180; older preferences have none, #1671).
    pub amplitude: f64,
    pub frequency: u32,
    pub random: bool,
    pub smooth: bool,
    pub local: bool,
    #[serde(default)]
    pub tilt: f64,
    /// Zipper: the teeth of a `.cdr` file (see
    /// `Distort::Zipper::shifted_tips`), kept while its values are
    /// changed; not a setting of its own.
    #[serde(skip)]
    pub shifted_tips: bool,
    /// Twister: whole turns, degrees more (0 to 359) and the direction.
    pub turns: u32,
    pub degrees: f64,
    pub clockwise: bool,
}

impl Default for DistortSettings {
    fn default() -> Self {
        DistortSettings {
            mode: DistortMode::PushPull,
            amount: 20.0,
            amplitude: 2.0,
            frequency: 6,
            random: false,
            smooth: false,
            local: false,
            tilt: 0.0,
            shifted_tips: false,
            turns: 0,
            degrees: 90.0,
            clockwise: true,
        }
    }
}

/// The highest number of whole turns the bar takes.
pub const MAX_TURNS: u32 = 9;

impl DistortSettings {
    /// The same values in their ranges (a preferences file edited by
    /// hand).
    pub fn clamped(self) -> Self {
        let d = DistortSettings::default();
        let fin = |v: f64, lo: f64, hi: f64, or: f64| {
            if v.is_finite() {
                v.clamp(lo, hi)
            } else {
                or
            }
        };
        DistortSettings {
            amount: fin(self.amount, -200.0, 200.0, d.amount),
            amplitude: fin(self.amplitude, 0.0, 100.0, d.amplitude),
            frequency: self.frequency.min(MAX_FREQUENCY),
            tilt: fin(self.tilt, -MAX_TILT, MAX_TILT, d.tilt),
            turns: self.turns.min(MAX_TURNS),
            degrees: fin(self.degrees, 0.0, 359.0, d.degrees),
            ..self
        }
    }

    /// The Zipper's lean as the bar shows it: the tilt folded into -90 to
    /// 90 degrees, since teeth leaning `t` and `t + 180` are the same
    /// zigzag with its sides swapped; the tilt itself keeps which side of
    /// the outline the handle line is on (#1671).
    pub fn lean(&self) -> f64 {
        if !self.tilt.is_finite() {
            return 0.0;
        }
        (self.tilt - 180.0 * (self.tilt / 180.0).round()).round()
    }

    /// The lean typed on the bar: the tilt takes it, keeping the side the
    /// handle line is on.
    pub fn set_lean(&mut self, lean: f64) {
        if !lean.is_finite() {
            return;
        }
        let side = if self.tilt.is_finite() {
            180.0 * (self.tilt / 180.0).round()
        } else {
            0.0
        };
        let mut tilt = lean.clamp(-90.0, 90.0) + side;
        while tilt > 180.0 {
            tilt -= 360.0;
        }
        while tilt <= -180.0 {
            tilt += 360.0;
        }
        self.tilt = tilt;
    }

    /// The Twister angle, degrees (counterclockwise positive).
    pub fn angle(&self) -> f64 {
        let a = self.turns as f64 * 360.0 + self.degrees;
        if self.clockwise {
            -a
        } else {
            a
        }
    }

    /// The distortion of kind `mode` with these values about `center`.
    pub fn effect(&self, mode: DistortMode, center: Option<Point>) -> Distort {
        match mode {
            DistortMode::PushPull => Distort::PushPull {
                amount: self.amount,
                center,
            },
            DistortMode::Zipper => Distort::Zipper {
                amplitude: self.amplitude,
                frequency: self.frequency,
                center,
                random: self.random,
                smooth: self.smooth,
                local: self.local,
                tilt_deg: self.tilt,
                // A new distortion (no centre yet) has Craft Corel's teeth.
                shifted_tips: self.shifted_tips && center.is_some(),
            },
            DistortMode::Twister => Distort::Twister {
                angle_deg: self.angle(),
                center,
            },
        }
    }

    /// Take the values of `d` (its kind becomes the mode).
    pub fn take(&mut self, d: &Distort) {
        self.mode = DistortMode::of(d);
        match *d {
            Distort::PushPull { amount, .. } => self.amount = amount,
            Distort::Zipper {
                amplitude,
                frequency,
                random,
                smooth,
                local,
                tilt_deg,
                shifted_tips,
                ..
            } => {
                self.shifted_tips = shifted_tips;
                self.amplitude = amplitude;
                self.frequency = frequency;
                self.random = random;
                self.smooth = smooth;
                self.local = local;
                self.tilt = tilt_deg;
            }
            Distort::Twister { angle_deg, .. } => {
                let a = if angle_deg.is_finite() {
                    angle_deg
                } else {
                    0.0
                };
                self.clockwise = a < 0.0;
                let a = a.abs().min(MAX_TURNS as f64 * 360.0 + 359.0);
                self.turns = (a / 360.0).floor() as u32;
                self.degrees = (a - self.turns as f64 * 360.0).round().min(359.0);
            }
        }
        *self = self.clamped();
    }
}

/// What a drag of the Distort tool holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grab {
    /// The object: a new distortion about where the drag started.
    Object,
    /// The centre marker: the centre moves with the pointer.
    Center,
    /// The handle line between the centre and the end, away from both:
    /// the centre moves by the pointer's way, the line with it.
    Line,
    /// The end handle: the amount or amplitude from its distance to the
    /// centre, the Twister angle from its direction.
    End,
    /// The Zipper's frequency slider.
    Frequency,
}

/// A drag of the Distort tool under way.
#[derive(Debug, Clone, PartialEq)]
pub struct DistortDrag {
    pub id: ShapeId,
    /// Where the drag started (page space) and the same point in the
    /// object's own space (the centre).
    pub start: Point,
    pub center: Point,
    /// Where the press was (page space).
    pub press: Point,
    /// Where the end handle was at the press (page space; the press
    /// point for a drag that makes a new distortion): Shift keeps the
    /// end at its height, Ctrl at its distance from the centre (#1563).
    pub end: Point,
    /// Half the object's larger side (page mm): Push and Pull's 100.
    pub reach: f64,
    pub grab: Grab,
    /// Twister: the turn so far (radians) and the pointer's last angle
    /// about the start.
    pub turned: f64,
    pub last_angle: Option<f64>,
    /// A drag that makes a new distortion: the object's effects before it
    /// (the new one goes over them, #1442) and whether the drag has
    /// changed the drawing yet (one undo step for the whole drag).
    pub base: Option<Vec<Effect>>,
    pub begun: bool,
}

/// Where the end handle of a distortion goes for the pointer at `p`
/// (page space), its line from `center`, the end at `end` when the drag
/// began (#1563, #1793): with `shift` alone it moves only sideways, at
/// the height it had; with `ctrl` alone it keeps its length and turns
/// freely toward the pointer, all the way round (a line still of no
/// length follows the pointer); with both it keeps to steps of the
/// constrain angle `step` (degrees) about the centre, as far out as the
/// pointer reaches along the step; with neither it is at the pointer.
pub fn constrain_end(
    center: Point,
    end: Point,
    p: Point,
    ctrl: bool,
    shift: bool,
    step: f64,
) -> Point {
    match (ctrl, shift) {
        (true, true) => crate::interaction::constrain_from(center, p, step),
        (false, true) => Point::new(p.x, end.y),
        (true, false) => {
            let len = (end - center).hypot();
            let to = p - center;
            if len < 1e-9 || to.hypot() < 1e-9 {
                p
            } else {
                center + to * (len / to.hypot())
            }
        }
        (false, false) => p,
    }
}

/// Where a drag left an object's handle line (#1442, #1263): the line is
/// free, it goes from the centre toward the pointer in any direction and
/// stays there when let go. It holds for distortions of its kind; one of
/// another kind takes the default line (to the right). The Zipper's line
/// is not kept here: it stands on the outline's normal turned by the
/// teeth's tilt, which the distortion itself holds (#1671).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandleLine {
    pub mode: DistortMode,
    /// The line's direction (radians, page space, counterclockwise from
    /// the right): the push side for Push and Pull (a negative amount
    /// goes the other way), the direction of a Twister angle of zero.
    pub angle: f64,
    /// The Twister's handle length (page mm), as far as it was dragged;
    /// half the object's larger side until it has been.
    pub len: Option<f64>,
}

impl HandleLine {
    /// The line's direction as a unit vector; to the right when the angle
    /// is not a number.
    pub fn dir(&self) -> Vec2 {
        if self.angle.is_finite() {
            Vec2::from_angle(self.angle)
        } else {
            Vec2::new(1.0, 0.0)
        }
    }
}

/// The handles of a selected distortion, page space.
#[derive(Debug, Clone, PartialEq)]
pub struct DistortHandles {
    pub id: ShapeId,
    pub mode: DistortMode,
    pub center: Point,
    /// The end of the handle line: a square at the amount (Push and
    /// Pull, half the object's larger side being 100, the other way when
    /// negative) from the centre along the line the drag left (to the
    /// right until one has, `HandleLine`), or four times the amplitude
    /// (Zipper, as the drag that made it) along the outline's normal at
    /// the point nearest the centre turned by the teeth's tilt, so the
    /// line leans with the teeth (#1671); a circle on the Twister's
    /// angle, half the larger side away.
    pub end: Point,
    /// Zipper: the frequency slider, `frequency / MAX_FREQUENCY` of the
    /// way from the centre to the end.
    pub slider: Option<Point>,
}

/// How near (pixels) a press must be to a handle to take it, at least:
/// larger handles are taken anywhere on them (`App::distort_pick_px`).
const HANDLE_PICK_PX: f64 = 6.0;

/// The distance of the Zipper's end handle for an amplitude.
const AMPLITUDE_REACH: f64 = 4.0;

/// Objects the tool distorts: not groups or bitmaps.
pub fn distortable(kind: &ShapeKind) -> bool {
    !matches!(kind, ShapeKind::Bitmap { .. } | ShapeKind::Group { .. })
}

/// The newest distortion among `effects` (the last; a new drag adds one
/// over those before it), if any.
pub fn distortion_of(effects: &[Effect]) -> Option<Distort> {
    effects.iter().rev().find_map(|e| match e {
        Effect::Distort(d) => Some(*d),
        _ => None,
    })
}

/// The outline the newest distortion among `effects` is applied to, in
/// the object's own space: the object's own outline as the distortions
/// before the newest leave it (each is applied to the result of the ones
/// before it, #1442). With `all`, as all of them leave it: the outline
/// a new distortion goes on.
fn distortion_input(s: &Shape, effects: &[Effect], all: bool) -> BezPath {
    let newest = effects
        .iter()
        .rposition(|e| matches!(e, Effect::Distort(_)))
        .filter(|_| !all)
        .unwrap_or(effects.len());
    let mut path = s.local_path();
    for e in &effects[..newest] {
        if let Effect::Distort(d) = e {
            path = tracedraw_core::distort::distort(&path, *d);
        }
    }
    path
}

/// A direction in the object's own space as it shows on the page (the
/// transform's turn, scale and skew, not its shift), as a unit vector;
/// to the right when it vanishes.
fn page_direction(s: &Shape, v: Vec2) -> Vec2 {
    let d = s.transform * (Point::ZERO + v) - s.transform * Point::ZERO;
    if d.hypot() > 1e-12 && d.x.is_finite() && d.y.is_finite() {
        d / d.hypot()
    } else {
        Vec2::new(1.0, 0.0)
    }
}

/// The Zipper's handle line on the page (a unit vector) for teeth that
/// lean `tilt_deg` on the outline `input` about `center` (both in the
/// object's own space): the outline's normal at the point nearest the
/// centre, turned by the tilt (#1671).
fn zipper_line_dir(s: &Shape, input: &BezPath, center: Point, tilt_deg: f64) -> Vec2 {
    let zero = tracedraw_core::distort::zipper_zero_direction(input, center);
    let tilt = if tilt_deg.is_finite() {
        tilt_deg.to_radians()
    } else {
        0.0
    };
    page_direction(s, Vec2::from_angle(zero.atan2() + tilt))
}

/// The teeth's lean (degrees, -180 to 180) that puts the Zipper's handle
/// line along `dir` on the page: the turn from the outline's normal at
/// the point nearest `center` to `dir`, measured in the object's own
/// space (#1671). None when `dir` vanishes or the object is flat.
fn zipper_tilt_toward(s: &Shape, input: &BezPath, center: Point, dir: Vec2) -> Option<f64> {
    if dir.hypot() < 1e-9 || s.transform.determinant().abs() < 1e-12 {
        return None;
    }
    let inv = s.transform.inverse();
    let local = inv * (Point::ZERO + dir) - inv * Point::ZERO;
    if local.hypot() < 1e-12 || !local.x.is_finite() || !local.y.is_finite() {
        return None;
    }
    let zero = tracedraw_core::distort::zipper_zero_direction(input, center);
    let mut tilt = (local.atan2() - zero.atan2()).to_degrees();
    while tilt > 180.0 {
        tilt -= 360.0;
    }
    while tilt <= -180.0 {
        tilt += 360.0;
    }
    Some(tilt.clamp(-MAX_TILT, MAX_TILT))
}

/// `effects` with the newest distortion replaced in place by `d` (added
/// at the end when there is none; removed with `None`, the ones before it
/// staying, so Clear Distortion takes them off one at a time).
pub fn with_distortion(effects: &[Effect], d: Option<Distort>) -> Vec<Effect> {
    let mut out: Vec<Effect> = effects.to_vec();
    match out.iter().rposition(|e| matches!(e, Effect::Distort(_))) {
        Some(i) => match d {
            Some(d) => out[i] = Effect::Distort(d),
            None => {
                out.remove(i);
            }
        },
        None => {
            if let Some(d) = d {
                out.push(Effect::Distort(d));
            }
        }
    }
    out
}

impl App {
    /// The distortion the bar shows: the first selected object's.
    pub fn selected_distortion(&self) -> Option<Distort> {
        self.selected_shapes()
            .iter()
            .find_map(|s| distortion_of(&s.effects))
    }

    /// The selected objects that are distorted.
    fn distorted_selection(&self) -> Vec<(ShapeId, Distort, Vec<Effect>)> {
        self.selected_shapes()
            .into_iter()
            .filter_map(|s| distortion_of(&s.effects).map(|d| (s.id, d, s.effects.clone())))
            .collect()
    }

    /// Change every selected distortion with `f`, one undo step labelled
    /// `label` (a run of the same change on the same objects with
    /// nothing in between is one step).
    pub fn change_distortions(
        &mut self,
        label: &'static str,
        f: impl Fn(&Distort) -> Option<Distort>,
    ) {
        let targets = self.distorted_selection();
        self.set_distortions(label, targets, f);
    }

    /// [`App::change_distortions`] on `targets` (object, its distortion,
    /// its effects).
    fn set_distortions(
        &mut self,
        label: &'static str,
        targets: Vec<(ShapeId, Distort, Vec<Effect>)>,
        f: impl Fn(&Distort) -> Option<Distort>,
    ) {
        let mut cmds = Vec::new();
        let mut ids = Vec::new();
        for (id, d, effects) in targets {
            let next = f(&d);
            if next == Some(d) && distortion_of(&effects) == Some(d) {
                continue;
            }
            ids.push(id);
            cmds.push(Command::SetEffects {
                shape: id,
                effects: with_distortion(&effects, next),
            });
        }
        if cmds.is_empty() {
            return;
        }
        let key = (self.engine.revision(), ids.clone(), label.to_string());
        if self.engine.undo_label() == Some(label) && self.effect_edit_step.as_ref() == Some(&key) {
            let _ = self.engine.undo();
        }
        if let Err(e) = self.engine.run_batch(label, &cmds) {
            self.status = e.to_string();
            return;
        }
        self.effect_edit_step = Some((self.engine.revision(), ids, label.to_string()));
    }

    /// The bar's values went to the tool; the selected distortions of the
    /// mode shown take them (keeping their centres), and selected objects
    /// without one take it about their middles, so an edit on the bar
    /// shows on the selection at once (#1030). Typing on, the edits are
    /// one undo step.
    pub fn apply_distort_settings(&mut self) {
        self.distort = self.distort.clamped();
        let s = self.distort;
        let targets: Vec<(ShapeId, Distort, Vec<Effect>)> = self
            .selected_shapes()
            .into_iter()
            .filter_map(|sh| match distortion_of(&sh.effects) {
                Some(d) => Some((sh.id, d, sh.effects.clone())),
                None if distortable(&sh.kind) => {
                    Some((sh.id, s.effect(s.mode, None), sh.effects.clone()))
                }
                None => None,
            })
            .collect();
        self.set_distortions("Distortion", targets, |d| {
            Some(s.effect(s.mode, d.center()))
        });
    }

    /// A mode button: the tool's mode, and the selected distortions turn
    /// into that kind with the tool's values for it.
    pub fn set_distort_mode(&mut self, mode: DistortMode) {
        self.distort.mode = mode;
        self.distort = self.distort.clamped();
        let s = self.distort;
        self.change_distortions("Distortion", |d| Some(s.effect(s.mode, d.center())));
    }

    /// Apply the bar's distortion to the selected objects that have none
    /// (about their middles).
    pub fn add_distortion(&mut self) {
        let s = self.distort.clamped();
        let mut cmds = Vec::new();
        for sh in self.selected_shapes() {
            if distortion_of(&sh.effects).is_some() || !distortable(&sh.kind) {
                continue;
            }
            cmds.push(Command::SetEffects {
                shape: sh.id,
                effects: with_distortion(&sh.effects, Some(s.effect(s.mode, None))),
            });
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Distortion", &cmds) {
            self.status = e.to_string();
        }
    }

    /// Center Distortion: the selected distortions about their objects'
    /// middles again.
    pub fn center_distortion(&mut self) {
        self.change_distortions("Center Distortion", |d| Some(d.with_center(None)));
    }

    /// Clear Distortion.
    pub fn clear_distortion(&mut self) {
        self.change_distortions("Clear Distortion", |_| None);
    }

    /// Convert to Curves: the selected objects' distortions become their
    /// outlines.
    pub fn bake_distortion(&mut self) {
        let mut cmds = Vec::new();
        for (id, _, effects) in self.distorted_selection() {
            let Some(s) = self.doc().find_shape(id) else {
                continue;
            };
            // Every distortion, oldest first, as they are drawn.
            let mut path = s.local_path();
            for e in &effects {
                if let Effect::Distort(d) = e {
                    path = tracedraw_core::distort::distort(&path, *d);
                }
            }
            let closed = path
                .elements()
                .iter()
                .any(|e| matches!(e, tracedraw_core::geometry::PathEl::ClosePath));
            cmds.push(Command::SetShapeKind {
                shape: id,
                kind: ShapeKind::Path { path, closed },
            });
            cmds.push(Command::SetEffects {
                shape: id,
                effects: effects
                    .iter()
                    .filter(|e| !matches!(e, Effect::Distort(_)))
                    .cloned()
                    .collect(),
            });
        }
        if cmds.is_empty() {
            return;
        }
        if let Err(e) = self.engine.run_batch("Convert To Curves", &cmds) {
            self.status = e.to_string();
        }
    }

    /// The handles of the selected distortions, page space.
    pub fn distortion_handles(&self) -> Vec<DistortHandles> {
        self.selection
            .iter()
            .filter_map(|id| {
                let s = self.doc().shape_on_page(*id)?;
                let d = distortion_of(&s.effects)?;
                let local = d
                    .center()
                    .unwrap_or_else(|| s.local_path().bounding_box().center());
                let center = s.transform * local;
                if !(center.x.is_finite() && center.y.is_finite()) {
                    return None;
                }
                let b = s.bounds();
                let reach = (b.width().max(b.height()) / 2.0).max(1.0);
                let mode = DistortMode::of(&d);
                let line = self.handle_line(*id, mode);
                let dir = line.dir();
                let (end, slider) = match d {
                    Distort::PushPull { amount, .. } => {
                        let a = if amount.is_finite() { amount } else { 0.0 };
                        (center + dir * (a / 100.0 * reach), None)
                    }
                    Distort::Zipper {
                        amplitude,
                        frequency,
                        tilt_deg,
                        ..
                    } => {
                        let a = if amplitude.is_finite() {
                            amplitude
                        } else {
                            0.0
                        };
                        let len = a * AMPLITUDE_REACH;
                        let t = frequency.min(MAX_FREQUENCY) as f64 / MAX_FREQUENCY as f64;
                        // The line leans with the teeth: the outline's
                        // normal at the nearest point, turned by the tilt.
                        let input = distortion_input(&s, &s.effects, false);
                        let dir = zipper_line_dir(&s, &input, local, tilt_deg);
                        (center + dir * len, Some(center + dir * (len * t)))
                    }
                    Distort::Twister { angle_deg, .. } => {
                        let a = if angle_deg.is_finite() {
                            angle_deg.to_radians()
                        } else {
                            0.0
                        };
                        // A free handle: as long as it was dragged (#1263),
                        // turned from the direction the drag began in.
                        let len = line
                            .len
                            .filter(|l| l.is_finite() && *l > 0.0)
                            .unwrap_or(reach);
                        let rest = if line.angle.is_finite() {
                            line.angle
                        } else {
                            0.0
                        };
                        (center + Vec2::from_angle(rest + a) * len, None)
                    }
                };
                Some(DistortHandles {
                    id: *id,
                    mode: DistortMode::of(&d),
                    center,
                    end,
                    slider,
                })
            })
            .collect()
    }

    /// How near (pixels) a press must be to a handle to take it: 6, or
    /// out to the edge of a handle drawn larger by Options > Nodes and
    /// Handles > Handle size (#1574).
    pub fn distort_pick_px(&self) -> f64 {
        HANDLE_PICK_PX.max(self.handle_size() as f64 / 2.0 + 2.0)
    }

    /// The handle line of `id`'s distortion of kind `mode`: where a drag
    /// left it, or the default (to the right, the Twister's handle half
    /// the object's larger side long).
    fn handle_line(&self, id: ShapeId, mode: DistortMode) -> HandleLine {
        self.distort_lines
            .get(&id)
            .copied()
            .filter(|l| l.mode == mode)
            .unwrap_or(HandleLine {
                mode,
                angle: 0.0,
                len: None,
            })
    }

    /// The handle line of `id` for `mode` goes along `angle` (radians);
    /// a Twister's keeps its length.
    fn set_handle_angle(&mut self, id: ShapeId, mode: DistortMode, angle: f64) {
        if !angle.is_finite() {
            return;
        }
        let len = self.handle_line(id, mode).len;
        self.distort_lines
            .insert(id, HandleLine { mode, angle, len });
    }

    /// The Push and Pull amount of a drag `d` on an object of half size
    /// `reach`, and the line's direction: the drag's length, 100 at
    /// `reach`, pushes right of the centre and pulls left of it, so the
    /// handle goes with the pointer in any direction (#1442).
    fn push_pull_of(d: Vec2, reach: f64) -> (f64, f64) {
        let amount = (100.0 * d.hypot() / reach.max(1e-9))
            .clamp(0.0, 200.0)
            .round();
        if d.x < 0.0 {
            (-amount, (-d).atan2())
        } else {
            (amount, d.atan2())
        }
    }

    /// Start a drag at `at` (page space): on a handle, take it; on an
    /// object, distort it about `at`. False when there is nothing there.
    pub fn begin_distort(&mut self, at: Point) -> bool {
        let tol = self.distort_pick_px() / self.view.zoom.max(1e-6) as f64;
        let near = |p: Point| (p - at).hypot() <= tol;
        let handles = self.distortion_handles();
        // The centre first, then the slider (it can sit on the line's
        // end), then the end.
        let grabbed = handles
            .iter()
            .find(|h| near(h.center))
            .map(|h| (h, Grab::Center))
            .or_else(|| {
                handles
                    .iter()
                    .find(|h| h.slider.is_some_and(near))
                    .map(|h| (h, Grab::Frequency))
            })
            .or_else(|| handles.iter().find(|h| near(h.end)).map(|h| (h, Grab::End)))
            .or_else(|| {
                // The line between them, within the pick distance.
                let tol = self.line_pick_tol();
                handles
                    .iter()
                    .find(|h| crate::effect_line::near_segment(h.center, h.end, at, tol))
                    .map(|h| (h, Grab::Line))
            });
        if let Some((h, grab)) = grabbed {
            // Each drag of a handle or the line is an undo step of its own.
            self.effect_edit_step = None;
            let Some(s) = self.doc().shape_on_page(h.id) else {
                return false;
            };
            let b = s.bounds();
            let reach = (b.width().max(b.height()) / 2.0).max(1.0);
            // The Twister's handle goes on turning from its angle.
            let (turned, last_angle) = match distortion_of(&s.effects) {
                Some(Distort::Twister { angle_deg, .. }) if angle_deg.is_finite() => {
                    (angle_deg.to_radians(), Some((h.end - h.center).atan2()))
                }
                _ => (0.0, None),
            };
            match distortion_of(&s.effects) {
                Some(d) => self.distort.take(&d),
                // A new distortion has Craft Corel's teeth (#1715).
                None => self.distort.shifted_tips = false,
            }
            self.distort_drag = Some(DistortDrag {
                id: h.id,
                start: h.center,
                center: h.center,
                press: at,
                end: h.end,
                reach,
                grab,
                turned,
                last_angle,
                base: None,
                begun: false,
            });
            return true;
        }
        // On an object, that object; anywhere else, the one selected: the
        // distortion's centre can be anywhere, off the object too (#1442).
        let id = match self.hit_test_inside(at) {
            Some(id) => id,
            None => match self.selection.as_slice() {
                [one] => *one,
                _ => return false,
            },
        };
        let Some(s) = self.doc().shape_on_page(id) else {
            return false;
        };
        if matches!(s.kind, ShapeKind::Bitmap { .. } | ShapeKind::Group { .. }) {
            return false;
        }
        if s.transform.determinant().abs() < 1e-12 {
            return false;
        }
        self.select(vec![id]);
        let b = s.bounds();
        let reach = (b.width().max(b.height()) / 2.0).max(1.0);
        self.distort_drag = Some(DistortDrag {
            id,
            start: at,
            center: s.transform.inverse() * at,
            press: at,
            end: at,
            reach,
            grab: Grab::Object,
            turned: 0.0,
            last_angle: None,
            base: Some(s.effects.clone()),
            begun: false,
        });
        true
    }

    /// The pointer's turn about `dd.start` added to `dd.turned`, the
    /// shortest way round from its last angle (so whole turns count).
    fn turn_toward(&self, dd: &mut DistortDrag, p: Point) {
        let d = p - dd.start;
        let tol = 2.0 / self.view.zoom.max(1e-6) as f64;
        if d.hypot() <= tol {
            return;
        }
        let a = d.atan2();
        if let Some(last) = dd.last_angle {
            let mut da = a - last;
            while da > std::f64::consts::PI {
                da -= std::f64::consts::TAU;
            }
            while da < -std::f64::consts::PI {
                da += std::f64::consts::TAU;
            }
            dd.turned += da;
        }
        dd.last_angle = Some(a);
    }

    /// The Twister's handle reaches the pointer: it is free, as long as
    /// the drag makes it, and stays so when let go (#1263); it turns
    /// from the direction the drag began in, so it sits under the
    /// pointer whichever way that was (#1442).
    fn stretch_twister_handle(&mut self, dd: &DistortDrag, p: Point) {
        let len = (p - dd.start).hypot();
        let tol = 2.0 / self.view.zoom.max(1e-6) as f64;
        if !(len.is_finite() && len > tol) {
            return;
        }
        // The pointer's angle less the turn so far: where the turn began.
        let rest = dd.last_angle.unwrap_or(0.0) - dd.turned;
        if !rest.is_finite() {
            return;
        }
        self.distort_lines.insert(
            dd.id,
            HandleLine {
                mode: DistortMode::Twister,
                angle: rest,
                len: Some(len),
            },
        );
    }

    /// The Twister angle (degrees) of a turn, within the bar's range.
    fn turn_degrees(turned: f64) -> f64 {
        let most = MAX_TURNS as f64 * 360.0 + 359.0;
        turned.to_degrees().clamp(-most, most).round()
    }

    /// The pointer moved to `p` during a drag.
    #[cfg(test)]
    pub fn drag_distort(&mut self, p: Point) {
        self.drag_distort_to(p, false, false);
    }

    /// The selected distortion whose handle line passes within the pick
    /// distance of `p`, away from its centre, slider and end.
    pub fn distortion_line_at(&self, p: Point) -> bool {
        let near = self.distort_pick_px() / self.view.zoom.max(1e-6) as f64;
        let tol = self.line_pick_tol();
        let handles = self.distortion_handles();
        let on_handle = handles.iter().any(|h| {
            (h.center - p).hypot() <= near
                || (h.end - p).hypot() <= near
                || h.slider.is_some_and(|q| (q - p).hypot() <= near)
        });
        !on_handle
            && handles
                .iter()
                .any(|h| crate::effect_line::near_segment(h.center, h.end, p, tol))
    }

    /// The pointer moved to `p` during a drag; a drag of the handle line
    /// keeps its move horizontal or vertical with `ctrl`. The end handle,
    /// and the line a drag on the object sets, follow the keys (#1563,
    /// #1793): `shift` moves it only sideways (it keeps its height),
    /// `ctrl` keeps its length and turns it freely all the way round,
    /// both keep it to steps of the constrain angle about the centre.
    pub fn drag_distort_to(&mut self, p: Point, ctrl: bool, shift: bool) {
        let Some(mut dd) = self.distort_drag.clone() else {
            return;
        };
        let p = if matches!(dd.grab, Grab::End | Grab::Object) {
            constrain_end(
                dd.start,
                dd.end,
                p,
                ctrl,
                shift,
                self.settings.constrain_angle,
            )
        } else {
            p
        };
        let Some(s) = self.doc().shape_on_page(dd.id) else {
            self.distort_drag = None;
            return;
        };
        if s.transform.determinant().abs() < 1e-12 {
            return;
        }
        if self.selection != vec![dd.id] {
            self.select(vec![dd.id]);
        }
        match dd.grab {
            Grab::Center | Grab::Line => {
                // The line: the centre goes by the pointer's way from the
                // press, the end with it.
                let p = if dd.grab == Grab::Line {
                    dd.start + crate::effect_line::constrain_move(p - dd.press, ctrl)
                } else {
                    p
                };
                let local = s.transform.inverse() * p;
                self.change_distortions("Distortion Center", |d| Some(d.with_center(Some(local))));
                return;
            }
            Grab::End | Grab::Frequency => {
                // The handles edit the object's distortion as the bar does:
                // its values with the pointer's, its centre kept.
                let Some(current) = distortion_of(&s.effects) else {
                    self.distort_drag = None;
                    return;
                };
                let mut sett = self.distort;
                sett.take(&current);
                let d = p - dd.start;
                // The Zipper's line stands on the outline the zipper is
                // applied to, about its centre (the object's own space).
                let zipper_input = || {
                    let center = current
                        .center()
                        .unwrap_or_else(|| s.local_path().bounding_box().center());
                    (distortion_input(&s, &s.effects, false), center)
                };
                match (sett.mode, dd.grab) {
                    (DistortMode::PushPull, _) => {
                        let (amount, angle) = Self::push_pull_of(d, dd.reach);
                        sett.amount = amount;
                        if d.hypot() > 1e-9 {
                            self.set_handle_angle(dd.id, DistortMode::PushPull, angle);
                        }
                    }
                    (DistortMode::Zipper, Grab::Frequency) => {
                        // Along the line, whichever way it goes.
                        let len = sett.amplitude * AMPLITUDE_REACH;
                        if len > 1e-9 {
                            let (input, center) = zipper_input();
                            let dir = zipper_line_dir(&s, &input, center, sett.tilt);
                            let t = (d.dot(dir) / len).clamp(0.0, 1.0);
                            sett.frequency = (t * MAX_FREQUENCY as f64).round() as u32;
                        }
                    }
                    (DistortMode::Zipper, _) => {
                        sett.amplitude = (d.hypot() / AMPLITUDE_REACH).clamp(0.0, 100.0);
                        // The teeth lean the way the handle is dragged
                        // from the outline's normal (#1671).
                        let (input, center) = zipper_input();
                        if let Some(tilt) = zipper_tilt_toward(&s, &input, center, d) {
                            sett.tilt = tilt;
                        }
                    }
                    (DistortMode::Twister, _) => {
                        self.turn_toward(&mut dd, p);
                        self.stretch_twister_handle(&dd, p);
                        sett.take(&Distort::Twister {
                            angle_deg: Self::turn_degrees(dd.turned),
                            center: None,
                        });
                    }
                }
                let sett = sett.clamped();
                self.distort = sett;
                self.distort_drag = Some(dd);
                self.change_distortions("Distortion", |d| Some(sett.effect(sett.mode, d.center())));
                return;
            }
            Grab::Object => {}
        }
        let d = p - dd.start;
        let mode = self.distort.mode;
        match mode {
            DistortMode::PushPull => {
                let (amount, angle) = Self::push_pull_of(d, dd.reach);
                self.distort.amount = amount;
                if d.hypot() > 1e-9 {
                    self.set_handle_angle(dd.id, mode, angle);
                }
            }
            DistortMode::Zipper => {
                self.distort.amplitude = (d.hypot() / AMPLITUDE_REACH).clamp(0.0, 100.0);
                // A new zipper goes on the outline as the distortions the
                // object has leave it; its teeth lean the way the drag
                // goes from that outline's normal at the centre (#1671).
                let base = dd.base.as_deref().unwrap_or(&s.effects);
                let input = distortion_input(&s, base, true);
                if let Some(tilt) = zipper_tilt_toward(&s, &input, dd.center, d) {
                    self.distort.tilt = tilt;
                }
            }
            DistortMode::Twister => {
                self.turn_toward(&mut dd, p);
                self.stretch_twister_handle(&dd, p);
                self.distort.take(&Distort::Twister {
                    angle_deg: Self::turn_degrees(dd.turned),
                    center: None,
                });
            }
        }
        let effect = self.distort.effect(mode, Some(dd.center));
        // A new distortion over the ones the object has (#1442): each drag
        // adds one, as the handles of the newest edit it.
        let mut effects = dd.base.clone().unwrap_or_else(|| s.effects.clone());
        effects.push(Effect::Distort(effect));
        let cmd = Command::SetEffects {
            shape: dd.id,
            effects,
        };
        let r = if dd.begun {
            self.engine.amend(&cmd)
        } else {
            self.engine.run_with_label(&cmd, "Distortion")
        };
        if let Err(e) = r {
            self.status = e.to_string();
        }
        dd.begun = true;
        self.distort_drag = Some(dd);
    }

    /// The Distort tool's clicks and drags.
    pub fn distort_input(&mut self, response: &egui::Response, p: Point) {
        use egui::PointerButton;
        if response.drag_started_by(PointerButton::Primary) {
            let origin = response
                .ctx
                .input(|i| i.pointer.press_origin())
                .map(|o| self.view.to_page(o))
                .unwrap_or(p);
            self.begin_distort(origin);
        }
        if response.dragged_by(PointerButton::Primary) && self.distort_drag.is_some() {
            let (ctrl, shift) = response
                .ctx
                .input(|i| (i.modifiers.ctrl || i.modifiers.command, i.modifiers.shift));
            self.drag_distort_to(p, ctrl, shift);
        }
        if response.drag_stopped() {
            self.distort_drag = None;
        }
        if response.clicked_by(PointerButton::Primary) {
            match self.hit_test(p) {
                Some(id) => self.select(vec![id]),
                None => self.select(Vec::new()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::Tool;
    use tracedraw_core::geometry::Rect;

    fn square(app: &mut App) -> ShapeId {
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            radius: 0.0,
            corners: None,
        })
        .expect("square")
    }

    fn effect(app: &App, id: ShapeId) -> Option<Distort> {
        app.doc()
            .find_shape(id)
            .and_then(|s| distortion_of(&s.effects))
    }

    fn steps(app: &App) -> usize {
        app.engine.history_labels().0.len()
    }

    #[test]
    fn a_drag_distorts_about_where_it_started() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        let before = steps(&app);
        // Push and Pull: half the width to the right is 100.
        assert!(app.begin_distort(Point::new(10.0, 10.0)));
        app.drag_distort(Point::new(15.0, 10.0));
        app.drag_distort(Point::new(20.0, 10.0));
        app.distort_drag = None;
        assert_eq!(
            effect(&app, id),
            Some(Distort::PushPull {
                amount: 100.0,
                center: Some(Point::new(10.0, 10.0)),
            })
        );
        assert_eq!(steps(&app), before + 1, "one undo step for the drag");
        // The bar shows it.
        assert_eq!(app.distort.amount, 100.0);
        // Twister: a quarter turn of the pointer about the start, then
        // a whole turn more.
        app.distort.mode = DistortMode::Twister;
        assert!(app.begin_distort(Point::new(5.0, 5.0)));
        let c = Point::new(5.0, 5.0);
        let mut a: f64 = 0.0;
        while a <= std::f64::consts::FRAC_PI_2 * 5.0 + 1e-9 {
            app.drag_distort(c + tracedraw_core::geometry::Vec2::new(a.cos(), a.sin()) * 8.0);
            a += 0.2;
        }
        app.distort_drag = None;
        let Some(Distort::Twister { angle_deg, center }) = effect(&app, id) else {
            panic!("a twister");
        };
        assert!((angle_deg - 450.0).abs() <= 12.0, "{angle_deg}");
        assert_eq!(center, Some(Point::new(5.0, 5.0)));
        assert_eq!((app.distort.turns, app.distort.clockwise), (1, false));
        // Nothing under the pointer and nothing selected: no drag.
        app.select(Vec::new());
        assert!(!app.begin_distort(Point::new(500.0, 500.0)));
    }

    /// #1442: a drag can start anywhere, off the selected object too (the
    /// centre where it starts), and a new drag on a distorted object adds
    /// a distortion over the one it has instead of replacing it. Clear
    /// Distortion takes the newest off first; Convert to Curves makes all
    /// of them the outline.
    #[test]
    fn drags_start_anywhere_and_add_distortions() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        app.select(vec![id]);
        // Off the object, to its left: centred there.
        assert!(app.begin_distort(Point::new(-30.0, 10.0)));
        app.drag_distort(Point::new(-25.0, 10.0));
        app.distort_drag = None;
        assert_eq!(
            effect(&app, id),
            Some(Distort::PushPull {
                amount: 50.0,
                center: Some(Point::new(-30.0, 10.0)),
            })
        );
        let count = |app: &App| {
            app.doc()
                .find_shape(id)
                .map(|s| {
                    s.effects
                        .iter()
                        .filter(|e| matches!(e, Effect::Distort(_)))
                        .count()
                })
                .unwrap_or(0)
        };
        assert_eq!(count(&app), 1);
        let once = app.doc().find_shape(id).expect("square").visual_bounds();
        // A second drag, on the object, away from the handles: one more
        // distortion, drawn over the first.
        let before = steps(&app);
        app.distort.mode = DistortMode::Zipper;
        assert!(app.begin_distort(Point::new(10.0, 18.0)));
        app.drag_distort(Point::new(10.0, 22.0));
        app.drag_distort(Point::new(10.0, 26.0));
        app.distort_drag = None;
        assert_eq!(steps(&app), before + 1, "one undo step for the drag");
        assert_eq!(count(&app), 2);
        let Some(Distort::Zipper { center, .. }) = effect(&app, id) else {
            panic!("the newest is the zipper: {:?}", effect(&app, id));
        };
        assert_eq!(center, Some(Point::new(10.0, 18.0)));
        // Its handles are the newest's.
        let h = app.distortion_handles();
        assert_eq!(h.len(), 1);
        assert_eq!(h[0].mode, DistortMode::Zipper);
        // Undo takes the second drag back whole.
        app.undo();
        assert_eq!(count(&app), 1);
        app.redo();
        assert_eq!(count(&app), 2);
        // Clear Distortion: the newest first.
        app.clear_distortion();
        assert_eq!(count(&app), 1);
        assert!(matches!(effect(&app, id), Some(Distort::PushPull { .. })));
        let b = app.doc().find_shape(id).expect("square").visual_bounds();
        assert!((b.width() - once.width()).abs() < 1e-6);
        app.undo();
        // Convert to Curves: both, in order, become the outline.
        let drawn = {
            let s = app.doc().find_shape(id).expect("square");
            tracedraw_core::live::evaluate(s).main.page_path()
        };
        app.bake_distortion();
        let s = app.doc().find_shape(id).expect("square");
        assert_eq!(count(&app), 0);
        let baked = s.page_path();
        let (a, b) = (
            tracedraw_core::geometry::Shape::bounding_box(&drawn),
            tracedraw_core::geometry::Shape::bounding_box(&baked),
        );
        assert!(
            (a.x0 - b.x0).abs() < 1e-6 && (a.y1 - b.y1).abs() < 1e-6,
            "{a:?} {b:?}"
        );
    }

    #[test]
    fn the_bar_edits_the_selected_distortion() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        app.select(vec![id]);
        // Apply adds the bar's distortion about the middle.
        app.distort.mode = DistortMode::Zipper;
        app.distort.frequency = 4;
        app.add_distortion();
        assert!(matches!(
            effect(&app, id),
            Some(Distort::Zipper {
                frequency: 4,
                center: None,
                ..
            })
        ));
        // Fields edit it; a mode button turns it into another kind.
        app.distort.smooth = true;
        app.apply_distort_settings();
        assert!(matches!(
            effect(&app, id),
            Some(Distort::Zipper { smooth: true, .. })
        ));
        app.distort.degrees = 45.0;
        app.distort.clockwise = true;
        app.set_distort_mode(DistortMode::Twister);
        assert_eq!(
            effect(&app, id),
            Some(Distort::Twister {
                angle_deg: -45.0,
                center: None
            })
        );
        // The centre marker sits in the middle and can be dragged.
        let handles = app.distortion_handles();
        assert_eq!(handles.len(), 1);
        assert!((handles[0].center - Point::new(10.0, 10.0)).hypot() < 1e-9);
        assert!(app.begin_distort(Point::new(10.0, 10.0)));
        app.drag_distort(Point::new(14.0, 12.0));
        app.distort_drag = None;
        assert_eq!(
            effect(&app, id).and_then(|d| d.center()),
            Some(Point::new(14.0, 12.0))
        );
        app.center_distortion();
        assert_eq!(effect(&app, id).and_then(|d| d.center()), None);
        // Convert to Curves bakes it; Clear takes it away.
        let shape = app.doc().find_shape(id).cloned().expect("shape");
        let baked = tracedraw_core::live::evaluate(&shape).main.local_path();
        app.bake_distortion();
        let after = app.doc().find_shape(id).cloned().expect("shape");
        assert!(effect(&app, id).is_none());
        assert_eq!(after.local_path(), baked);
        app.undo();
        app.clear_distortion();
        assert!(effect(&app, id).is_none());
    }

    /// A value changed on the bar shows on the selected object at once,
    /// also on one without a distortion yet (about its middle), and a
    /// run of edits is one undo step (#1030).
    #[test]
    fn a_bar_edit_previews_on_an_undistorted_selection() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        let picture = app
            .new_shape(ShapeKind::Bitmap {
                rect: Rect::new(30.0, 0.0, 40.0, 10.0),
                width_px: 1,
                height_px: 1,
                png: Vec::new().into(),
                fx: None,
            })
            .expect("bitmap");
        app.select(vec![id, picture]);
        let before = steps(&app);
        // Choosing the kind alone leaves the object as it is.
        app.set_distort_mode(DistortMode::Twister);
        assert_eq!(effect(&app, id), None);
        app.distort.degrees = 30.0;
        app.apply_distort_settings();
        app.distort.degrees = 45.0;
        app.apply_distort_settings();
        assert_eq!(
            effect(&app, id),
            Some(Distort::Twister {
                angle_deg: -45.0,
                center: None
            })
        );
        assert_eq!(effect(&app, picture), None, "bitmaps are not distorted");
        assert_eq!(steps(&app), before + 1, "one undo step for the edits");
        app.undo();
        assert_eq!(effect(&app, id), None);
    }

    /// The handles of a selected distortion (#1122): the end square sets
    /// the Push and Pull amount by its distance (to the left negative)
    /// and the Zipper amplitude; the slider on the Zipper's line sets the
    /// frequency; the Twister's circle sets the angle, whole turns
    /// counted as it goes round. Each drag is one undo step.
    #[test]
    fn the_handles_set_the_values_by_dragging() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        app.select(vec![id]);
        app.distort.mode = DistortMode::PushPull;
        app.distort.amount = 50.0;
        app.add_distortion();
        // Reach is 10 (half of 20): the square sits 5 mm right of the
        // centre; dragged to 10 mm left it is -100.
        let h = &app.distortion_handles()[0];
        assert_eq!(h.mode, DistortMode::PushPull);
        assert!((h.end - Point::new(15.0, 10.0)).hypot() < 1e-9, "{h:?}");
        assert!(h.slider.is_none());
        let before = steps(&app);
        assert!(app.begin_distort(Point::new(15.0, 10.0)));
        assert_eq!(app.distort_drag.as_ref().map(|d| d.grab), Some(Grab::End));
        app.drag_distort(Point::new(5.0, 10.0));
        app.drag_distort(Point::new(0.0, 10.0));
        app.distort_drag = None;
        assert_eq!(
            effect(&app, id),
            Some(Distort::PushPull {
                amount: -100.0,
                center: None
            })
        );
        assert_eq!(steps(&app), before + 1, "one undo step for the drag");
        // Zipper: the square four times the amplitude away along the
        // outline's normal at the point nearest the centre (the first
        // side of the square, going right along the bottom, its normal
        // up, #1671); the slider frequency / 100 of the way.
        app.distort.amplitude = 20.0;
        app.distort.frequency = 10;
        app.set_distort_mode(DistortMode::Zipper);
        let h = app.distortion_handles()[0].clone();
        assert!((h.end - Point::new(10.0, 90.0)).hypot() < 1e-9, "{h:?}");
        let slider = h.slider.expect("a slider");
        assert!((slider - Point::new(10.0, 18.0)).hypot() < 1e-9, "{h:?}");
        assert!(app.begin_distort(slider));
        assert_eq!(
            app.distort_drag.as_ref().map(|d| d.grab),
            Some(Grab::Frequency)
        );
        app.drag_distort(Point::new(10.5, 30.0));
        app.distort_drag = None;
        assert!(matches!(
            effect(&app, id),
            Some(Distort::Zipper { frequency: 25, .. })
        ));
        assert!(app.begin_distort(Point::new(10.0, 90.0)));
        assert_eq!(app.distort_drag.as_ref().map(|d| d.grab), Some(Grab::End));
        app.drag_distort(Point::new(10.0, 50.0));
        app.distort_drag = None;
        let Some(Distort::Zipper {
            amplitude,
            frequency,
            ..
        }) = effect(&app, id)
        else {
            panic!("a zipper");
        };
        assert!((amplitude - 10.0).abs() < 1e-9, "{amplitude}");
        assert_eq!(frequency, 25, "the slider's value stays");
        // Twister: the circle on the angle; dragged round one and a
        // quarter turns counterclockwise from 30 degrees it is 480.
        app.distort.turns = 0;
        app.distort.degrees = 30.0;
        app.distort.clockwise = false;
        app.set_distort_mode(DistortMode::Twister);
        let h = app.distortion_handles()[0].clone();
        let c = Point::new(10.0, 10.0);
        let want = c + tracedraw_core::geometry::Vec2::new(
            30f64.to_radians().cos(),
            30f64.to_radians().sin(),
        ) * 10.0;
        assert!((h.end - want).hypot() < 1e-9, "{h:?}");
        assert!(app.begin_distort(h.end));
        let mut a: f64 = 30f64.to_radians();
        let stop = (30.0f64 + 450.0).to_radians();
        while a <= stop + 1e-9 {
            app.drag_distort(c + tracedraw_core::geometry::Vec2::new(a.cos(), a.sin()) * 10.0);
            a += 0.2;
        }
        app.drag_distort(c + tracedraw_core::geometry::Vec2::new(stop.cos(), stop.sin()) * 10.0);
        app.distort_drag = None;
        let Some(Distort::Twister { angle_deg, .. }) = effect(&app, id) else {
            panic!("a twister");
        };
        assert!((angle_deg - 480.0).abs() <= 1.0, "{angle_deg}");
        assert_eq!((app.distort.turns, app.distort.clockwise), (1, false));
    }

    /// #1263: the Twister's handle is free: it reaches as far as it is
    /// dragged, stays there when let go and is taken there again; its
    /// length changes nothing but the handle.
    #[test]
    fn the_twister_handle_is_as_long_as_it_is_dragged() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        app.select(vec![id]);
        app.distort.mode = DistortMode::Twister;
        let c = Point::new(10.0, 10.0);
        let at = |len: f64, deg: f64| {
            c + tracedraw_core::geometry::Vec2::new(deg.to_radians().cos(), deg.to_radians().sin())
                * len
        };
        // A drag from the middle, out to 40 mm, turned a quarter.
        assert!(app.begin_distort(c));
        let mut deg = 0.0;
        while deg <= 90.0 {
            app.drag_distort(at(40.0, deg));
            deg += 10.0;
        }
        app.distort_drag = None;
        let h = app.distortion_handles()[0].clone();
        assert!((h.end - at(40.0, 90.0)).hypot() < 1e-6, "{h:?}");
        let Some(Distort::Twister { angle_deg, .. }) = effect(&app, id) else {
            panic!("a twister");
        };
        assert!((angle_deg - 90.0).abs() <= 1.0, "{angle_deg}");
        // Taken at its far end and drawn in to 5 mm: the angle stays.
        assert!(app.begin_distort(h.end));
        assert_eq!(app.distort_drag.as_ref().map(|d| d.grab), Some(Grab::End));
        app.drag_distort(at(20.0, 90.0));
        app.drag_distort(at(5.0, 90.0));
        app.distort_drag = None;
        let h = app.distortion_handles()[0].clone();
        assert!((h.end - at(5.0, 90.0)).hypot() < 1e-6, "{h:?}");
        let Some(Distort::Twister {
            angle_deg: again, ..
        }) = effect(&app, id)
        else {
            panic!("a twister");
        };
        assert!((again - angle_deg).abs() <= 1.0, "{again}");
    }

    /// #1442: the handle follows the pointer in any direction, not along
    /// an axis: it ends where the drag ends (within the rounding of the
    /// value), for a new distortion as for a handle taken and dragged,
    /// in every mode. Push and Pull takes the drag's length, pushed to
    /// the right of the centre and pulled to the left; the Zipper's
    /// slider runs along the line; a value changed on the bar keeps the
    /// line's direction.
    #[test]
    fn the_handles_follow_the_pointer_in_any_direction() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        app.select(vec![id]);
        let end = |app: &App| app.distortion_handles()[0].end;
        let near = |a: Point, b: Point, tol: f64| (a - b).hypot() <= tol;
        // Push and Pull: up and to the right, 10 mm (the reach): 100,
        // the handle under the pointer.
        let c = Point::new(10.0, 10.0);
        assert!(app.begin_distort(c));
        app.drag_distort(Point::new(13.0, 14.0));
        app.drag_distort(Point::new(16.0, 18.0));
        app.distort_drag = None;
        assert_eq!(
            effect(&app, id),
            Some(Distort::PushPull {
                amount: 100.0,
                center: Some(c),
            })
        );
        assert!(
            near(end(&app), Point::new(16.0, 18.0), 1e-6),
            "{:?}",
            end(&app)
        );
        // Taken and dragged down to the left: pulled, and still under
        // the pointer.
        assert!(app.begin_distort(Point::new(16.0, 18.0)));
        assert_eq!(app.distort_drag.as_ref().map(|d| d.grab), Some(Grab::End));
        app.drag_distort(Point::new(2.0, 16.0));
        app.distort_drag = None;
        assert!(matches!(
            effect(&app, id),
            Some(Distort::PushPull { amount, .. }) if amount == -100.0
        ));
        assert!(
            near(end(&app), Point::new(2.0, 16.0), 1e-6),
            "{:?}",
            end(&app)
        );
        // Then up to the right again, half way: pushed 50.
        assert!(app.begin_distort(Point::new(2.0, 16.0)));
        app.drag_distort(Point::new(13.0, 6.0));
        app.distort_drag = None;
        assert!(matches!(
            effect(&app, id),
            Some(Distort::PushPull { amount, .. }) if amount == 50.0
        ));
        assert!(
            near(end(&app), Point::new(13.0, 6.0), 1e-6),
            "{:?}",
            end(&app)
        );
        // The bar's value moves the handle along the same line.
        app.distort.amount = -100.0;
        app.apply_distort_settings();
        assert!(
            near(end(&app), Point::new(4.0, 18.0), 1e-6),
            "{:?}",
            end(&app)
        );
        // Zipper: a new drag (away from the handles) down to the left;
        // the slider on the line.
        app.distort.mode = DistortMode::Zipper;
        app.distort.frequency = 50;
        let c = Point::new(10.0, 2.0);
        let far = Point::new(-6.0, -14.0);
        assert!(app.begin_distort(c));
        assert_eq!(
            app.distort_drag.as_ref().map(|d| d.grab),
            Some(Grab::Object)
        );
        app.drag_distort(far);
        app.distort_drag = None;
        let Some(Distort::Zipper { amplitude, .. }) = effect(&app, id) else {
            panic!("a zipper: {:?}", effect(&app, id));
        };
        assert!(
            (amplitude - 512f64.sqrt() / 4.0).abs() < 1e-9,
            "{amplitude}"
        );
        let h = app.distortion_handles()[0].clone();
        assert!(near(h.end, far, 1e-6), "{h:?}");
        let along = |t: f64| c + (far - c) * t;
        let slider = h.slider.expect("a slider");
        assert!(near(slider, along(0.5), 1e-6), "{h:?}");
        assert!(app.begin_distort(slider));
        assert_eq!(
            app.distort_drag.as_ref().map(|d| d.grab),
            Some(Grab::Frequency)
        );
        app.drag_distort(along(0.9));
        app.distort_drag = None;
        assert!(matches!(
            effect(&app, id),
            Some(Distort::Zipper { frequency: 90, .. })
        ));
        // Twister: a new drag that starts upward and turns to the left,
        // 15 mm out: the circle under the pointer all the way.
        app.distort.mode = DistortMode::Twister;
        let c = Point::new(10.0, 10.0);
        assert!(app.begin_distort(c));
        let mut deg: f64 = 90.0;
        while deg <= 180.0 {
            let p = c + Vec2::from_angle(deg.to_radians()) * 15.0;
            app.drag_distort(p);
            assert!(near(end(&app), p, 0.2), "{deg}: {:?} {p:?}", end(&app));
            deg += 15.0;
        }
        app.distort_drag = None;
        let Some(Distort::Twister { angle_deg, .. }) = effect(&app, id) else {
            panic!("a twister");
        };
        assert!((angle_deg - 90.0).abs() <= 1.0, "{angle_deg}");
        // Taken at its end and dragged on round, still under the pointer.
        let p = c + Vec2::from_angle(180f64.to_radians()) * 15.0;
        assert!(app.begin_distort(p));
        assert_eq!(app.distort_drag.as_ref().map(|d| d.grab), Some(Grab::End));
        let p = c + Vec2::from_angle(250f64.to_radians()) * 8.0;
        app.drag_distort(p);
        app.distort_drag = None;
        assert!(near(end(&app), p, 0.2), "{:?} {p:?}", end(&app));
        let Some(Distort::Twister { angle_deg, .. }) = effect(&app, id) else {
            panic!("a twister");
        };
        assert!((angle_deg - 160.0).abs() <= 1.0, "{angle_deg}");
    }

    /// #1671: the Zipper's teeth lean the way the handle is dragged from
    /// the outline's normal, and the handle line leans with them. A drag
    /// from the middle of a square's bottom side straight down (along
    /// the normal) makes upright teeth with the handle where the drag
    /// ended; the handle taken and dragged round 30 degrees leans the
    /// teeth 30 degrees, the handle under the pointer; a tilt typed on
    /// the bar turns the handle back.
    #[test]
    fn a_zipper_drag_leans_the_teeth() {
        let mut app = App::headless();
        app.tool = Tool::Distort;
        let id = square(&mut app);
        app.select(vec![id]);
        app.distort.mode = DistortMode::Zipper;
        app.distort.frequency = 3;
        let c = Point::new(10.0, 0.0);
        let down = Point::new(10.0, -8.0);
        assert!(app.begin_distort(c));
        app.drag_distort(Point::new(10.0, -4.0));
        app.drag_distort(down);
        app.distort_drag = None;
        let Some(Distort::Zipper {
            amplitude,
            tilt_deg,
            center,
            ..
        }) = effect(&app, id)
        else {
            panic!("a zipper: {:?}", effect(&app, id));
        };
        assert!((amplitude - 2.0).abs() < 1e-9, "{amplitude}");
        assert_eq!(center, Some(c));
        assert!(
            tilt_deg.abs() < 1e-9 || (tilt_deg.abs() - 180.0).abs() < 1e-9,
            "upright: {tilt_deg}"
        );
        assert_eq!(app.distort.lean(), 0.0);
        let end = |app: &App| app.distortion_handles()[0].end;
        assert!((end(&app) - down).hypot() < 1e-6, "{:?}", end(&app));
        // The tips on the bottom side stand 2 mm off it.
        let tips = |app: &App| -> Vec<Point> {
            let s = app.doc().find_shape(id).expect("square");
            tracedraw_core::live::evaluate(s)
                .main
                .page_path()
                .elements()
                .iter()
                .filter_map(|e| match e {
                    tracedraw_core::geometry::PathEl::MoveTo(p)
                    | tracedraw_core::geometry::PathEl::LineTo(p)
                    | tracedraw_core::geometry::PathEl::CurveTo(_, _, p) => Some(*p),
                    _ => None,
                })
                .filter(|p| p.y.abs() < 2.5 && p.x > 2.5 && p.x < 17.5)
                .collect()
        };
        let low = tips(&app);
        assert!(!low.is_empty());
        for p in &low {
            assert!((p.y.abs() - 2.0).abs() < 1e-6, "upright tip {p:?}");
        }
        // Taken and dragged round, 30 degrees off the normal: the teeth
        // lean 30 degrees and the handle is under the pointer. Only the
        // tips lean, still 2 mm off the side; the bases stay (#1736).
        assert!(app.begin_distort(down));
        assert_eq!(app.distort_drag.as_ref().map(|d| d.grab), Some(Grab::End));
        let round = c + Vec2::from_angle((-90.0f64 - 30.0).to_radians()) * 8.0;
        app.drag_distort(round);
        app.distort_drag = None;
        let Some(Distort::Zipper { tilt_deg, .. }) = effect(&app, id) else {
            panic!("a zipper");
        };
        assert!(
            (app.distort.lean().abs() - 30.0).abs() < 1e-6,
            "{tilt_deg} shown as {}",
            app.distort.lean()
        );
        assert!((end(&app) - round).hypot() < 1e-6, "{:?}", end(&app));
        let leaning = tips(&app);
        for p in &leaning {
            assert!((p.y.abs() - 2.0).abs() < 1e-6, "leaning tip {p:?}");
        }
        let kept = low
            .iter()
            .filter(|p| leaning.iter().any(|q| (*q - **p).hypot() < 1e-6))
            .count();
        assert!(kept > 0, "bases stay: {low:?} {leaning:?}");
        // The bar: tilt 0 puts the handle back on the normal, the teeth
        // upright.
        app.distort.set_lean(0.0);
        app.apply_distort_settings();
        assert!((end(&app) - down).hypot() < 1e-6, "{:?}", end(&app));
        for p in tips(&app) {
            assert!((p.y.abs() - 2.0).abs() < 1e-6, "upright tip {p:?}");
        }
        // Folding: a tilt of 150 shows as -30 and keeps its side.
        let mut s = DistortSettings {
            tilt: 150.0,
            ..Default::default()
        };
        assert_eq!(s.lean(), -30.0);
        s.set_lean(20.0);
        assert_eq!(s.tilt, -160.0);
        assert_eq!(s.lean(), 20.0);
    }

    #[test]
    fn settings_round_trip_and_stay_in_range() {
        let mut s = DistortSettings::default();
        s.take(&Distort::Twister {
            angle_deg: -800.0,
            center: None,
        });
        assert_eq!((s.turns, s.degrees, s.clockwise), (2, 80.0, true));
        assert_eq!(s.angle(), -800.0);
        let odd = DistortSettings {
            amount: f64::NAN,
            amplitude: -5.0,
            frequency: 10_000,
            turns: 99,
            degrees: 720.0,
            ..Default::default()
        }
        .clamped();
        assert_eq!(odd.amount, 20.0);
        assert_eq!(odd.amplitude, 0.0);
        assert_eq!(odd.frequency, MAX_FREQUENCY);
        assert_eq!((odd.turns, odd.degrees), (MAX_TURNS, 359.0));
        // Replacing in place keeps the order of the other effects.
        let effects = vec![
            Effect::Distort(Distort::PushPull {
                amount: 1.0,
                center: None,
            }),
            Effect::Envelope {
                nodes: vec![],
                keep_lines: false,
                handles: vec![],
                corners: vec![],
                mapping: Default::default(),
            },
        ];
        let next = with_distortion(
            &effects,
            Some(Distort::PushPull {
                amount: 2.0,
                center: None,
            }),
        );
        assert!(matches!(
            next[0],
            Effect::Distort(Distort::PushPull { amount, .. }) if amount == 2.0
        ));
        assert_eq!(with_distortion(&effects, None).len(), 1);
    }
}
