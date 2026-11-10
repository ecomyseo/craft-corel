//! TraceDraw: a graphics tablet's pen on a native Wayland window (#1841,
//! `vendor/README.md`). winit 0.30 binds no tablet protocol, and a
//! compositor sends a pen's input over nothing else, so the pen did not
//! reach a Wayland window at all. This file binds the public tablet-v2
//! protocol (`zwp_tablet_manager_v2`, one `zwp_tablet_seat_v2` for each
//! seat) and hands each tool's events to `crate::tablet_tool`, which
//! decides what the window is told; this file talks to the compositor.

use std::sync::Mutex;

use sctk::compositor::SurfaceData;
use sctk::globals::GlobalData;
use sctk::reexports::client::globals::GlobalList;
use sctk::reexports::client::protocol::wl_seat::WlSeat;
use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::{
    event_created_child, Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use sctk::reexports::protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::{
    Shape, WpCursorShapeDeviceV1,
};
use sctk::reexports::protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_manager_v2::ZwpTabletManagerV2;
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_pad_group_v2::{
    self, ZwpTabletPadGroupV2,
};
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_pad_ring_v2::{
    self, ZwpTabletPadRingV2,
};
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_pad_strip_v2::{
    self, ZwpTabletPadStripV2,
};
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_pad_v2::{
    self, ZwpTabletPadV2,
};
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_seat_v2::{
    self, ZwpTabletSeatV2,
};
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_tool_v2::{
    self, ButtonState, ZwpTabletToolV2,
};
use sctk::reexports::protocols::wp::tablet::zv2::client::zwp_tablet_v2::{self, ZwpTabletV2};

use crate::dpi::{LogicalPosition, PhysicalPosition};
use crate::event::{ElementState, Force, MouseButton, Touch, TouchPhase, WindowEvent};
use crate::platform_impl::wayland::state::WinitState;
use crate::platform_impl::wayland::window::state::TabletCursor;
use crate::platform_impl::wayland::{self, DeviceId};
use crate::tablet_tool::{PenButton, PenOut, ToolEvent, ToolState};

/// Touch ids of pens: the tool's protocol id above this bit, so that
/// they never meet the ids of a touch screen's fingers.
const PEN_TOUCH_ID: u64 = 1 << 40;

/// The tablet manager, and the cursor shape manager that gives the pen
/// a cursor over the window.
#[derive(Debug)]
pub struct TabletState {
    manager: ZwpTabletManagerV2,
    cursor_shape: Option<WpCursorShapeManagerV1>,
}

impl TabletState {
    /// Bind the tablet manager; `None` when the compositor has none.
    pub fn bind(globals: &GlobalList, queue_handle: &QueueHandle<WinitState>) -> Option<Self> {
        let manager = match globals.bind(queue_handle, 1..=1, GlobalData) {
            Ok(manager) => manager,
            Err(e) => {
                tracing::warn!("Tablet protocol not available, no pen input: {e:?}");
                return None;
            }
        };
        let cursor_shape = globals.bind(queue_handle, 1..=1, GlobalData).ok();
        Some(Self {
            manager,
            cursor_shape,
        })
    }

    /// The tablet seat of `seat`, which announces its tablets and tools.
    pub fn tablet_seat(
        &self,
        seat: &WlSeat,
        queue_handle: &QueueHandle<WinitState>,
    ) -> ZwpTabletSeatV2 {
        self.manager.get_tablet_seat(seat, queue_handle, ())
    }
}

/// One tool's state.
#[derive(Debug, Default)]
pub struct TabletToolData {
    tool: Mutex<ToolState<WlSurface>>,
    cursor: Mutex<Option<WpCursorShapeDeviceV1>>,
    /// The surface of the pen's custom cursor images, made when it first
    /// comes over a window (#1841).
    cursor_surface: Mutex<Option<WlSurface>>,
}

impl WinitState {
    /// The pen `tool` left proximity: no window shows its cursor on it
    /// any longer (#1841).
    fn tablet_tool_left(&mut self, tool: &ZwpTabletToolV2) {
        for window in self.windows.get_mut().values() {
            window.lock().unwrap().tablet_tool_left(tool);
        }
    }

    /// Tell the window under each of `out` what the pen did.
    /// `altitude` is the pen's (radians), when it reports its tilt: it
    /// goes out with the pressure, which stays the same share (#1841).
    fn send_pen(&mut self, touch_id: u64, altitude: Option<f64>, out: Vec<PenOut<WlSurface>>) {
        let device_id = crate::event::DeviceId(crate::platform_impl::DeviceId::Wayland(DeviceId));
        for o in out {
            let surface = match &o {
                PenOut::Entered(s)
                | PenOut::Moved(s, _)
                | PenOut::Pressure(s, _, _)
                | PenOut::Button(s, _, _)
                | PenOut::Left(s) => s,
            };
            // Over the window's frame (a surface of its own) the pen
            // does nothing.
            let Some(parent) = surface.data::<SurfaceData>().map(|d| d.parent_surface()) else {
                continue;
            };
            if parent.is_some() {
                continue;
            }
            let window_id = wayland::make_wid(surface);
            let scale_factor = match self.windows.get_mut().get(&window_id) {
                Some(window) => window.lock().unwrap().scale_factor(),
                None => continue,
            };
            let physical = |(x, y): (f64, f64)| -> PhysicalPosition<f64> {
                LogicalPosition::new(x, y).to_physical(scale_factor)
            };
            let event = match o {
                PenOut::Entered(_) => WindowEvent::CursorEntered { device_id },
                PenOut::Moved(_, at) => WindowEvent::CursorMoved {
                    device_id,
                    position: physical(at),
                },
                // As the X11 and macOS copies send a pen's pressure: a
                // touch that only moves (vendor/README.md).
                PenOut::Pressure(_, at, share) => WindowEvent::Touch(Touch {
                    device_id,
                    phase: TouchPhase::Moved,
                    location: physical(at),
                    force: share.map(|force| match altitude {
                        // egui-winit reads `force / max_possible_force`.
                        Some(altitude) => Force::Calibrated {
                            force,
                            max_possible_force: 1.0,
                            altitude_angle: Some(altitude),
                        },
                        None => Force::Normalized(force),
                    }),
                    id: touch_id,
                }),
                PenOut::Button(_, button, pressed) => WindowEvent::MouseInput {
                    device_id,
                    state: if pressed {
                        ElementState::Pressed
                    } else {
                        ElementState::Released
                    },
                    button: match button {
                        PenButton::Primary => MouseButton::Left,
                        PenButton::Secondary => MouseButton::Right,
                        PenButton::Middle => MouseButton::Middle,
                        PenButton::Back => MouseButton::Back,
                        PenButton::Other(code) => MouseButton::Other(code),
                    },
                },
                PenOut::Left(_) => WindowEvent::CursorLeft { device_id },
            };
            self.events_sink.push_window_event(event, window_id);
            self.dispatched_events = true;
        }
    }
}

impl Dispatch<ZwpTabletManagerV2, GlobalData> for WinitState {
    fn event(
        _: &mut WinitState,
        _: &ZwpTabletManagerV2,
        _: <ZwpTabletManagerV2 as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
        // No events.
    }
}

impl Dispatch<ZwpTabletSeatV2, ()> for WinitState {
    fn event(
        _: &mut WinitState,
        _: &ZwpTabletSeatV2,
        _: zwp_tablet_seat_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
        // The tablets, tools and pads it announces are made with their
        // data below and handled by their own dispatch.
    }

    event_created_child!(WinitState, ZwpTabletSeatV2, [
        zwp_tablet_seat_v2::EVT_TABLET_ADDED_OPCODE => (ZwpTabletV2, ()),
        zwp_tablet_seat_v2::EVT_TOOL_ADDED_OPCODE => (ZwpTabletToolV2, TabletToolData::default()),
        zwp_tablet_seat_v2::EVT_PAD_ADDED_OPCODE => (ZwpTabletPadV2, ()),
    ]);
}

impl Dispatch<ZwpTabletV2, ()> for WinitState {
    fn event(
        _: &mut WinitState,
        tablet: &ZwpTabletV2,
        event: zwp_tablet_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
        // Its name and ids are not needed: the tools carry the input.
        if let zwp_tablet_v2::Event::Removed = event {
            tablet.destroy();
        }
    }
}

impl Dispatch<ZwpTabletToolV2, TabletToolData> for WinitState {
    fn event(
        state: &mut WinitState,
        tool: &ZwpTabletToolV2,
        event: zwp_tablet_tool_v2::Event,
        data: &TabletToolData,
        _: &Connection,
        queue_handle: &QueueHandle<WinitState>,
    ) {
        use zwp_tablet_tool_v2::Event;
        let event = match event {
            Event::ProximityIn {
                serial, surface, ..
            } => {
                // The cursor over the window: without one the compositor
                // may show none for the pen.
                let shape = state
                    .tablet
                    .as_ref()
                    .and_then(|t| t.cursor_shape.as_ref())
                    .map(|manager| {
                        data.cursor
                            .lock()
                            .unwrap()
                            .get_or_insert_with(|| {
                                manager.get_tablet_tool_v2(tool, queue_handle, GlobalData)
                            })
                            .clone()
                    });
                // The window's own cursor, as its pointers show it: the
                // tool's, or none where TraceDraw draws its own (#1841).
                // Over the window's frame, the default arrow.
                let window_id = wayland::make_wid(&surface);
                let window = state.windows.get_mut().get(&window_id).cloned();
                match window {
                    Some(window) => {
                        let cursor_surface = data
                            .cursor_surface
                            .lock()
                            .unwrap()
                            .get_or_insert_with(|| {
                                state.compositor_state.create_surface(queue_handle)
                            })
                            .clone();
                        window.lock().unwrap().tablet_tool_entered(TabletCursor {
                            tool: tool.clone(),
                            serial,
                            shape,
                            surface: cursor_surface,
                        });
                    }
                    None => {
                        if let Some(device) = shape {
                            device.set_shape(serial, Shape::Default);
                        }
                    }
                }
                ToolEvent::ProximityIn(surface)
            }
            Event::ProximityOut => {
                state.tablet_tool_left(tool);
                ToolEvent::ProximityOut
            }
            Event::Down { .. } => ToolEvent::Down,
            Event::Up => ToolEvent::Up,
            Event::Motion { x, y } => ToolEvent::Motion(x, y),
            Event::Pressure { pressure } => ToolEvent::Pressure(pressure),
            Event::Tilt { tilt_x, tilt_y } => ToolEvent::Tilt(tilt_x, tilt_y),
            Event::Button {
                button,
                state: WEnum::Value(button_state),
                ..
            } => ToolEvent::Button {
                code: button,
                pressed: button_state == ButtonState::Pressed,
            },
            Event::Frame { .. } => ToolEvent::Frame,
            Event::Removed => {
                let mut out = Vec::new();
                data.tool.lock().unwrap().leave(&mut out);
                state.send_pen(PEN_TOUCH_ID | u64::from(tool.id().protocol_id()), None, out);
                state.tablet_tool_left(tool);
                if let Some(cursor) = data.cursor.lock().unwrap().take() {
                    cursor.destroy();
                }
                if let Some(surface) = data.cursor_surface.lock().unwrap().take() {
                    surface.destroy();
                }
                tool.destroy();
                return;
            }
            // The tool's type and ids, its distance, rotation, slider and
            // wheel: not read (`crate::tablet_tool`).
            _ => return,
        };
        let mut out = Vec::new();
        let altitude = {
            let mut tool_state = data.tool.lock().unwrap();
            tool_state.handle(event, &mut out);
            tool_state.altitude()
        };
        if !out.is_empty() {
            state.send_pen(
                PEN_TOUCH_ID | u64::from(tool.id().protocol_id()),
                altitude,
                out,
            );
        }
    }
}

// A tablet's pad (the buttons, rings and strips on the tablet itself) is
// not read; its objects are taken and let go when removed.

impl Dispatch<ZwpTabletPadV2, ()> for WinitState {
    fn event(
        _: &mut WinitState,
        pad: &ZwpTabletPadV2,
        event: zwp_tablet_pad_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
        if let zwp_tablet_pad_v2::Event::Removed = event {
            pad.destroy();
        }
    }

    event_created_child!(WinitState, ZwpTabletPadV2, [
        zwp_tablet_pad_v2::EVT_GROUP_OPCODE => (ZwpTabletPadGroupV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadGroupV2, ()> for WinitState {
    fn event(
        _: &mut WinitState,
        _: &ZwpTabletPadGroupV2,
        _: zwp_tablet_pad_group_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
    }

    event_created_child!(WinitState, ZwpTabletPadGroupV2, [
        zwp_tablet_pad_group_v2::EVT_RING_OPCODE => (ZwpTabletPadRingV2, ()),
        zwp_tablet_pad_group_v2::EVT_STRIP_OPCODE => (ZwpTabletPadStripV2, ()),
    ]);
}

impl Dispatch<ZwpTabletPadRingV2, ()> for WinitState {
    fn event(
        _: &mut WinitState,
        _: &ZwpTabletPadRingV2,
        _: zwp_tablet_pad_ring_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
    }
}

impl Dispatch<ZwpTabletPadStripV2, ()> for WinitState {
    fn event(
        _: &mut WinitState,
        _: &ZwpTabletPadStripV2,
        _: zwp_tablet_pad_strip_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<WinitState>,
    ) {
    }
}
