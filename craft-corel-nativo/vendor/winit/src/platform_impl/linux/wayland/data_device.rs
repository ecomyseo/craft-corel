//! TraceDraw: files dragged over a window from a file manager (#1142,
//! `vendor/README.md`). winit 0.30 binds no `wl_data_device`, so a
//! native Wayland window received no file drops at all. The Wayland
//! drag's events go to `crate::file_drop`, which decides what the window
//! is told; this file talks to the compositor.

use std::io::Read;

use sctk::compositor::SurfaceData;
use sctk::data_device_manager::data_device::{DataDeviceData, DataDeviceHandler};
use sctk::data_device_manager::data_offer::{DataOfferHandler, DragOffer};
use sctk::data_device_manager::data_source::DataSourceHandler;
use sctk::data_device_manager::ReadPipe;
use sctk::reexports::calloop::PostAction;
use sctk::reexports::client::protocol::wl_data_device::WlDataDevice;
use sctk::reexports::client::protocol::wl_data_device_manager::DndAction;
use sctk::reexports::client::protocol::wl_data_source::WlDataSource;
use sctk::reexports::client::protocol::wl_surface::WlSurface;
use sctk::reexports::client::{Connection, Proxy, QueueHandle};

use crate::dpi::{LogicalPosition, PhysicalPosition};
use crate::event::WindowEvent;
use crate::file_drop::{AfterDrop, Out, URI_LIST};
use crate::platform_impl::wayland::state::WinitState;
use crate::platform_impl::wayland::{self, DeviceId, WindowId};

impl WinitState {
    /// The window a drag is over and the place in it (`None` over the
    /// window's frame, a surface of its own).
    fn drag_target(
        &self,
        surface: &WlSurface,
        x: f64,
        y: f64,
    ) -> Option<(WindowId, Option<(f64, f64)>)> {
        let parent = surface
            .data::<SurfaceData>()?
            .parent_surface()
            .unwrap_or(surface);
        let window_id = wayland::make_wid(parent);
        let windows = self.windows.borrow();
        let window = windows.get(&window_id)?;
        if parent != surface {
            return Some((window_id, None));
        }
        let scale_factor = window.lock().unwrap().scale_factor();
        let p: PhysicalPosition<f64> = LogicalPosition::new(x, y).to_physical(scale_factor);
        Some((window_id, Some((p.x, p.y))))
    }

    /// Tell the windows.
    fn send_file_drop(&mut self, out: Vec<Out<WindowId>>) {
        let device_id = crate::event::DeviceId(crate::platform_impl::DeviceId::Wayland(DeviceId));
        for o in out {
            let (event, window_id) = match o {
                // Over the frame: a place outside the area the program
                // draws in, so that a drop there is not taken for one on
                // its contents.
                Out::Moved(w, at) => {
                    let (x, y) = at.unwrap_or((-1.0, -1.0));
                    (
                        WindowEvent::CursorMoved {
                            device_id,
                            position: PhysicalPosition::new(x, y),
                        },
                        w,
                    )
                }
                Out::Hovered(w, p) => (WindowEvent::HoveredFile(p), w),
                Out::Dropped(w, p) => (WindowEvent::DroppedFile(p), w),
                Out::Cancelled(w) => (WindowEvent::HoveredFileCancelled, w),
            };
            if self.windows.get_mut().contains_key(&window_id) {
                self.events_sink.push_window_event(event, window_id);
                self.dispatched_events = true;
            }
        }
    }

    /// Read the list of files of `offer` for `ticket`, without blocking:
    /// the event loop calls back as the source writes.
    fn read_file_list(&mut self, offer: &DragOffer, ticket: u64) {
        let pipe: ReadPipe = match offer.receive(URI_LIST.to_owned()) {
            Ok(pipe) => pipe,
            Err(error) => {
                tracing::warn!("could not read the files dragged over the window: {error}");
                self.file_list_failed(ticket);
                return;
            }
        };
        let mut list = Vec::new();
        let inserted = self
            .loop_handle
            .insert_source(pipe, move |(), file, state| {
                let mut buf = [0u8; 4096];
                // One read for each wake-up: the pipe blocks, and it is ready.
                match (&**file).read(&mut buf) {
                    Ok(0) => {
                        let mut out = Vec::new();
                        let finish = state.file_drag.data(ticket, &list, &mut out);
                        state.send_file_drop(out);
                        if finish {
                            state.finish_file_drop();
                        }
                        PostAction::Remove
                    }
                    Ok(n) => {
                        list.extend_from_slice(&buf[..n]);
                        PostAction::Continue
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {
                        PostAction::Continue
                    }
                    Err(error) => {
                        tracing::warn!("could not read the files dragged over the window: {error}");
                        state.file_list_failed(ticket);
                        PostAction::Remove
                    }
                }
            });
        if inserted.is_err() {
            self.file_list_failed(ticket);
        }
    }

    fn file_list_failed(&mut self, ticket: u64) {
        let mut out = Vec::new();
        let give_up = self.file_drag.read_failed(ticket, &mut out);
        self.send_file_drop(out);
        if give_up {
            self.forget_file_drop_offer();
        }
    }

    /// The files of a drop went out: tell the source the copy is done.
    fn finish_file_drop(&mut self) {
        if let Some(offer) = self.file_drop_offer.take() {
            if self
                .file_drop_action
                .intersects(DndAction::Copy | DndAction::Move)
            {
                offer.finish();
            }
            offer.destroy();
        }
    }

    fn forget_file_drop_offer(&mut self) {
        if let Some(offer) = self.file_drop_offer.take() {
            offer.destroy();
        }
    }
}

impl DataDeviceHandler for WinitState {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        data_device: &WlDataDevice,
        x: f64,
        y: f64,
        surface: &WlSurface,
    ) {
        let Some(offer) = data_device
            .data::<DataDeviceData>()
            .and_then(|d| d.drag_offer())
        else {
            return;
        };
        // An earlier drop still being read is given up for this drag.
        if self.file_drag.active() {
            self.forget_file_drop_offer();
        }
        self.file_drop_action = DndAction::empty();
        let target = self.drag_target(surface, x, y);
        let files = target.is_some() && offer.with_mime_types(|m| m.iter().any(|t| t == URI_LIST));
        if files {
            offer.set_actions(DndAction::Copy, DndAction::Copy);
            offer.accept_mime_type(offer.serial, Some(URI_LIST.to_owned()));
        } else {
            offer.accept_mime_type(offer.serial, None);
        }
        let Some((window_id, at)) = target else {
            let mut out = Vec::new();
            self.file_drag.leave(&mut out);
            self.send_file_drop(out);
            return;
        };
        let mut out = Vec::new();
        let ticket = self.file_drag.enter(window_id, at, files, &mut out);
        self.send_file_drop(out);
        if let Some(ticket) = ticket {
            self.file_drop_offer = Some(offer.clone());
            self.read_file_list(&offer, ticket);
        }
    }

    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {
        let mut out = Vec::new();
        self.file_drag.leave(&mut out);
        self.send_file_drop(out);
        // The toolkit destroys an offer that was not dropped.
        if !self.file_drag.active() {
            self.file_drop_offer = None;
        }
    }

    fn motion(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        data_device: &WlDataDevice,
        x: f64,
        y: f64,
    ) {
        let Some(offer) = data_device
            .data::<DataDeviceData>()
            .and_then(|d| d.drag_offer())
        else {
            return;
        };
        if let Some((window_id, at)) = self.drag_target(&offer.surface, x, y) {
            let mut out = Vec::new();
            self.file_drag.motion(window_id, at, &mut out);
            self.send_file_drop(out);
        }
    }

    fn selection(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataDevice) {}

    fn drop_performed(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        data_device: &WlDataDevice,
    ) {
        let mut out = Vec::new();
        let next = self.file_drag.drop(&mut out);
        self.send_file_drop(out);
        match next {
            AfterDrop::Finish => self.finish_file_drop(),
            AfterDrop::Wait => {}
            AfterDrop::Read(ticket) => {
                let offer = data_device
                    .data::<DataDeviceData>()
                    .and_then(|d| d.drag_offer());
                match offer.or_else(|| self.file_drop_offer.clone()) {
                    Some(offer) => self.read_file_list(&offer, ticket),
                    None => self.file_list_failed(ticket),
                }
            }
            AfterDrop::Ignore => {
                if let Some(offer) = data_device
                    .data::<DataDeviceData>()
                    .and_then(|d| d.drag_offer())
                {
                    offer.destroy();
                }
            }
        }
    }
}

impl DataOfferHandler for WinitState {
    fn source_actions(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        offer: &mut DragOffer,
        actions: DndAction,
    ) {
        if self.file_drag.active() && actions.contains(DndAction::Copy) {
            offer.set_actions(DndAction::Copy, DndAction::Copy);
        }
    }

    fn selected_action(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &mut DragOffer,
        actions: DndAction,
    ) {
        self.file_drop_action = actions;
    }
}

// Only for the dispatch of the data device: winit offers no data of its own.
impl DataSourceHandler for WinitState {
    fn accept_mime(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: Option<String>,
    ) {
    }

    fn send_request(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlDataSource,
        _: String,
        _: sctk::data_device_manager::WritePipe,
    ) {
    }

    fn cancelled(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn dnd_dropped(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn dnd_finished(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource) {}

    fn action(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &WlDataSource, _: DndAction) {}
}

sctk::delegate_data_device!(WinitState);
