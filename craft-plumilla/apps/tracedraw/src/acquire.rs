//! File > Acquire Image: the source (WIA or TWAIN) and Acquire, which
//! brings the pictures a scanner or camera gives into the drawing as File >
//! Import does (see `docs/behavior/acquire-image.md`).
//!
//! WIA is reached through the Windows Image Acquisition device manager
//! (`IWiaDevMgr2`): its own dialog picks the device, scans or takes the
//! pictures and saves them to a folder of ours, whose files are then
//! imported. TWAIN goes through the source manager Windows carries (see
//! `twain`). Only the public interfaces of Windows and the public TWAIN
//! specification are used.

use crate::app::App;

#[cfg(windows)]
mod twain;

/// Where pictures are acquired from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Source {
    /// Windows Image Acquisition.
    #[default]
    Wia,
    /// TWAIN data sources.
    Twain,
}

/// The end of one acquisition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Acquired {
    /// The pictures saved, in order.
    Files(Vec<std::path::PathBuf>),
    /// The user closed the device's dialog.
    Cancelled,
    /// No scanner or camera is connected.
    NoDevice,
    /// The source cannot be used here, with why.
    Unavailable(String),
}

/// Whether the WIA device manager can be reached: asked once, since menus
/// ask every time they are drawn.
pub fn wia_available() -> bool {
    static WIA: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *WIA.get_or_init(imp::wia_available)
}

/// The WIA devices connected now (scanners and cameras), or `None` when
/// WIA cannot be reached. Asks the device manager only: no dialog.
pub fn wia_device_count() -> Option<usize> {
    imp::wia_device_count()
}

/// Whether the 64-bit TWAIN source manager is installed: asked once.
/// Whether it has sources is asked when acquiring, as WIA's devices are.
pub fn twain_available() -> bool {
    static TWAIN: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *TWAIN.get_or_init(imp::twain_available)
}

/// Run WIA's own dialog, over the window in front, and save what it
/// gives in `folder`.
fn acquire_wia(folder: &std::path::Path) -> Acquired {
    imp::acquire_wia(folder)
}

impl App {
    /// Whether Acquire is offered: a drawing is open and the chosen source
    /// can be used.
    pub fn can_acquire(&self) -> bool {
        self.has_document()
            && match self.acquire_source {
                Source::Wia => self.acquire_with.is_some() || wia_available(),
                Source::Twain => self.acquire_with.is_some() || twain_available(),
            }
    }

    /// File > Acquire Image > WIA / TWAIN.
    pub fn set_acquire_source(&mut self, source: Source) {
        if source == Source::Twain && !twain_available() {
            return;
        }
        self.acquire_source = source;
    }

    /// File > Acquire Image > Acquire: the source's dialog, then the
    /// pictures imported (selected, one undo step each as File > Import).
    pub fn acquire_image(&mut self) {
        if !self.can_acquire() {
            return;
        }
        let folder = std::env::temp_dir().join(format!(
            "craft-plumilla-acquire-{}-{}",
            std::process::id(),
            self.engine.revision()
        ));
        if let Err(e) = std::fs::create_dir_all(&folder) {
            self.status = crate::i18n::trf("status.acquire_failed", &[("e", &e.to_string())]);
            return;
        }
        let acquired = match &self.acquire_with {
            Some(f) => f(&folder),
            None => match self.acquire_source {
                Source::Wia => acquire_wia(&folder),
                Source::Twain => imp::acquire_twain(&folder),
            },
        };
        self.finish_acquire(acquired);
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// The pictures of one acquisition into the drawing, or the reason
    /// none came in the status bar.
    fn finish_acquire(&mut self, acquired: Acquired) {
        match acquired {
            Acquired::Files(files) => {
                let mut placed = Vec::new();
                for f in &files {
                    self.import_path(f);
                    placed.extend(self.selection.iter().copied());
                }
                if !placed.is_empty() {
                    self.select(placed);
                }
                self.status = crate::i18n::trf("status.acquired", &[("n", &files.len().to_string())]);
            }
            Acquired::Cancelled => {}
            Acquired::NoDevice => self.status = crate::i18n::tr("status.acquire_no_device"),
            Acquired::Unavailable(e) => {
                self.status = crate::i18n::trf("status.acquire_failed", &[("e", &e)]);
            }
        }
    }
}

#[cfg(windows)]
mod imp {
    use super::Acquired;
    use std::ffi::c_void;
    use windows_sys::core::{GUID, HRESULT};
    use windows_sys::Win32::Foundation::{SysAllocString, SysFreeString, SysStringLen, S_FALSE};
    use windows_sys::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_LOCAL_SERVER,
        COINIT_APARTMENTTHREADED,
    };

    const CLSID_WIA_DEV_MGR2: GUID = GUID {
        data1: 0xB6C2_92BC,
        data2: 0x7C88,
        data3: 0x41EE,
        data4: [0x8B, 0x54, 0x8E, 0xC9, 0x26, 0x17, 0xE5, 0x99],
    };
    const IID_IWIA_DEV_MGR2: GUID = GUID {
        data1: 0x79C0_7CF1,
        data2: 0xCBDD,
        data3: 0x41EE,
        data4: [0x8E, 0xC3, 0xF0, 0x00, 0x80, 0xCA, 0xDA, 0x7A],
    };
    /// `WIA_S_NO_DEVICE_AVAILABLE`.
    const NO_DEVICE: HRESULT = 0x0021_0006;
    /// `WIA_ERROR_OFFLINE` and friends from no device: `0x80210015`.
    const NO_DEVICE_ERROR: HRESULT = 0x8021_0015_u32 as HRESULT;
    /// `WIA_DEVINFO_ENUM_LOCAL`.
    const ENUM_LOCAL: i32 = 0x10;

    type This = *mut c_void;

    #[repr(C)]
    struct DevMgr2Vtbl {
        query_interface: unsafe extern "system" fn(This, *const GUID, *mut *mut c_void) -> HRESULT,
        add_ref: unsafe extern "system" fn(This) -> u32,
        release: unsafe extern "system" fn(This) -> u32,
        enum_device_info: unsafe extern "system" fn(This, i32, *mut This) -> HRESULT,
        create_device: usize,
        select_device_dlg: usize,
        select_device_dlg_id: usize,
        register_event_callback_interface: usize,
        register_event_callback_program: usize,
        register_event_callback_clsid: usize,
        #[allow(clippy::type_complexity)]
        get_image_dlg: unsafe extern "system" fn(
            This,
            i32,
            *const u16,
            isize,
            *const u16,
            *const u16,
            *mut i32,
            *mut *mut *mut u16,
            *mut This,
        ) -> HRESULT,
    }

    #[repr(C)]
    struct EnumDevInfoVtbl {
        query_interface: usize,
        add_ref: usize,
        release: unsafe extern "system" fn(This) -> u32,
        next: usize,
        skip: usize,
        reset: usize,
        clone: usize,
        get_count: unsafe extern "system" fn(This, *mut u32) -> HRESULT,
    }

    /// The device manager, released when dropped; COM set up on this
    /// thread while it lives.
    struct Manager {
        ptr: This,
        uninit: bool,
    }

    impl Manager {
        fn new() -> Option<Manager> {
            // SAFETY: COM calls with valid out pointers; the interface
            // pointer is checked before use and released in `drop`.
            unsafe {
                let hr = CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32);
                let uninit = hr >= 0;
                let mut ptr: This = std::ptr::null_mut();
                let hr = CoCreateInstance(
                    &CLSID_WIA_DEV_MGR2,
                    std::ptr::null_mut(),
                    CLSCTX_LOCAL_SERVER,
                    &IID_IWIA_DEV_MGR2,
                    &mut ptr,
                );
                if hr < 0 || ptr.is_null() {
                    if uninit {
                        CoUninitialize();
                    }
                    return None;
                }
                Some(Manager { ptr, uninit })
            }
        }

        fn vtbl(&self) -> &DevMgr2Vtbl {
            // SAFETY: a COM interface pointer starts with its vtable.
            unsafe { &**(self.ptr as *const *const DevMgr2Vtbl) }
        }
    }

    impl Drop for Manager {
        fn drop(&mut self) {
            // SAFETY: the pointer came from CoCreateInstance.
            unsafe {
                (self.vtbl().release)(self.ptr);
                if self.uninit {
                    CoUninitialize();
                }
            }
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn wia_available() -> bool {
        Manager::new().is_some()
    }

    pub fn wia_device_count() -> Option<usize> {
        let m = Manager::new()?;
        let mut e: This = std::ptr::null_mut();
        // SAFETY: valid out pointers; the enumerator is released after use.
        unsafe {
            if (m.vtbl().enum_device_info)(m.ptr, ENUM_LOCAL, &mut e) < 0 || e.is_null() {
                return None;
            }
            let ev = &**(e as *const *const EnumDevInfoVtbl);
            let mut n = 0u32;
            let hr = (ev.get_count)(e, &mut n);
            (ev.release)(e);
            (hr >= 0).then_some(n as usize)
        }
    }

    pub fn twain_available() -> bool {
        super::twain::dsm_path().is_some()
    }

    /// TWAIN's source chosen and its dialog, over the window in front.
    pub fn acquire_twain(folder: &std::path::Path) -> Acquired {
        // SAFETY: no arguments; returns the window in front or null.
        let parent =
            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() } as isize;
        super::twain::acquire_installed(parent, folder)
    }

    pub fn acquire_wia(folder: &std::path::Path) -> Acquired {
        // SAFETY: no arguments; returns the window in front or null.
        let parent =
            unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() } as isize;
        let Some(m) = Manager::new() else {
            return Acquired::Unavailable("WIA".into());
        };
        let folder_w = wide(&folder.to_string_lossy());
        let name_w = wide("picture");
        // SAFETY: BSTRs made and freed here; the returned array and its
        // strings are freed as the interface's contract says.
        unsafe {
            let folder_b = SysAllocString(folder_w.as_ptr());
            let name_b = SysAllocString(name_w.as_ptr());
            let mut count: i32 = 0;
            let mut paths: *mut *mut u16 = std::ptr::null_mut();
            let mut item: This = std::ptr::null_mut();
            let hr = (m.vtbl().get_image_dlg)(
                m.ptr,
                0,
                std::ptr::null(),
                parent,
                folder_b,
                name_b,
                &mut count,
                &mut paths,
                &mut item,
            );
            SysFreeString(folder_b);
            SysFreeString(name_b);
            if !item.is_null() {
                let v = &**(item as *const *const EnumDevInfoVtbl);
                (v.release)(item);
            }
            let mut files = Vec::new();
            if !paths.is_null() {
                for i in 0..count.max(0) as usize {
                    let b = *paths.add(i);
                    if b.is_null() {
                        continue;
                    }
                    let len = SysStringLen(b) as usize;
                    let s = String::from_utf16_lossy(std::slice::from_raw_parts(b, len));
                    files.push(std::path::PathBuf::from(s));
                    SysFreeString(b);
                }
                CoTaskMemFree(paths as *const c_void);
            }
            match hr {
                NO_DEVICE | NO_DEVICE_ERROR => Acquired::NoDevice,
                S_FALSE => Acquired::Cancelled,
                h if h < 0 => Acquired::Unavailable(format!("WIA 0x{:08X}", h as u32)),
                _ if files.is_empty() => Acquired::Cancelled,
                _ => Acquired::Files(files),
            }
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use super::Acquired;

    pub fn wia_available() -> bool {
        false
    }

    pub fn wia_device_count() -> Option<usize> {
        None
    }

    pub fn twain_available() -> bool {
        false
    }

    pub fn acquire_twain(_folder: &std::path::Path) -> Acquired {
        Acquired::Unavailable("TWAIN".into())
    }

    pub fn acquire_wia(_folder: &std::path::Path) -> Acquired {
        Acquired::Unavailable("WIA".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::menus::run_command;

    fn app_and_ctx() -> (App, egui::Context) {
        crate::i18n::set_language("en");
        let app = App::headless();
        let ctx = crate::theme::ui_context();
        let mut out = ctx.run_ui(egui::RawInput::default(), |_| {});
        out.textures_delta.clear();
        (app, ctx)
    }

    /// Asking Windows for its scanners and cameras shows no dialog and
    /// answers; on a machine without any it says none.
    #[cfg(windows)]
    #[test]
    fn windows_answers_how_many_devices_it_has() {
        if !wia_available() {
            return;
        }
        let n = wia_device_count().expect("a count");
        println!("::notice::WIA devices: {n}");
    }

    /// With no scanner or camera, WIA's dialog answers so at once, with no
    /// window (the real call; skipped where a device is connected).
    #[cfg(windows)]
    #[test]
    fn the_real_dialog_says_when_there_is_no_device() {
        if wia_device_count() != Some(0) {
            return;
        }
        let dir = std::env::temp_dir().join(format!("acquire-none-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        assert_eq!(acquire_wia(&dir), Acquired::NoDevice);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// File > Acquire Image > Acquire: the pictures the device gave come
    /// into the drawing as File > Import brings them, selected, each its
    /// own undo step; no device and a cancelled dialog leave the drawing
    /// as it was and say why.
    #[test]
    fn acquired_pictures_come_in_like_imported_ones() {
        let (mut app, ctx) = app_and_ctx();
        let dir = std::env::temp_dir().join(format!("acquire-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let mut paths = Vec::new();
        for (i, c) in [[200u8, 0, 0, 255], [0, 0, 200, 255]].iter().enumerate() {
            let img = image::RgbaImage::from_pixel(30, 20, image::Rgba(*c));
            let p = dir.join(format!("picture{i}.png"));
            img.save(&p).expect("png");
            paths.push(p);
        }
        let given = paths.clone();
        app.acquire_with = Some(std::rc::Rc::new(move |_| Acquired::Files(given.clone())));
        assert!(run_command(&mut app, &ctx, "menu.file.acquire"), "Acquire is offered");
        let count = |app: &App| app.doc().pages[0].layers.iter().map(|l| l.shapes.len()).sum::<usize>();
        assert_eq!(count(&app), 2, "{}", app.status);
        assert_eq!(app.selection.len(), 2);
        for id in &app.selection {
            let s = app.doc().find_shape(*id).expect("picture");
            assert!(matches!(s.kind, tracedraw_core::ShapeKind::Bitmap { .. }));
        }
        app.undo();
        app.undo();
        assert_eq!(count(&app), 0);
        for (answer, key) in [
            (Acquired::NoDevice, "status.acquire_no_device"),
            (Acquired::Cancelled, ""),
        ] {
            let a = answer.clone();
            app.status.clear();
            app.acquire_with = Some(std::rc::Rc::new(move |_| a.clone()));
            assert!(run_command(&mut app, &ctx, "menu.file.acquire"));
            assert_eq!(count(&app), 0);
            let want = if key.is_empty() { String::new() } else { crate::i18n::tr(key) };
            assert_eq!(app.status, want);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The source items: WIA is the default; TWAIN can be chosen where its
    /// source manager is installed, and Acquire then goes through it.
    #[test]
    fn the_source_items_reflect_what_can_be_used() {
        let (mut app, ctx) = app_and_ctx();
        assert_eq!(app.acquire_source, Source::Wia);
        assert_eq!(run_command(&mut app, &ctx, "menu.file.acquire_twain"), twain_available());
        let want = if twain_available() { Source::Twain } else { Source::Wia };
        assert_eq!(app.acquire_source, want);
        assert_eq!(app.can_acquire(), twain_available() || wia_available());
        assert_eq!(
            run_command(&mut app, &ctx, "menu.file.acquire_wia"),
            wia_available()
        );
        if wia_available() {
            assert_eq!(app.acquire_source, Source::Wia);
        }
    }
}
