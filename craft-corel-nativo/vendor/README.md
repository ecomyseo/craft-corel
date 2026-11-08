# Patched dependencies

Copies of published crates with a small change TraceDraw needs before the
upstream release that carries it can be used. They are not workspace
members (`exclude` in the root `Cargo.toml`) and are wired in with
`[patch.crates-io]`. Each is the published crate as downloaded from
crates.io, licence and authors unchanged (see `ATTRIBUTION.md`), with the
change marked `TraceDraw:` in a comment.

## eframe 0.36.2

- Change: `native/glow_integration.rs` creates the OpenGL context with
  three attempts in turn, glutin's default (OpenGL 3.3 core), OpenGL ES,
  then a compatibility profile of any version (`context_attributes_to_try`,
  `create_first_context` and their tests), backported unchanged from
  eframe 0.37.0. 0.36.2 stopped after OpenGL ES, which WGL cannot make on
  older Intel drivers, so a card with OpenGL 2.1 only (Intel HD Graphics
  Ironlake, #616) could not open the window with `--renderer=gl`.
- `create_first_context` sits in `native/first_context.rs` (0.37 has it in
  `glow_integration.rs`) with its tests, so that TraceDraw's tests include
  and run it (`gpu::eframe_context_fallback::tests`); the copy's own tests
  run in no gate. The licence texts, which the published crate leaves
  out, are egui's `LICENSE-MIT` and `LICENSE-APACHE` at tag 0.36.2.
- Change: a painter for the fixed-function OpenGL pipeline
  (`native/fixed_function_painter.rs`, its arithmetic and tests in
  `native/fixed_function_geometry.rs`, the choice in
  `native/gl_painter.rs`), TraceDraw's own code. The OpenGL renderer draws
  with it instead of `egui_glow`'s painter when the context is desktop
  OpenGL below 3.0, or when `TRACEDRAW_GL_PAINTER=fixed` asks for it
  (`shaders` asks for `egui_glow`'s): client-side vertex arrays and
  `GL_MODULATE` texturing, no shaders, buffer objects or vertex array
  objects. The OpenGL 2.1 driver of the Intel HD Graphics of 2010
  (8.15.10.2900, #616) compiles `egui_glow`'s GLSL 1.20 shaders and then
  draws everything after the first mesh of a frame as black boxes and
  stray triangles; the fixed-function pipeline is what such drivers were
  made for, and it computes what egui's shader does (vertex colour times
  texel, premultiplied alpha). `TRACEDRAW_GL_CONTEXT=compat` asks for the
  compatibility profile before OpenGL 3.3 core and OpenGL ES, for tests
  under Mesa forced to OpenGL 2.1 (which would answer the ES attempt);
  `fixed` alone, on a core profile, draws nothing (every fixed-function
  call is invalid there, the log shows `OpenGL error` lines), which is
  how TraceDraw's read-back check of the renderer is exercised.
  TraceDraw's tests include and run the arithmetic
  (`gpu::eframe_fixed_function_geometry::tests`).
- Change: a window made maximized is not given its size and position a
  second time once it exists (`native/window_start.rs`, used by
  `glow_integration.rs` and by `epi_integration::create_window`, which
  `wgpu_integration.rs` calls in place of `egui_winit::create_window`).
  egui-winit applies them again after creating a window, as it then knows
  the monitor's scaling; the window already has that size as the size it
  restores to, and on Windows a size or position asked for after the
  maximize clears it, so the hidden window was restored, shown at its
  normal size for a moment and maximized again: TraceDraw started without
  the splash screen opened at its normal size and then grew to the
  screen (#787). TraceDraw's tests include and run it
  (`splash::eframe_window_start::tests`). Check whether egui 0.37 still
  needs it before dropping the copy.
- Change: in the browser, `dragover` and `drop` (`web/events.rs`,
  `install_drag_and_drop`) push a pointer move at the event's place
  before the hovered or dropped files (#1142): the browser sends no mouse
  moves while files are dragged over the page, so TraceDraw could not
  tell a drop on the drawing (import) from one on the menus or tabs
  (open in a tab of its own), and the hint always said "Drop to open".
- Change: in the browser, a pen's pointer events send its pressure
  (#1410). The published web layer reads a force from touch events only
  and drops the `pressure` of pointer events, so a graphics tablet's pen
  drew Freehand and Brush Strokes at an even width. `web/events.rs`
  now listens for `pointermove` and `pointercancel` on the document and
  reads the pen's pressure at `pointerdown` and `pointerup`
  (`push_pen_pressure`), and sends it as the patched winit copy does on
  X11 and macOS: an `egui::Event::Touch` of phase `Move` only, of a
  device of its own (`PEN_DEVICE`), with the reading as `force`, which
  TraceDraw's raw input hook reads and takes out (`pen.rs`,
  `docs/behavior/pen-pressure.md`). The pen's position comes from its
  own `pointerdown` and `pointermove` too (`pen_moved`, #1655): Safari
  on iPad sends an Apple Pencil no mouse events at all, so the Pencil
  did nothing; its press and release come from `pointerdown` and
  `pointerup` as for every pointer, a pen's `pointerdown` on the canvas
  is not left to the browser (`preventDefault`: no text selection or
  handwriting), a `pointercancel` lets the tip go (`pen_cancelled`), and
  the touches Safari also sends for the Pencil (`touchType` `stylus`)
  are left out of the touch events (`input::is_stylus`), or the press
  would come twice and egui would take the Pencil for a finger; a move
  of the Pencil's touches alone only keeps the page from scrolling.
  Only `pointerType` `pen`
  gives pressure: the mouse (whose 0.5 while pressed means no pressure)
  and touches (whose force still comes from the touch events) send
  nothing; a hovering or lifted pen, or a reading of 0, sends no force;
  0.5 counts as no pressure until a pen has given another reading
  (`WebInput::pen`), since a pen that cannot measure pressure reports
  0.5 while pressed. The arithmetic is in `web/pen_pressure.rs`, which
  uses nothing of the crate but egui; TraceDraw's tests include and run
  it (`eframe_web_pen_pressure::tests`).
- Change: the application can keep a paste shortcut from egui-winit's
  clipboard text read (#1488). egui-winit reads the clipboard as text on
  every Ctrl+V, before the application sees the key. On X11, an owner
  that holds a picture alone (`xclip -selection clipboard -t image/png`
  with a picture of about 1 MB) answers that text request with its
  picture in parts (INCR); the clipboard library stops after the first
  part, and the owner then waits for the rest of that transfer forever
  and answers no other request, so TraceDraw's own picture read timed
  out, the picture never pasted and the window froze for about 24 s.
  `native/paste_gate.rs` holds `set_paste_text_gate` (exported from the
  crate root) and `blocks`, which `EpiIntegration::on_window_event`
  calls before handing an event to egui-winit: a real paste key press
  (`is_paste_key`, egui-winit's `is_paste_command` with its logical or
  physical key) is not handed over when the gate answers `false`, so
  egui-winit neither reads the clipboard nor sends a paste event, as when
  the clipboard holds no text; the application sees the key's release
  and pastes by itself. `blocks` also follows the modifier keys from
  `ModifiersChanged`. With no gate set, nothing changes. TraceDraw's
  gate (`clipboard::paste_may_read_text`) reads the X11 clipboard's
  types and lets the text be read only when a text type is offered.
  TraceDraw's tests include and run the key test
  (`clipboard::eframe_paste_gate::tests`). When moving to a newer egui,
  check whether egui-winit still reads the text this way.
- Change: a window started `centered` is centred where the primary
  monitor lies on the desktop, and the primary monitor's rectangle is
  kept for the application (#1800). `epi_integration::viewport_builder`
  centred the window by the monitor's size from the desktop's top-left
  corner, and egui's `ViewportInfo` gives the application a monitor's
  size but not its position, so on a desktop whose primary monitor does
  not hold that corner (X11, the primary at (1544, 2160) below two
  others) the splash screen and the main window opened on the top-left
  monitor. `native/monitor_place.rs` holds `centred_on` (the
  arithmetic), `remember_primary` and `primary_monitor` (exported from
  the crate root: `[x, y, width, height]` in system points); the
  application centres its main window with them
  (`splash::open_main_window`). TraceDraw's tests include and run it
  (`splash::eframe_monitor_place::tests`). When moving to a newer
  eframe, check whether it centres with the monitor's position.
- Drop the copy and the `[patch.crates-io]` entry when the workspace moves
  to egui/eframe 0.37 or later: what holds that back is the egui 0.37
  API migration across the app. The fixed-function painter has no
  upstream counterpart: carry it over, or drop it once no supported card
  needs it.

## winit 0.30.13

- Change: a graphics tablet's pen reports its pressure on X11 and macOS
  (#850). winit 0.30 carries pen pressure on Windows only, as a
  `WindowEvent::Touch` with a `force`, which egui-winit hands to egui as
  `Event::Touch { force }` and TraceDraw reads (`apps/tracedraw/src/pen.rs`,
  `docs/behavior/pen-pressure.md`). The copy does the same on the two
  other desktops:
  - X11 (`platform_impl/linux/x11/mod.rs`, `event_processor.rs`,
    `atoms.rs`): a slave device's valuator labelled `Abs Pressure` is
    remembered with its range when the device is listed
    (`Device::pressure_axis`); a motion event carrying that valuator
    sends, besides the `AxisMotion` it always sent, a
    `WindowEvent::Touch` of phase `Moved` at the pointer's place with the
    reading as a share of the axis (`Force::Normalized`), or no force at
    the axis's minimum (the pen lifted).
  - macOS (`platform_impl/macos/view.rs`): a mouse move or drag whose
    `NSEvent` subtype is a tablet point sends the same `Touch` with the
    event's `pressure` (already 0 to 1). `subtype` and `pressure` are
    read only when the event's type is a mouse down, up, move or drag
    (left, right, other) or a tablet point: the scroll wheel and the
    magnify, smart magnify and rotate gestures go through the same
    `mouse_motion`, and AppKit raises an exception for `subtype` on them.
  - `src/pen_pressure.rs`: the shared arithmetic, with its tests; it
    depends on nothing in the crate, and TraceDraw's tests include and
    run it (`winit_pen_pressure::tests`).
  - Only `Moved` touches are sent, never `Started` or `Ended`: egui-winit
    turns those into presses and releases of the primary button, which
    the pen's real button events already send. The pointer move
    egui-winit also makes of a `Moved` touch repeats the one just sent.
    egui itself would still take any `Event::Touch` for a touch screen
    for the rest of the session (it makes a touch state for the device,
    and `has_touch_screen` turns on: lists scroll by dragging, windows
    move from anywhere, labels lose drag selection), so TraceDraw's raw
    input hook reads the force and then takes out every `Moved` touch
    whose id never had a `Started` one (`pen::filter_raw_input`); a real
    touch screen's touches start, move and end, and pass as they came.
    Wayland: see the next change.
- Change: a graphics tablet's pen on a native Wayland window (#1841).
  winit 0.30 binds no tablet protocol, and a Wayland compositor sends a
  pen's input over nothing else (GNOME's sends a window that does not
  bind it nothing), so the pen did not move the pointer or draw; only
  XWayland (`tracedraw --x11`) took it. The copy binds the public
  tablet-v2 protocol (`zwp_tablet_manager_v2`, version 1, from
  `wayland-protocols`, already a dependency) and makes a tablet seat for
  each seat (`platform_impl/linux/wayland/state.rs`, `seat/mod.rs`);
  `platform_impl/linux/wayland/tablet.rs` takes the tablet seat's
  tablets, tools and pads, and hands each tool's events to
  `src/tablet_tool.rs`, which decides what the window is told: proximity
  in and out as `CursorEntered` and `CursorLeft`, motion as
  `CursorMoved`, the tip's down and up as presses and releases of the
  left button, the barrel buttons (`BTN_STYLUS`, `BTN_STYLUS2`,
  `BTN_STYLUS3`) as the middle, right and back buttons, and the
  pressure (0 to 65535) as the same moving `Touch` with a
  `Force::Normalized` that the X11 and macOS changes send, no force once
  the tip lifts; a frame's events go out when the frame ends, and
  leaving proximity releases whatever is still held. A tool that comes
  into proximity of a window shows the window's own cursor, as its
  pointers do, and follows it when it changes (`WindowState::
  tablet_tool_entered`, `apply_on_tablets`, called from `set_cursor`,
  `set_custom_cursor` and `set_cursor_visible`): a named cursor through
  `wp_cursor_shape_manager_v1` (`tablet_tool::cursor_shape_value`), a
  custom image on a surface of the tool's own (`zwp_tablet_tool_v2.
  set_cursor`), none while the window hides its cursor (TraceDraw draws
  its tool cursors itself, so the pen showed the compositor's arrow on
  top of them before); over the window's frame the default arrow. The
  tilt is read and goes out with the pressure as the pen's altitude in
  `Force::Calibrated { force, max_possible_force: 1, altitude_angle }`,
  whose share egui-winit reads unchanged (`force / max_possible_force`).
  Distance, rotation, slider and wheel, and the pad's buttons, rings and
  strips, are not read; over the window's frame the pen does nothing.
  `src/tablet_tool.rs` depends on nothing in the crate, and TraceDraw's
  tests include and run it (`winit_tablet_tool::tests`).
- Change: macOS hands the files opened from Finder (double click, Open
  With, a file dropped on the Dock icon) to the program (#646). macOS sends
  them as an "open documents" event, never on the command line, and winit
  0.30 has no event for it. The application delegate
  (`platform_impl/macos/app_state.rs`) answers `application:openURLs:`
  and passes the paths of the file URLs to `src/opened_files.rs`, which
  calls the handler the program set with
  `platform::macos::set_open_files_handler`, or keeps them until one is
  set (at launch they come before the window exists). The manifest asks
  `objc2-foundation` for `NSURL` as well. `src/opened_files.rs` depends on
  nothing in the crate, and TraceDraw's tests include and run it
  (`winit_opened_files::tests`).
- Change: the pointer's place while files are dragged over the window
  (#1142). winit 0.30's file drag events (`HoveredFile`, `DroppedFile`)
  carry no position, and the system drag sends the window no mouse
  moves, so TraceDraw could not tell a drop on the drawing (import) from
  one on the menus or tabs (open in a tab of its own). The copy sends a
  `WindowEvent::CursorMoved` with the place, which egui-winit hands on as
  a pointer move:
  - X11 (`platform_impl/linux/x11/event_processor.rs`): each
    `XdndPosition` message, its root window coordinates translated to
    the window's (`TranslateCoordinates`), before the drop data is asked
    for.
  - macOS (`platform_impl/macos/window_delegate.rs`): `draggingEntered:`
    and `performDragOperation:` first, and a new `draggingUpdated:`
    (answering a copy, as `draggingEntered:` does), the dragging info's
    `draggingLocation` converted to the flipped view's points
    (`queue_drag_position`).
  - Windows keeps reading the cursor itself (`drop_files::drop_pointer`).
- Change: files dropped on a native Wayland window (#1142). winit 0.30
  binds no `wl_data_device`, so a Wayland window received no
  `HoveredFile` or `DroppedFile` at all and the compositor showed the drag
  as refused; only XWayland (`tracedraw --x11`) took drops. The copy binds
  the data device manager and makes a data device for each seat
  (`platform_impl/linux/wayland/state.rs`, `seat/mod.rs`), and
  `platform_impl/linux/wayland/data_device.rs` handles the drag with the
  toolkit's data device: an offer of `text/uri-list` over a window is
  accepted as a copy and read at once through the event loop (never a
  blocking read), so `HoveredFile` comes while the files hover; enter
  and motion send a `CursorMoved` at the place, as the X11 and macOS
  changes above do (over the window's frame, a surface of its own, a
  place outside the window, so the drop counts as one on the frame);
  the drop sends `DroppedFile` for each file once the list is read and
  then finishes and destroys the offer; a drag that leaves without a
  drop sends `HoveredFileCancelled`. What to send is decided by
  `src/file_drop.rs` (the list's parsing, RFC 2483, and the drag's
  states), which depends on nothing in the crate; TraceDraw's tests
  include and run it (`winit_file_drop::tests`).
- Change: macOS asks the program before the application quits (#1669).
  The default application menu's Quit item (Command+Q), the Dock icon's
  Quit and a log-out or shut-down send `terminate:`; winit 0.30's
  application delegate has no `applicationShouldTerminate:`, so AppKit
  terminated at once, `applicationWillTerminate:` ended the event loop
  and no window had a `CloseRequested`: TraceDraw could not ask about
  unsaved drawings. The delegate (`platform_impl/macos/app_state.rs`)
  now answers `applicationShouldTerminate:` by asking `src/quit_request.rs`,
  which calls the handler the program set with
  `platform::macos::set_quit_handler`: `true` cancels the terminate
  (`NSTerminateCancel`, the program then closes its window itself),
  `false` or no handler lets it go ahead (`NSTerminateNow`, as before).
  `src/quit_request.rs` depends on nothing in the crate, and TraceDraw's
  tests include and run it (`winit_quit_request::tests`).
- Change: Android reports each pointer with its tool to the program,
  for palm rejection (#1832). winit 0.30 turns every pointer of a
  `MotionEvent` into a `Touch` with no word of its tool (finger, stylus,
  eraser, mouse, palm) and drops the hover events of a pen near the
  screen, so a hand resting on a tablet drew like a finger. The Android
  backend (`platform_impl/android/mod.rs`) now asks
  `src/pointer_filter.rs` about each pointer of a motion event, with its
  device, id, tool (`pointer_tool`) and action, and about each pointer of
  a hover event; the filter the program set with
  `platform::android::set_pointer_filter` answers whether to deliver the
  touch and which touches of the event to end first with `Cancelled`.
  With no filter set every touch is delivered, as before. The file
  depends on nothing in the crate, and TraceDraw's tests include and run
  it (`palm::winit_pointer_filter::tests`); TraceDraw's filter is
  `apps/tracedraw/src/palm.rs`.
- Change: on Windows a window's theme set by the program reaches its
  title bar (#1920). `Window::set_theme` called only the undocumented
  composition attribute (`WCA_USEDARKMODECOLORS`) and `SetWindowTheme`,
  which left the caption light on Windows 11 and later Windows 10, and
  it neither kept the theme asked for nor redrew the frame.
  `platform_impl/windows/dark_mode.rs` (`try_theme`) now also sets the
  documented `DWMWA_USE_IMMERSIVE_DARK_MODE` (20, or 19 before Windows 10
  build 18985) with `DwmSetWindowAttribute` and, for a window on show,
  redraws the frame (`SetWindowPos` with `SWP_FRAMECHANGED`) so the title
  bar changes at once; `platform_impl/windows/window.rs` (`set_theme`)
  keeps the theme asked for as the preferred one, so a change of the
  system's theme (`WM_SETTINGCHANGE`) no longer puts the system's back,
  and records the theme the window now has.
- Change: on Windows a shortcut with Alt keeps the pointer up to date
  over a still mouse (#1922). For a moment the system showed its arrow,
  then the window got a `WM_MOUSELEAVE` with the pointer still over it:
  winit sent `CursorLeft`, and egui-winit applies no pointer until the
  next `CursorMoved`, so the pointer stayed the arrow, or the blank image
  of a drawn pointer (invisible), until the mouse moved. Both are the
  marks of the keyboard menu mode, which takes the mouse while it runs.
  `platform_impl/windows/event_loop.rs` now (1) answers a
  `WM_SYSCOMMAND` with `SC_KEYMENU` and a character other than Space
  itself in a window without a native menu bar, so no menu mode starts
  (Alt+Space, the window menu, and every other command still reach
  `DefWindowProcW`), and (2) on `WM_MOUSELEAVE` reads where the pointer
  is (`GetCursorPos`, `WindowFromPoint`): still in the client area with
  this window under it, no `CursorLeft` is sent; the inside flag is
  cleared all the same, so the next move enters again and asks for the
  leave tracking anew. Both decisions are in `src/alt_shortcuts.rs`,
  which depends on nothing in the crate; TraceDraw's tests include and
  run it (`winit_alt_shortcuts::tests`).
- The published crate's `examples/` and `tests/` stay as downloaded (the
  manifest lists them); they build in no gate.
- Drop the copy and the `[patch.crates-io]` entry when a winit release
  carries pen pressure on X11 and macOS (rust-windowing/winit#99) and
  pen input on Wayland, and
  passes on the files macOS asks to open, and
  check that egui-winit still maps `Touch` forces the same way.
  drag positions (winit 0.31's `DragEntered`/`DragMoved`/`DragDropped`
  carry them: move `drop_files` onto those) and file drops on Wayland,
  and check that egui-winit
  still maps `Touch` forces the same way.
