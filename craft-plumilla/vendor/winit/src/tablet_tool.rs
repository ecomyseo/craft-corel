//! TraceDraw: a graphics tablet's pen on a native Wayland window (#1841).
//!
//! winit 0.30 binds no tablet protocol (`zwp_tablet_manager_v2`, the
//! public tablet-v2 protocol of wayland-protocols), and a Wayland
//! compositor sends a pen's input only over that protocol: GNOME's sends
//! a client that does not bind it nothing at all, so the pen neither
//! moved the pointer nor drew. The Wayland backend
//! (`platform_impl/linux/wayland/tablet.rs`) now binds the protocol and
//! hands each tool's events to this module, which turns them into what
//! the window is told, the way the other backends report a pen:
//!
//! - the pen coming into proximity of a window enters it, and leaving
//!   proximity leaves it, as a mouse pointer does;
//! - its motion moves the pointer;
//! - the tip touching the tablet presses the primary button, lifting it
//!   releases it; the barrel buttons press the middle (`BTN_STYLUS`) and
//!   secondary (`BTN_STYLUS2`) buttons, and a third one (`BTN_STYLUS3`)
//!   the back button;
//! - while the tip touches, its pressure (0 to 65535 in the protocol)
//!   goes out as a share of its range, as the X11 and macOS copies send
//!   it (a touch that only moves, with that force; the window layer
//!   reads it in `pen_pressure`); lifting the tip, or a reading of 0,
//!   sends no force, so a lifted pen leaves no stale pressure.
//!
//! The protocol groups a tool's events into frames: what a frame
//! carries is sent when its `frame` event comes, in the order place,
//! pressure, presses, releases. Leaving proximity sends what is still
//! pending first, then releases every button the pen still holds (the
//! compositor sends the tip's `up` before, but a button left pressed
//! would stick in the program), then leaves.
//!
//! The pen's tilt is read (`ToolEvent::Tilt`) and goes out with the
//! pressure as the pen's altitude (`altitude_of`), in
//! `Force::Calibrated { force, max_possible_force: 1, altitude_angle }`:
//! egui-winit takes `force / max_possible_force`, so the pressure is the
//! same share as before (#1841). Its distance, rotation, slider and wheel
//! are not read: winit 0.30's events have no place for them and no tool
//! uses them.
//!
//! The pen over a window shows the window's cursor (#1841): the Wayland
//! backend sets it on the tool as on the pointers, a named cursor through
//! the cursor shape protocol (`cursor_shape_value`), a custom image on a
//! surface of the tool's own, none while the window hides its cursor
//! (TraceDraw draws its tool cursors itself). Before, the compositor
//! always showed its arrow for the pen.
//!
//! This file depends on nothing in the crate, and TraceDraw's tests
//! include and run it (`winit_tablet_tool::tests`).

/// The largest pressure the protocol reports.
pub const PRESSURE_MAX: u32 = 65535;

/// Linux input event codes of a pen's barrel buttons, as the protocol's
/// `button` event names them.
pub const BTN_STYLUS: u32 = 0x14b;
pub const BTN_STYLUS2: u32 = 0x14c;
pub const BTN_STYLUS3: u32 = 0x149;

/// A button of the pen, as the window is told of it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PenButton {
    /// The tip touching the tablet.
    Primary,
    /// `BTN_STYLUS2`.
    Secondary,
    /// `BTN_STYLUS`.
    Middle,
    /// `BTN_STYLUS3`.
    Back,
    /// Any other code.
    Other(u16),
}

impl PenButton {
    /// The button a barrel button's event code stands for.
    pub fn from_code(code: u32) -> Self {
        match code {
            BTN_STYLUS => Self::Middle,
            BTN_STYLUS2 => Self::Secondary,
            BTN_STYLUS3 => Self::Back,
            other => Self::Other(other as u16),
        }
    }
}

/// One event of a tablet tool, as the protocol sends it. `S` is the
/// surface the tool comes into proximity of.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolEvent<S> {
    ProximityIn(S),
    ProximityOut,
    Down,
    Up,
    /// Surface-local place, in logical pixels.
    Motion(f64, f64),
    /// 0 to `PRESSURE_MAX`.
    Pressure(u32),
    /// The pen's tilt from the perpendicular, degrees, along x and y.
    Tilt(f64, f64),
    /// A barrel button's event code and whether it is now pressed.
    Button {
        code: u32,
        pressed: bool,
    },
    Frame,
}

/// What the window over `S` is told.
#[derive(Debug, Clone, PartialEq)]
pub enum PenOut<S> {
    /// The pen came over the surface.
    Entered(S),
    /// The pointer is at this surface-local place.
    Moved(S, (f64, f64)),
    /// The pen's pressure at that place, a share of 0..1, or `None` when
    /// it does not press.
    Pressure(S, (f64, f64), Option<f64>),
    /// A button pressed (`true`) or released.
    Button(S, PenButton, bool),
    /// The pen left the surface.
    Left(S),
}

/// The events of one frame that have not been sent yet.
#[derive(Debug, Default)]
struct Pending {
    motion: Option<(f64, f64)>,
    pressure: Option<u32>,
    down: bool,
    up: bool,
    buttons: Vec<(PenButton, bool)>,
}

/// One tablet tool (a pen, its eraser end...), from proximity to
/// proximity.
#[derive(Debug)]
pub struct ToolState<S> {
    surface: Option<S>,
    position: (f64, f64),
    pressure: u32,
    tip_down: bool,
    held: Vec<PenButton>,
    pending: Pending,
    /// The last tilt read, degrees; `None` for a tool that reports none.
    tilt: Option<(f64, f64)>,
}

impl<S> Default for ToolState<S> {
    fn default() -> Self {
        Self {
            surface: None,
            position: (0.0, 0.0),
            pressure: 0,
            tip_down: false,
            held: Vec::new(),
            pending: Pending::default(),
            tilt: None,
        }
    }
}

/// The pen's altitude above the tablet, radians (pi/2 upright), from its
/// tilt along x and y in degrees from the perpendicular.
pub fn altitude_of(tilt_x: f64, tilt_y: f64) -> f64 {
    let (tx, ty) = (tilt_x.to_radians().tan(), tilt_y.to_radians().tan());
    let lean = (tx * tx + ty * ty).sqrt();
    if lean.is_finite() {
        (1.0 / lean).atan()
    } else {
        0.0
    }
}

/// The value of the cursor shape protocol's `shape` enum for a cursor
/// of CSS name `name` (`CursorIcon::name`, "col-resize"); the default
/// arrow (1) for a name it has not. The values are the protocol's
/// (`wp_cursor_shape_device_v1`, version 1).
pub fn cursor_shape_value(name: &str) -> u32 {
    const NAMES: [&str; 34] = [
        "default",
        "context_menu",
        "help",
        "pointer",
        "progress",
        "wait",
        "cell",
        "crosshair",
        "text",
        "vertical_text",
        "alias",
        "copy",
        "move",
        "no_drop",
        "not_allowed",
        "grab",
        "grabbing",
        "e_resize",
        "n_resize",
        "ne_resize",
        "nw_resize",
        "s_resize",
        "se_resize",
        "sw_resize",
        "w_resize",
        "ew_resize",
        "ns_resize",
        "nesw_resize",
        "nwse_resize",
        "col_resize",
        "row_resize",
        "all_scroll",
        "zoom_in",
        "zoom_out",
    ];
    let name = name.replace('-', "_");
    NAMES
        .iter()
        .position(|n| *n == name)
        .map_or(1, |i| i as u32 + 1)
}

/// A pressure reading as a share of the protocol's range; `None` at 0.
pub fn pressure_share(pressure: u32) -> Option<f64> {
    (pressure > 0).then(|| (f64::from(pressure) / f64::from(PRESSURE_MAX)).min(1.0))
}

impl<S: Clone + PartialEq> ToolState<S> {
    /// The surface the tool is over, if any.
    #[cfg(test)]
    pub fn surface(&self) -> Option<&S> {
        self.surface.as_ref()
    }

    /// The pen's altitude (radians) while its tilt is known.
    pub fn altitude(&self) -> Option<f64> {
        self.tilt.map(|(x, y)| altitude_of(x, y))
    }

    /// Take one event of the tool; what the window is to be told is
    /// pushed onto `out`.
    pub fn handle(&mut self, event: ToolEvent<S>, out: &mut Vec<PenOut<S>>) {
        match event {
            ToolEvent::ProximityIn(surface) => {
                if self.surface.as_ref() != Some(&surface) {
                    self.leave(out);
                }
                if self.surface.is_none() {
                    out.push(PenOut::Entered(surface.clone()));
                    self.surface = Some(surface);
                }
            }
            ToolEvent::ProximityOut => self.leave(out),
            ToolEvent::Down => {
                self.pending.down = true;
                self.pending.up = false;
            }
            ToolEvent::Up => self.pending.up = true,
            ToolEvent::Motion(x, y) => self.pending.motion = Some((x, y)),
            ToolEvent::Pressure(p) => self.pending.pressure = Some(p.min(PRESSURE_MAX)),
            ToolEvent::Tilt(x, y) => {
                self.tilt = (x.is_finite() && y.is_finite()).then_some((x, y));
            }
            ToolEvent::Button { code, pressed } => self
                .pending
                .buttons
                .push((PenButton::from_code(code), pressed)),
            ToolEvent::Frame => self.flush(out),
        }
    }

    /// Send what the frame carried.
    fn flush(&mut self, out: &mut Vec<PenOut<S>>) {
        let pending = std::mem::take(&mut self.pending);
        let Some(surface) = self.surface.clone() else {
            return;
        };
        if let Some(at) = pending.motion {
            self.position = at;
            out.push(PenOut::Moved(surface.clone(), at));
        }
        let pressure_changed = pending.pressure.is_some_and(|p| p != self.pressure);
        if let Some(p) = pending.pressure {
            self.pressure = p;
        }
        // A frame that lifts the tip ends with no pressure (below), so
        // its last reading is not sent first.
        let pressing = (self.tip_down || pending.down) && !pending.up;
        if pressing && (pending.down || pending.motion.is_some() || pressure_changed) {
            out.push(PenOut::Pressure(
                surface.clone(),
                self.position,
                pressure_share(self.pressure),
            ));
        }
        if pending.down && !self.tip_down {
            self.tip_down = true;
            self.press(&surface, PenButton::Primary, out);
        }
        for (button, pressed) in pending.buttons {
            if pressed {
                self.press(&surface, button, out);
            } else {
                self.release(&surface, button, out);
            }
        }
        if pending.up && self.tip_down {
            self.tip_down = false;
            self.release(&surface, PenButton::Primary, out);
            out.push(PenOut::Pressure(surface, self.position, None));
        }
    }

    fn press(&mut self, surface: &S, button: PenButton, out: &mut Vec<PenOut<S>>) {
        if !self.held.contains(&button) {
            self.held.push(button);
            out.push(PenOut::Button(surface.clone(), button, true));
        }
    }

    fn release(&mut self, surface: &S, button: PenButton, out: &mut Vec<PenOut<S>>) {
        if let Some(i) = self.held.iter().position(|&b| b == button) {
            self.held.remove(i);
            out.push(PenOut::Button(surface.clone(), button, false));
        }
    }

    /// Leave the surface the tool is over: what is pending first, then
    /// every button still held, then the leave itself. Also what a tool
    /// that is removed does.
    pub fn leave(&mut self, out: &mut Vec<PenOut<S>>) {
        self.flush(out);
        let Some(surface) = self.surface.take() else {
            return;
        };
        let was_down = self.tip_down;
        self.tip_down = false;
        for button in std::mem::take(&mut self.held) {
            out.push(PenOut::Button(surface.clone(), button, false));
        }
        if was_down {
            out.push(PenOut::Pressure(surface.clone(), self.position, None));
        }
        self.pressure = 0;
        self.tilt = None;
        out.push(PenOut::Left(surface));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The pen's cursor shapes by name: the protocol's values, the CSS
    /// names with hyphens as `CursorIcon::name` gives them, the arrow for
    /// a name the protocol lacks.
    #[test]
    fn cursor_names_become_the_protocols_shapes() {
        assert_eq!(cursor_shape_value("default"), 1);
        assert_eq!(cursor_shape_value("context-menu"), 2);
        assert_eq!(cursor_shape_value("crosshair"), 8);
        assert_eq!(cursor_shape_value("text"), 9);
        assert_eq!(cursor_shape_value("move"), 13);
        assert_eq!(cursor_shape_value("grabbing"), 17);
        assert_eq!(cursor_shape_value("nwse-resize"), 29);
        assert_eq!(cursor_shape_value("zoom-out"), 34);
        assert_eq!(cursor_shape_value("no-such-cursor"), 1);
    }

    /// The tilt becomes an altitude (upright pi/2, lying flat 0), is
    /// kept while the pen is near and forgotten when it leaves; the
    /// pressure the window is told does not change with it.
    #[test]
    fn the_tilt_is_read_as_an_altitude() {
        use std::f64::consts::FRAC_PI_2;
        assert!((altitude_of(0.0, 0.0) - FRAC_PI_2).abs() < 1e-9);
        assert!((altitude_of(30.0, 0.0) - 60f64.to_radians()).abs() < 1e-9);
        assert!((altitude_of(0.0, -45.0) - 45f64.to_radians()).abs() < 1e-9);
        assert!(altitude_of(90.0, 0.0).abs() < 1e-6);
        let mut tool = ToolState::<u8>::default();
        let mut out = Vec::new();
        tool.handle(ToolEvent::ProximityIn(1), &mut out);
        assert_eq!(tool.altitude(), None);
        tool.handle(ToolEvent::Tilt(30.0, 0.0), &mut out);
        tool.handle(ToolEvent::Down, &mut out);
        tool.handle(ToolEvent::Pressure(PRESSURE_MAX / 2), &mut out);
        tool.handle(ToolEvent::Frame, &mut out);
        assert!((tool.altitude().unwrap() - 60f64.to_radians()).abs() < 1e-9);
        let share = out.iter().find_map(|o| match o {
            PenOut::Pressure(_, _, Some(p)) => Some(*p),
            _ => None,
        });
        assert!((share.unwrap() - 0.5).abs() < 1e-3, "{out:?}");
        tool.handle(ToolEvent::ProximityOut, &mut out);
        assert_eq!(tool.altitude(), None);
    }

    fn run(state: &mut ToolState<u32>, events: Vec<ToolEvent<u32>>) -> Vec<PenOut<u32>> {
        let mut out = Vec::new();
        for e in events {
            state.handle(e, &mut out);
        }
        out
    }

    #[test]
    fn a_stroke_enters_moves_presses_with_pressure_and_releases() {
        use PenOut::*;
        use ToolEvent as E;
        let mut tool = ToolState::default();
        // Hover in: enter and move, no pressure, no press.
        let out = run(
            &mut tool,
            vec![E::ProximityIn(7), E::Motion(10.0, 20.0), E::Frame],
        );
        assert_eq!(out, vec![Entered(7), Moved(7, (10.0, 20.0))]);
        assert_eq!(tool.surface(), Some(&7));
        // Hovering with a reading of 0 sends nothing more.
        let out = run(&mut tool, vec![E::Pressure(0), E::Frame]);
        assert!(out.is_empty());
        // The tip touches: place, pressure, then the press.
        let out = run(
            &mut tool,
            vec![E::Down, E::Motion(11.0, 20.0), E::Pressure(32768), E::Frame],
        );
        assert_eq!(out.len(), 3);
        assert_eq!(out[0], Moved(7, (11.0, 20.0)));
        match &out[1] {
            Pressure(7, (x, y), Some(p)) => {
                assert_eq!((*x, *y), (11.0, 20.0));
                assert!((p - 32768.0 / 65535.0).abs() < 1e-12);
            }
            other => panic!("expected a pressure, got {other:?}"),
        }
        assert_eq!(out[2], Button(7, PenButton::Primary, true));
        // A drag: each move carries the pressure.
        let out = run(
            &mut tool,
            vec![E::Motion(15.0, 22.0), E::Pressure(65535), E::Frame],
        );
        assert_eq!(
            out,
            vec![Moved(7, (15.0, 22.0)), Pressure(7, (15.0, 22.0), Some(1.0))]
        );
        // A move with the pressure unchanged still carries it.
        let out = run(&mut tool, vec![E::Motion(16.0, 22.0), E::Frame]);
        assert_eq!(
            out,
            vec![Moved(7, (16.0, 22.0)), Pressure(7, (16.0, 22.0), Some(1.0))]
        );
        // A frame of pressure alone.
        let out = run(&mut tool, vec![E::Pressure(16384), E::Frame]);
        assert!(matches!(
            out.as_slice(),
            [Pressure(7, (16.0, 22.0), Some(_))]
        ));
        // The tip lifts: the release, then no pressure.
        let out = run(&mut tool, vec![E::Up, E::Pressure(0), E::Frame]);
        assert_eq!(
            out,
            vec![
                Button(7, PenButton::Primary, false),
                Pressure(7, (16.0, 22.0), None)
            ]
        );
        // Hovering away.
        let out = run(&mut tool, vec![E::ProximityOut, E::Frame]);
        assert_eq!(out, vec![Left(7)]);
        assert_eq!(tool.surface(), None);
    }

    #[test]
    fn nothing_is_sent_before_the_frame() {
        let mut tool = ToolState::default();
        let out = run(
            &mut tool,
            vec![
                ToolEvent::ProximityIn(1),
                ToolEvent::Motion(1.0, 2.0),
                ToolEvent::Down,
            ],
        );
        assert_eq!(out, vec![PenOut::Entered(1)]);
        let out = run(&mut tool, vec![ToolEvent::Frame]);
        assert_eq!(out.len(), 3);
        // A down at a reading of 0 (no pressure yet) presses without force.
        assert_eq!(out[1], PenOut::Pressure(1, (1.0, 2.0), None));
        assert_eq!(out[2], PenOut::Button(1, PenButton::Primary, true));
    }

    #[test]
    fn a_tap_in_one_frame_presses_and_releases() {
        use ToolEvent as E;
        let mut tool = ToolState::default();
        run(
            &mut tool,
            vec![E::ProximityIn(2), E::Motion(4.0, 4.0), E::Frame],
        );
        let out = run(
            &mut tool,
            vec![E::Down, E::Pressure(30000), E::Up, E::Pressure(0), E::Frame],
        );
        assert_eq!(
            out,
            vec![
                PenOut::Button(2, PenButton::Primary, true),
                PenOut::Button(2, PenButton::Primary, false),
                PenOut::Pressure(2, (4.0, 4.0), None),
            ]
        );
    }

    #[test]
    fn barrel_buttons_press_middle_secondary_and_back() {
        assert_eq!(PenButton::from_code(BTN_STYLUS), PenButton::Middle);
        assert_eq!(PenButton::from_code(BTN_STYLUS2), PenButton::Secondary);
        assert_eq!(PenButton::from_code(BTN_STYLUS3), PenButton::Back);
        assert_eq!(PenButton::from_code(0x110), PenButton::Other(0x110));
        let mut tool = ToolState::default();
        let out = run(
            &mut tool,
            vec![
                ToolEvent::ProximityIn(3),
                ToolEvent::Frame,
                ToolEvent::Button {
                    code: BTN_STYLUS2,
                    pressed: true,
                },
                ToolEvent::Frame,
                // A press repeated is sent once.
                ToolEvent::Button {
                    code: BTN_STYLUS2,
                    pressed: true,
                },
                ToolEvent::Frame,
                ToolEvent::Button {
                    code: BTN_STYLUS2,
                    pressed: false,
                },
                ToolEvent::Frame,
                // A release of a button never pressed is not sent.
                ToolEvent::Button {
                    code: BTN_STYLUS,
                    pressed: false,
                },
                ToolEvent::Frame,
            ],
        );
        assert_eq!(
            out,
            vec![
                PenOut::Entered(3),
                PenOut::Button(3, PenButton::Secondary, true),
                PenOut::Button(3, PenButton::Secondary, false),
            ]
        );
    }

    #[test]
    fn leaving_proximity_releases_what_is_still_held() {
        use ToolEvent as E;
        let mut tool = ToolState::default();
        run(
            &mut tool,
            vec![
                E::ProximityIn(5),
                E::Motion(3.0, 4.0),
                E::Down,
                E::Pressure(1000),
                E::Button {
                    code: BTN_STYLUS,
                    pressed: true,
                },
                E::Frame,
            ],
        );
        // Out without an up first (and without a frame between).
        let out = run(
            &mut tool,
            vec![E::Motion(9.0, 9.0), E::ProximityOut, E::Frame],
        );
        assert_eq!(out.len(), 6, "{out:?}");
        assert_eq!(out[0], PenOut::Moved(5, (9.0, 9.0)));
        assert!(matches!(out[1], PenOut::Pressure(5, (9.0, 9.0), Some(_))));
        assert_eq!(out[2], PenOut::Button(5, PenButton::Primary, false));
        assert_eq!(out[3], PenOut::Button(5, PenButton::Middle, false));
        assert_eq!(out[4], PenOut::Pressure(5, (9.0, 9.0), None));
        assert_eq!(out[5], PenOut::Left(5));
        // The next stroke starts with no pressure left over.
        let out = run(&mut tool, vec![E::ProximityIn(5), E::Down, E::Frame]);
        assert_eq!(
            out,
            vec![
                PenOut::Entered(5),
                PenOut::Pressure(5, (9.0, 9.0), None),
                PenOut::Button(5, PenButton::Primary, true),
            ]
        );
    }

    #[test]
    fn moving_to_another_surface_leaves_the_first() {
        let mut tool = ToolState::default();
        let out = run(
            &mut tool,
            vec![
                ToolEvent::ProximityIn(1),
                ToolEvent::Frame,
                ToolEvent::ProximityIn(2),
                ToolEvent::Motion(0.5, 0.5),
                ToolEvent::Frame,
                // The same surface again is no new entry.
                ToolEvent::ProximityIn(2),
                ToolEvent::Frame,
            ],
        );
        assert_eq!(
            out,
            vec![
                PenOut::Entered(1),
                PenOut::Left(1),
                PenOut::Entered(2),
                PenOut::Moved(2, (0.5, 0.5)),
            ]
        );
    }

    #[test]
    fn events_out_of_proximity_send_nothing() {
        let mut tool = ToolState::default();
        let out = run(
            &mut tool,
            vec![
                ToolEvent::Motion(1.0, 1.0),
                ToolEvent::Down,
                ToolEvent::Pressure(500),
                ToolEvent::Up,
                ToolEvent::Frame,
                ToolEvent::ProximityOut,
                ToolEvent::Frame,
            ],
        );
        assert!(out.is_empty(), "{out:?}");
    }

    #[test]
    fn pressure_is_a_share_of_the_protocol_range() {
        assert_eq!(pressure_share(0), None);
        assert_eq!(pressure_share(PRESSURE_MAX), Some(1.0));
        assert_eq!(pressure_share(u32::MAX), Some(1.0));
        let low = pressure_share(655).expect("a share");
        assert!((low - 0.01).abs() < 1e-3);
    }
}
