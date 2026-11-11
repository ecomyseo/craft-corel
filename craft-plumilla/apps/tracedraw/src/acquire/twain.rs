//! TWAIN: File > Acquire Image with the TWAIN source.
//!
//! The 64-bit source manager Windows carries (`TWAINDSM.dll`) is loaded and
//! spoken to as the public TWAIN 2 specification describes: open the
//! manager over our window, list the sources, let the user choose one when
//! there are several (the manager's own Select Source dialog), open it,
//! show its own dialog, take every picture it says is ready as a native
//! transfer (a packed DIB in global memory) and save each as a BMP file of
//! ours, which is then imported as WIA's pictures are. Events reach us
//! both ways the specification allows: through the window messages handed
//! to the source (`MSG_PROCESSEVENT`) and, with version 2 sources, through
//! the callback registered with the manager.

// The structures carry every field the specification gives: the manager
// and the source read them, this side only fills them in.
#![allow(dead_code)]

use super::Acquired;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU16, Ordering};

/// `DSM_Entry`, and the callback a version 2 source calls.
pub type Entry =
    unsafe extern "system" fn(*mut Identity, *mut Identity, u32, u16, u16, *mut c_void) -> u16;

pub const DG_CONTROL: u32 = 0x0001;
pub const DG_IMAGE: u32 = 0x0002;
pub const DF_APP2: u32 = 0x2000_0000;
pub const DF_DS2: u32 = 0x4000_0000;

pub const DAT_EVENT: u16 = 0x0002;
pub const DAT_IDENTITY: u16 = 0x0003;
pub const DAT_PARENT: u16 = 0x0004;
pub const DAT_PENDINGXFERS: u16 = 0x0005;
pub const DAT_USERINTERFACE: u16 = 0x0009;
pub const DAT_CALLBACK: u16 = 0x0010;
pub const DAT_IMAGENATIVEXFER: u16 = 0x0104;

pub const MSG_NULL: u16 = 0x0000;
pub const MSG_GET: u16 = 0x0001;
pub const MSG_GETFIRST: u16 = 0x0004;
pub const MSG_GETNEXT: u16 = 0x0005;
pub const MSG_RESET: u16 = 0x0007;
pub const MSG_XFERREADY: u16 = 0x0101;
pub const MSG_CLOSEDSREQ: u16 = 0x0102;
pub const MSG_CLOSEDSOK: u16 = 0x0103;
pub const MSG_OPENDSM: u16 = 0x0301;
pub const MSG_CLOSEDSM: u16 = 0x0302;
pub const MSG_OPENDS: u16 = 0x0401;
pub const MSG_CLOSEDS: u16 = 0x0402;
pub const MSG_USERSELECT: u16 = 0x0403;
pub const MSG_DISABLEDS: u16 = 0x0501;
pub const MSG_ENABLEDS: u16 = 0x0502;
pub const MSG_PROCESSEVENT: u16 = 0x0601;
pub const MSG_ENDXFER: u16 = 0x0701;
pub const MSG_REGISTER_CALLBACK: u16 = 0x0902;

pub const TWRC_SUCCESS: u16 = 0;
pub const TWRC_FAILURE: u16 = 1;
pub const TWRC_CHECKSTATUS: u16 = 2;
pub const TWRC_CANCEL: u16 = 3;
pub const TWRC_DSEVENT: u16 = 4;
pub const TWRC_NOTDSEVENT: u16 = 5;
pub const TWRC_XFERDONE: u16 = 6;
pub const TWRC_ENDOFLIST: u16 = 7;

/// `TW_VERSION` (structures are packed on 2 bytes on Windows).
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub language: u16,
    pub country: u16,
    pub info: [u8; 34],
}

/// `TW_IDENTITY`: who calls, and which source is meant.
#[repr(C, packed(2))]
#[derive(Clone, Copy)]
pub struct Identity {
    pub id: u32,
    pub version: Version,
    pub protocol_major: u16,
    pub protocol_minor: u16,
    pub supported_groups: u32,
    pub manufacturer: [u8; 34],
    pub product_family: [u8; 34],
    pub product_name: [u8; 34],
}

impl Identity {
    pub fn empty() -> Identity {
        Identity {
            id: 0,
            version: Version { major: 0, minor: 0, language: 0, country: 0, info: [0; 34] },
            protocol_major: 0,
            protocol_minor: 0,
            supported_groups: 0,
            manufacturer: [0; 34],
            product_family: [0; 34],
            product_name: [0; 34],
        }
    }

    /// The product name, for messages.
    pub fn name(&self) -> String {
        let n = self.product_name;
        let end = n.iter().position(|&b| b == 0).unwrap_or(n.len());
        String::from_utf8_lossy(&n[..end]).into_owned()
    }
}

/// `TW_USERINTERFACE`.
#[repr(C, packed(2))]
pub struct UserInterface {
    pub show_ui: u16,
    pub modal_ui: u16,
    pub parent: *mut c_void,
}

/// `TW_EVENT`.
#[repr(C, packed(2))]
pub struct Event {
    pub event: *mut c_void,
    pub message: u16,
}

/// `TW_PENDINGXFERS`.
#[repr(C, packed(2))]
pub struct PendingXfers {
    pub count: u16,
    pub eoj: u32,
}

/// `TW_CALLBACK`.
#[repr(C, packed(2))]
pub struct Callback {
    pub proc_: *mut c_void,
    pub ref_con: u32,
    pub message: i16,
}

fn str34(s: &str) -> [u8; 34] {
    let mut out = [0u8; 34];
    for (o, b) in out.iter_mut().zip(s.bytes().take(33)) {
        *o = b;
    }
    out
}

/// Who we are to the source manager.
pub fn app_identity() -> Identity {
    let mut id = Identity::empty();
    let v: Vec<u16> = env!("CARGO_PKG_VERSION").split('.').filter_map(|p| p.parse().ok()).collect();
    id.version = Version {
        major: v.first().copied().unwrap_or(0),
        minor: v.get(1).copied().unwrap_or(0),
        language: 2, // TWLG_ENGLISH
        country: 1,  // TWCY_USA
        info: str34(env!("CARGO_PKG_VERSION")),
    };
    id.protocol_major = 2;
    id.protocol_minor = 4;
    id.supported_groups = DG_CONTROL | DG_IMAGE | DF_APP2;
    id.manufacturer = str34("Craft Plumilla");
    id.product_family = str34("Craft Plumilla");
    id.product_name = str34("Craft Plumilla");
    id
}

/// The message a version 2 source last sent through the callback.
static PENDING: AtomicU16 = AtomicU16::new(MSG_NULL);

unsafe extern "system" fn callback(
    _origin: *mut Identity,
    _dest: *mut Identity,
    _dg: u32,
    _dat: u16,
    msg: u16,
    _data: *mut c_void,
) -> u16 {
    PENDING.store(msg, Ordering::SeqCst);
    TWRC_SUCCESS
}

/// One triplet to the manager, or through it to `dest`.
unsafe fn call(
    entry: Entry,
    app: &mut Identity,
    dest: Option<&mut Identity>,
    dg: u32,
    dat: u16,
    msg: u16,
    data: *mut c_void,
) -> u16 {
    let dest = dest.map_or(std::ptr::null_mut(), |d| d as *mut Identity);
    entry(app, dest, dg, dat, msg, data)
}

fn data<T>(v: &mut T) -> *mut c_void {
    v as *mut T as *mut c_void
}

/// A packed DIB (header, colour table, pixels) as the bytes of a BMP file:
/// the 14-byte file header that says where the pixels start, then the DIB
/// untouched, so resolution and colours stay as the source gave them.
pub fn dib_to_bmp(dib: &[u8]) -> Option<Vec<u8>> {
    let u16_at = |o: usize| dib.get(o..o + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let u32_at = |o: usize| dib.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    let header = u32_at(0)? as usize;
    if header < 12 || header > dib.len() {
        return None;
    }
    let (bits, compression, used, entry) = if header == 12 {
        (u16_at(10)?, 0, 0, 3)
    } else if header >= 40 {
        (u16_at(14)?, u32_at(16)?, u32_at(32)?, 4)
    } else {
        return None;
    };
    let colours = if used != 0 {
        used as usize
    } else if bits <= 8 {
        1usize << bits
    } else {
        0
    };
    // BI_BITFIELDS and BI_ALPHABITFIELDS put their masks after a 40-byte header.
    let masks = match (compression, header) {
        (3, 40) => 12,
        (6, 40) => 16,
        _ => 0,
    };
    let offset = 14 + header + colours * entry + masks;
    if offset > 14 + dib.len() {
        return None;
    }
    let mut out = Vec::with_capacity(14 + dib.len());
    out.extend_from_slice(b"BM");
    out.extend_from_slice(&((14 + dib.len()) as u32).to_le_bytes());
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&(offset as u32).to_le_bytes());
    out.extend_from_slice(dib);
    Some(out)
}

/// Acquire through the source manager reached by `entry`, over the window
/// `parent`, saving the pictures in `folder`.
///
/// # Safety
/// `entry` must behave as `DSM_Entry` does, and `parent` be a window of
/// this thread or 0.
pub unsafe fn acquire(entry: Entry, parent: isize, folder: &Path) -> Acquired {
    let mut app = app_identity();
    let mut parent_h = parent;
    if call(entry, &mut app, None, DG_CONTROL, DAT_PARENT, MSG_OPENDSM, data(&mut parent_h))
        != TWRC_SUCCESS
    {
        return Acquired::Unavailable("TWAIN".into());
    }
    let mut sources = Vec::new();
    let mut first = true;
    loop {
        let mut s = Identity::empty();
        let msg = if first { MSG_GETFIRST } else { MSG_GETNEXT };
        first = false;
        if call(entry, &mut app, None, DG_CONTROL, DAT_IDENTITY, msg, data(&mut s)) != TWRC_SUCCESS {
            break;
        }
        sources.push(s);
    }
    let result = match sources.len() {
        0 => Acquired::NoDevice,
        1 => session(entry, &mut app, &mut sources[0], parent, folder),
        _ => {
            let mut chosen = Identity::empty();
            match call(entry, &mut app, None, DG_CONTROL, DAT_IDENTITY, MSG_USERSELECT, data(&mut chosen)) {
                TWRC_SUCCESS => session(entry, &mut app, &mut chosen, parent, folder),
                TWRC_CANCEL => Acquired::Cancelled,
                _ => Acquired::Unavailable("TWAIN".into()),
            }
        }
    };
    call(entry, &mut app, None, DG_CONTROL, DAT_PARENT, MSG_CLOSEDSM, data(&mut parent_h));
    result
}

/// One source opened, its dialog shown, the pictures taken, and the source
/// closed again whatever happened.
unsafe fn session(
    entry: Entry,
    app: &mut Identity,
    source: &mut Identity,
    parent: isize,
    folder: &Path,
) -> Acquired {
    if call(entry, app, None, DG_CONTROL, DAT_IDENTITY, MSG_OPENDS, data(source)) != TWRC_SUCCESS {
        return Acquired::Unavailable(format!("TWAIN: {}", source.name()));
    }
    PENDING.store(MSG_NULL, Ordering::SeqCst);
    if source.supported_groups & DF_DS2 != 0 {
        let mut cb = Callback { proc_: callback as Entry as *mut c_void, ref_con: 0, message: 0 };
        call(entry, app, Some(source), DG_CONTROL, DAT_CALLBACK, MSG_REGISTER_CALLBACK, data(&mut cb));
    }
    let mut ui = UserInterface { show_ui: 1, modal_ui: 0, parent: parent as *mut c_void };
    let result = match call(entry, app, Some(source), DG_CONTROL, DAT_USERINTERFACE, MSG_ENABLEDS, data(&mut ui)) {
        TWRC_SUCCESS | TWRC_CHECKSTATUS => {
            let files = pump(entry, app, source, folder);
            let mut ui = UserInterface { show_ui: 1, modal_ui: 0, parent: parent as *mut c_void };
            call(entry, app, Some(source), DG_CONTROL, DAT_USERINTERFACE, MSG_DISABLEDS, data(&mut ui));
            match files {
                Ok(f) if f.is_empty() => Acquired::Cancelled,
                Ok(f) => Acquired::Files(f),
                Err(e) => Acquired::Unavailable(e),
            }
        }
        TWRC_CANCEL => Acquired::Cancelled,
        _ => Acquired::Unavailable(format!("TWAIN: {}", source.name())),
    };
    call(entry, app, None, DG_CONTROL, DAT_IDENTITY, MSG_CLOSEDS, data(source));
    result
}

/// The thread's messages while the source's dialog is open: each is first
/// offered to the source; the first batch of pictures it has ready ends
/// the wait, and so does the source asking to be closed.
unsafe fn pump(
    entry: Entry,
    app: &mut Identity,
    source: &mut Identity,
    folder: &Path,
) -> Result<Vec<PathBuf>, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
        QS_ALLINPUT,
    };
    loop {
        let mut got = MSG_NULL;
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
            let mut ev = Event { event: data(&mut msg), message: MSG_NULL };
            if call(entry, app, Some(source), DG_CONTROL, DAT_EVENT, MSG_PROCESSEVENT, data(&mut ev))
                == TWRC_DSEVENT
            {
                if ev.message != MSG_NULL {
                    got = ev.message;
                    break;
                }
            } else {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        if got == MSG_NULL {
            got = PENDING.swap(MSG_NULL, Ordering::SeqCst);
        }
        match got {
            MSG_XFERREADY => return transfer(entry, app, source, folder),
            MSG_CLOSEDSREQ | MSG_CLOSEDSOK => return Ok(Vec::new()),
            _ => {
                MsgWaitForMultipleObjects(0, std::ptr::null(), 0, 50, QS_ALLINPUT);
            }
        }
    }
}

/// Every picture the source has ready, as native transfers, each saved as
/// a BMP file in `folder`.
unsafe fn transfer(
    entry: Entry,
    app: &mut Identity,
    source: &mut Identity,
    folder: &Path,
) -> Result<Vec<PathBuf>, String> {
    use windows_sys::Win32::Foundation::GlobalFree;
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};
    let mut files = Vec::new();
    loop {
        let mut handle: *mut c_void = std::ptr::null_mut();
        let rc = call(entry, app, Some(source), DG_IMAGE, DAT_IMAGENATIVEXFER, MSG_GET, data(&mut handle));
        if rc == TWRC_XFERDONE && !handle.is_null() {
            let p = GlobalLock(handle) as *const u8;
            let saved = if p.is_null() {
                None
            } else {
                let dib = std::slice::from_raw_parts(p, GlobalSize(handle));
                let bmp = dib_to_bmp(dib);
                GlobalUnlock(handle);
                bmp.and_then(|b| {
                    let path = folder.join(format!("picture{}.bmp", files.len() + 1));
                    std::fs::write(&path, b).ok().map(|_| path)
                })
            };
            GlobalFree(handle);
            match saved {
                Some(path) => files.push(path),
                None => {
                    let mut p = PendingXfers { count: 0, eoj: 0 };
                    call(entry, app, Some(source), DG_CONTROL, DAT_PENDINGXFERS, MSG_RESET, data(&mut p));
                    return Err(format!("TWAIN: {}", source.name()));
                }
            }
        } else if rc != TWRC_CANCEL {
            let mut p = PendingXfers { count: 0, eoj: 0 };
            call(entry, app, Some(source), DG_CONTROL, DAT_PENDINGXFERS, MSG_RESET, data(&mut p));
            return if files.is_empty() { Err(format!("TWAIN: {}", source.name())) } else { Ok(files) };
        }
        let mut p = PendingXfers { count: 0, eoj: 0 };
        call(entry, app, Some(source), DG_CONTROL, DAT_PENDINGXFERS, MSG_ENDXFER, data(&mut p));
        if p.count == 0 {
            return Ok(files);
        }
    }
}

/// The source manager installed with Windows, when there is one.
pub fn dsm_path() -> Option<PathBuf> {
    let windows = PathBuf::from(std::env::var_os("SystemRoot")?);
    let p = windows.join("System32").join("TWAINDSM.dll");
    p.is_file().then_some(p)
}

/// Acquire through the installed source manager, over `parent`.
pub fn acquire_installed(parent: isize, folder: &Path) -> Acquired {
    use windows_sys::Win32::Foundation::FreeLibrary;
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    let Some(path) = dsm_path() else {
        return Acquired::Unavailable("TWAIN".into());
    };
    let wide: Vec<u16> = path.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: the library is the TWAIN source manager; `DSM_Entry` has the
    // signature the specification gives, and the library is freed after
    // the manager was closed.
    unsafe {
        let lib = LoadLibraryW(wide.as_ptr());
        if lib == 0 {
            return Acquired::Unavailable("TWAIN".into());
        }
        let result = match GetProcAddress(lib, b"DSM_Entry\0".as_ptr()) {
            Some(f) => acquire(std::mem::transmute::<_, Entry>(f), parent, folder),
            None => Acquired::Unavailable("TWAIN".into()),
        };
        FreeLibrary(lib);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    /// The tests share the callback's slot: one at a time.
    static ONE_AT_A_TIME: std::sync::Mutex<()> = std::sync::Mutex::new(());

    /// A source manager with sources made up for the test, keeping the
    /// triplets it was sent and the TWAIN state it is in.
    #[derive(Default)]
    struct Fake {
        sources: Vec<(&'static str, u32)>,
        listed: usize,
        user_picks: Option<usize>,
        pictures: Vec<Vec<u8>>,
        /// What the source says once enabled: pictures ready or close me.
        says: u16,
        /// Through the callback (version 2) rather than window messages.
        by_callback: bool,
        callback: Option<Entry>,
        log: Vec<(u16, u16)>,
        state: u8,
        opened: Option<String>,
        /// Triplets sent in the wrong state (an assert would abort inside
        /// the callback, so they are kept and asserted after).
        errors: Vec<String>,
        /// Closes the source's dialog if the test's source is never heard,
        /// so that a broken protocol fails instead of waiting for ever.
        watchdog: usize,
    }

    impl Fake {
        fn expect(&mut self, state: u8, what: &str) {
            if self.state != state {
                self.errors.push(format!("{what} (state {} instead of {state})", self.state));
            }
        }
    }

    thread_local! {
        static FAKE: RefCell<Fake> = RefCell::new(Fake::default());
    }

    const POKE: u32 = windows_sys::Win32::UI::WindowsAndMessaging::WM_USER + 77;

    unsafe extern "system" fn fake_entry(
        _origin: *mut Identity,
        _dest: *mut Identity,
        _dg: u32,
        dat: u16,
        msg: u16,
        pdata: *mut c_void,
    ) -> u16 {
        use windows_sys::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
        use windows_sys::Win32::System::Threading::GetCurrentThreadId;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            KillTimer, PostThreadMessageW, SetTimer, MSG, WM_TIMER,
        };
        let mut fire: Option<(Entry, u16)> = None;
        let rc = FAKE.with(|f| {
            let mut f = f.borrow_mut();
            f.log.push((dat, msg));
            match (dat, msg) {
                (DAT_PARENT, MSG_OPENDSM) => {
                    f.state = 3;
                    TWRC_SUCCESS
                }
                (DAT_PARENT, MSG_CLOSEDSM) => {
                    f.expect(3, "the manager is closed with no source open");
                    f.state = 2;
                    TWRC_SUCCESS
                }
                (DAT_IDENTITY, MSG_GETFIRST) | (DAT_IDENTITY, MSG_GETNEXT) => {
                    if msg == MSG_GETFIRST {
                        f.listed = 0;
                    }
                    let Some(&(name, groups)) = f.sources.get(f.listed) else {
                        return TWRC_ENDOFLIST;
                    };
                    f.listed += 1;
                    let s = &mut *(pdata as *mut Identity);
                    s.id = f.listed as u32;
                    s.product_name = str34(name);
                    s.supported_groups = groups;
                    TWRC_SUCCESS
                }
                (DAT_IDENTITY, MSG_USERSELECT) => match f.user_picks {
                    Some(i) => {
                        let (name, groups) = f.sources[i];
                        let s = &mut *(pdata as *mut Identity);
                        s.id = i as u32 + 1;
                        s.product_name = str34(name);
                        s.supported_groups = groups;
                        TWRC_SUCCESS
                    }
                    None => TWRC_CANCEL,
                },
                (DAT_IDENTITY, MSG_OPENDS) => {
                    f.expect(3, "state");
                    f.state = 4;
                    f.opened = Some((*(pdata as *const Identity)).name());
                    TWRC_SUCCESS
                }
                (DAT_IDENTITY, MSG_CLOSEDS) => {
                    f.expect(4, "the source is disabled before it is closed");
                    f.state = 3;
                    TWRC_SUCCESS
                }
                (DAT_CALLBACK, MSG_REGISTER_CALLBACK) => {
                    let cb = &*(pdata as *const Callback);
                    f.callback = Some(std::mem::transmute::<*mut c_void, Entry>(cb.proc_));
                    TWRC_SUCCESS
                }
                (DAT_USERINTERFACE, MSG_ENABLEDS) => {
                    f.expect(4, "state");
                    f.state = 5;
                    f.watchdog = SetTimer(0, 0, 3000, None);
                    if f.by_callback {
                        match f.callback {
                            Some(cb) => fire = Some((cb, f.says)),
                            None => f.errors.push("a version 2 source enabled without the callback".into()),
                        }
                    } else {
                        PostThreadMessageW(GetCurrentThreadId(), POKE, 0, 0);
                    }
                    TWRC_SUCCESS
                }
                (DAT_USERINTERFACE, MSG_DISABLEDS) => {
                    f.expect(5, "only an enabled source with nothing pending is disabled");
                    f.state = 4;
                    KillTimer(0, f.watchdog);
                    TWRC_SUCCESS
                }
                (DAT_EVENT, MSG_PROCESSEVENT) => {
                    let ev = &mut *(pdata as *mut Event);
                    let m = &*(ev.event as *const MSG);
                    if m.message == WM_TIMER && m.wParam == f.watchdog {
                        f.errors.push("the source was never heard; closed after 3 s".into());
                        ev.message = MSG_CLOSEDSREQ;
                        TWRC_DSEVENT
                    } else if m.message == POKE {
                        ev.message = f.says;
                        if f.says == MSG_XFERREADY {
                            f.state = 6;
                        }
                        TWRC_DSEVENT
                    } else {
                        ev.message = MSG_NULL;
                        TWRC_NOTDSEVENT
                    }
                }
                (DAT_IMAGENATIVEXFER, MSG_GET) => {
                    f.expect(6, "a picture is asked for only when one is ready");
                    if f.pictures.is_empty() {
                        f.errors.push("a picture asked for when none was left".into());
                        return TWRC_FAILURE;
                    }
                    let dib = f.pictures.remove(0);
                    let h = GlobalAlloc(GMEM_MOVEABLE, dib.len());
                    let p = GlobalLock(h) as *mut u8;
                    std::ptr::copy_nonoverlapping(dib.as_ptr(), p, dib.len());
                    GlobalUnlock(h);
                    *(pdata as *mut *mut c_void) = h;
                    f.state = 7;
                    TWRC_XFERDONE
                }
                (DAT_PENDINGXFERS, MSG_ENDXFER) => {
                    f.expect(7, "each transfer is ended after it is taken");
                    let left = f.pictures.len() as u16;
                    (*(pdata as *mut PendingXfers)).count = left;
                    f.state = if left == 0 { 5 } else { 6 };
                    TWRC_SUCCESS
                }
                _ => TWRC_FAILURE,
            }
        });
        if let Some((cb, says)) = fire {
            if says == MSG_XFERREADY {
                FAKE.with(|f| f.borrow_mut().state = 6);
            }
            cb(std::ptr::null_mut(), std::ptr::null_mut(), DG_CONTROL, DAT_NULL_FOR_TEST, says, std::ptr::null_mut());
        }
        rc
    }

    const DAT_NULL_FOR_TEST: u16 = 0;

    /// A bottom-up 24-bit DIB of `w` by `h` whose pixel at (x, y), counted
    /// from the top, is `px(x, y)`.
    fn dib24(w: u32, h: u32, px: impl Fn(u32, u32) -> [u8; 3]) -> Vec<u8> {
        let row = (w * 3).div_ceil(4) * 4;
        let mut d = Vec::new();
        for v in [40u32, w, h] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&24u16.to_le_bytes());
        d.extend_from_slice(&0u32.to_le_bytes());
        d.extend_from_slice(&(row * h).to_le_bytes());
        for v in [11811u32, 11811, 0, 0] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        for y in (0..h).rev() {
            let start = d.len();
            for x in 0..w {
                let [r, g, b] = px(x, y);
                d.extend_from_slice(&[b, g, r]);
            }
            d.resize(start + row as usize, 0);
        }
        d
    }

    /// An 8-bit DIB with a two-colour table: black on the left half,
    /// yellow on the right.
    fn dib8(w: u32, h: u32) -> Vec<u8> {
        let row = w.div_ceil(4) * 4;
        let mut d = Vec::new();
        for v in [40u32, w, h] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        d.extend_from_slice(&1u16.to_le_bytes());
        d.extend_from_slice(&8u16.to_le_bytes());
        for v in [0u32, row * h, 2835, 2835, 2, 0] {
            d.extend_from_slice(&v.to_le_bytes());
        }
        d.extend_from_slice(&[0, 0, 0, 0, 0, 255, 255, 0]);
        for _ in 0..h {
            let start = d.len();
            for x in 0..w {
                d.push(u8::from(x >= w / 2));
            }
            d.resize(start + row as usize, 0);
        }
        d
    }

    fn run(fake: Fake) -> (Acquired, Fake, PathBuf) {
        let folder = std::env::temp_dir().join(format!(
            "twain-test-{}-{}",
            std::process::id(),
            fake.sources.len() * 10 + usize::from(fake.by_callback)
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).expect("folder");
        FAKE.with(|f| *f.borrow_mut() = fake);
        // The thread's message queue exists before the source posts to it.
        // SAFETY: a valid out pointer; nothing is removed.
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{PeekMessageW, MSG, PM_NOREMOVE};
            let mut m: MSG = std::mem::zeroed();
            PeekMessageW(&mut m, 0, 0, 0, PM_NOREMOVE);
        }
        // SAFETY: the fake answers as a source manager would.
        let got = unsafe { acquire(fake_entry, 0, &folder) };
        let fake = FAKE.with(|f| std::mem::take(&mut *f.borrow_mut()));
        assert!(fake.errors.is_empty(), "{:?}", fake.errors);
        (got, fake, folder)
    }

    fn calls(f: &Fake) -> Vec<(u16, u16)> {
        f.log.iter().copied().filter(|&(d, _)| d != DAT_EVENT).collect()
    }

    /// The structures have the sizes of the specification's, packed on two
    /// bytes, or the manager would read them wrong.
    #[test]
    fn the_structures_have_the_specified_sizes() {
        assert_eq!(std::mem::size_of::<Version>(), 42);
        assert_eq!(std::mem::size_of::<Identity>(), 156);
        assert_eq!(std::mem::size_of::<UserInterface>(), 12);
        assert_eq!(std::mem::size_of::<Event>(), 10);
        assert_eq!(std::mem::size_of::<PendingXfers>(), 6);
        assert_eq!(std::mem::size_of::<Callback>(), 14);
        let me = app_identity();
        assert_eq!(me.name(), "Craft Plumilla");
        assert_eq!({ me.supported_groups } & DF_APP2, DF_APP2);
    }

    /// A packed DIB becomes a BMP file that opens with the same pixels,
    /// colour table included; a broken one is refused.
    #[test]
    fn a_dib_becomes_a_bmp_with_the_same_pixels() {
        let red_blue = dib24(5, 3, |x, _| if x < 2 { [200, 0, 0] } else { [0, 0, 200] });
        let img = image::load_from_memory(&dib_to_bmp(&red_blue).expect("bmp")).expect("opens").to_rgb8();
        assert_eq!((img.width(), img.height()), (5, 3));
        assert_eq!(img.get_pixel(0, 0).0, [200, 0, 0]);
        assert_eq!(img.get_pixel(4, 2).0, [0, 0, 200]);
        let top_bottom = dib24(2, 2, |_, y| if y == 0 { [0, 150, 0] } else { [10, 10, 10] });
        let img = image::load_from_memory(&dib_to_bmp(&top_bottom).expect("bmp")).expect("opens").to_rgb8();
        assert_eq!(img.get_pixel(1, 0).0, [0, 150, 0], "the first row of a bottom-up DIB is the bottom one");
        // Where the pixels start, as other readers take it from the file
        // header: after the 14 + 40 bytes of headers and the colour table.
        let at = |b: &[u8]| (u32::from_le_bytes([b[2], b[3], b[4], b[5]]), u32::from_le_bytes([b[10], b[11], b[12], b[13]]));
        let bmp24 = dib_to_bmp(&red_blue).expect("bmp");
        assert_eq!(at(&bmp24), (bmp24.len() as u32, 54));
        let bmp8 = dib_to_bmp(&dib8(6, 2)).expect("bmp");
        assert_eq!(at(&bmp8), (bmp8.len() as u32, 54 + 2 * 4), "the two-colour table comes before the pixels");
        let paletted = image::load_from_memory(&bmp8).expect("opens").to_rgb8();
        assert_eq!(paletted.get_pixel(0, 1).0, [0, 0, 0]);
        assert_eq!(paletted.get_pixel(5, 0).0, [255, 255, 0]);
        assert!(dib_to_bmp(&[1, 2, 3]).is_none());
        assert!(dib_to_bmp(&[200, 0, 0, 0, 1, 2]).is_none());
    }

    /// One source: no Select Source dialog; it is opened, its dialog shown,
    /// every picture it has ready taken in order, and everything closed in
    /// the order the specification gives.
    #[test]
    fn the_pictures_of_a_source_are_taken_and_everything_closed() {
        let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        let (got, fake, folder) = run(Fake {
            sources: vec![("Test scanner", 0)],
            pictures: vec![dib24(4, 2, |_, _| [220, 30, 30]), dib8(6, 2)],
            says: MSG_XFERREADY,
            ..Fake::default()
        });
        let Acquired::Files(files) = got else { panic!("{got:?}") };
        assert_eq!(files.len(), 2);
        let first = image::open(&files[0]).expect("first").to_rgb8();
        assert_eq!(first.get_pixel(3, 1).0, [220, 30, 30]);
        let second = image::open(&files[1]).expect("second").to_rgb8();
        assert_eq!(second.get_pixel(5, 1).0, [255, 255, 0]);
        assert_eq!(fake.opened.as_deref(), Some("Test scanner"));
        assert_eq!(fake.state, 2, "manager and source closed");
        assert_eq!(
            calls(&fake),
            vec![
                (DAT_PARENT, MSG_OPENDSM),
                (DAT_IDENTITY, MSG_GETFIRST),
                (DAT_IDENTITY, MSG_GETNEXT),
                (DAT_IDENTITY, MSG_OPENDS),
                (DAT_USERINTERFACE, MSG_ENABLEDS),
                (DAT_IMAGENATIVEXFER, MSG_GET),
                (DAT_PENDINGXFERS, MSG_ENDXFER),
                (DAT_IMAGENATIVEXFER, MSG_GET),
                (DAT_PENDINGXFERS, MSG_ENDXFER),
                (DAT_USERINTERFACE, MSG_DISABLEDS),
                (DAT_IDENTITY, MSG_CLOSEDS),
                (DAT_PARENT, MSG_CLOSEDSM),
            ]
        );
        assert!(fake.log.contains(&(DAT_EVENT, MSG_PROCESSEVENT)), "the source saw the window messages");
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// A version 2 source is given the callback and its pictures come in
    /// through it, without window messages.
    #[test]
    fn a_version_2_source_is_heard_through_the_callback() {
        let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        let (got, fake, folder) = run(Fake {
            sources: vec![("Version 2 scanner", DF_DS2)],
            pictures: vec![dib24(3, 3, |_, _| [10, 120, 240])],
            says: MSG_XFERREADY,
            by_callback: true,
            ..Fake::default()
        });
        let Acquired::Files(files) = got else { panic!("{got:?}") };
        assert_eq!(files.len(), 1);
        assert_eq!(image::open(&files[0]).expect("picture").to_rgb8().get_pixel(1, 1).0, [10, 120, 240]);
        assert!(calls(&fake).contains(&(DAT_CALLBACK, MSG_REGISTER_CALLBACK)));
        assert_eq!(fake.state, 2);
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// Closing the source's dialog brings nothing in and still closes it;
    /// no source at all says so without opening anything; with several,
    /// the manager's Select Source dialog chooses, and cancelling it ends.
    #[test]
    fn closing_cancelling_and_no_source() {
        let _one = ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner());
        let (got, fake, _) = run(Fake { sources: vec![("Test scanner", 0)], says: MSG_CLOSEDSREQ, ..Fake::default() });
        assert_eq!(got, Acquired::Cancelled);
        assert_eq!(fake.state, 2);
        assert!(calls(&fake).contains(&(DAT_USERINTERFACE, MSG_DISABLEDS)));

        let (got, fake, _) = run(Fake::default());
        assert_eq!(got, Acquired::NoDevice);
        assert_eq!(calls(&fake), vec![(DAT_PARENT, MSG_OPENDSM), (DAT_IDENTITY, MSG_GETFIRST), (DAT_PARENT, MSG_CLOSEDSM)]);

        let two = vec![("First", 0), ("Second", 0)];
        let (got, fake, _) = run(Fake { sources: two.clone(), user_picks: None, ..Fake::default() });
        assert_eq!(got, Acquired::Cancelled);
        assert!(calls(&fake).contains(&(DAT_IDENTITY, MSG_USERSELECT)));
        assert_eq!(fake.opened, None);
        assert_eq!(fake.state, 2);

        let (got, fake, folder) = run(Fake {
            sources: two,
            user_picks: Some(1),
            pictures: vec![dib24(2, 2, |_, _| [1, 2, 3])],
            says: MSG_XFERREADY,
            ..Fake::default()
        });
        assert!(matches!(got, Acquired::Files(ref f) if f.len() == 1), "{got:?}");
        assert_eq!(fake.opened.as_deref(), Some("Second"), "the source chosen in the dialog is opened");
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// The source manager Windows carries, called for real over a hidden
    /// window: it opens, lists no source where none is installed and
    /// closes, with no dialog (skipped where it is missing or has sources).
    #[test]
    fn the_installed_manager_answers_that_there_is_no_source() {
        use windows_sys::Win32::UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow};
        if dsm_path().is_none() {
            return;
        }
        let windows = PathBuf::from(std::env::var_os("SystemRoot").expect("SystemRoot"));
        let has_sources = std::fs::read_dir(windows.join("twain_64"))
            .map(|d| d.flatten().next().is_some())
            .unwrap_or(false);
        if has_sources {
            return;
        }
        let class: Vec<u16> = "STATIC\0".encode_utf16().collect();
        // SAFETY: a hidden window of this thread, destroyed below.
        let hwnd = unsafe {
            CreateWindowExW(0, class.as_ptr(), class.as_ptr(), 0, 0, 0, 0, 0, 0, 0, 0, std::ptr::null())
        };
        assert_ne!(hwnd, 0);
        let folder = std::env::temp_dir().join(format!("twain-real-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("folder");
        assert_eq!(acquire_installed(hwnd, &folder), Acquired::NoDevice);
        // SAFETY: the window made above.
        unsafe { DestroyWindow(hwnd) };
        let _ = std::fs::remove_dir_all(&folder);
    }
}
