//! Thumbnails of `.tdraw` drawings in the Windows Explorer: a thumbnail
//! provider (`IThumbnailProvider` with `IInitializeWithStream`) the
//! installer registers for the `.tdraw` extension. Explorer hands it the
//! file as a stream; it draws the drawing's first page on white paper at
//! the size asked and returns it as a 32-bit bitmap with alpha.
//!
//! The picture itself is made by [`thumbnail_rgba`], which every platform
//! builds and tests; only the COM glue is Windows code.

use tracedraw_core::Document;

/// The class id the installer registers (`packaging/windows/tracedraw.iss`)
/// under `.tdraw\ShellEx\{e357fccd-a995-4576-b01f-234630154e96}`.
pub const CLSID_TEXT: &str = "{6B1E5D3A-2F41-4C8E-9B7A-1D2C3E4F5A60}";

/// The first page of the drawing in `bytes` (a `.tdraw` file) drawn with
/// its long side `size` pixels on white paper: width, height and RGBA
/// pixels (straight, not premultiplied). `None` when the file cannot be
/// read or has no page; never panics on what the file holds.
pub fn thumbnail_rgba(bytes: &[u8], size: u32) -> Option<(u32, u32, Vec<u8>)> {
    let text = std::str::from_utf8(bytes).ok()?;
    let doc = Document::from_json(text).ok()?;
    let page = doc.pages.first()?;
    let long = page.size.width.max(page.size.height);
    if !long.is_finite() || long <= 0.0 {
        return None;
    }
    let size = f64::from(size.clamp(16, 2048));
    let dpi = size / long * 25.4;
    let pm = tracedraw_render::render_page_image(&doc, page.id, dpi)?;
    let mut rgba = Vec::with_capacity(pm.data().len());
    for px in pm.pixels() {
        let c = px.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    Some((pm.width(), pm.height(), rgba))
}

#[cfg(windows)]
mod com {
    use std::cell::RefCell;
    use std::ffi::c_void;
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::Once;
    use windows::core::{implement, Interface, Ref, BOOL, GUID, HRESULT};
    use windows::Win32::Foundation::{
        CLASS_E_CLASSNOTAVAILABLE, CLASS_E_NOAGGREGATION, E_FAIL, E_POINTER, S_FALSE, S_OK,
    };
    use windows::Win32::Graphics::Gdi::{
        CreateDIBSection, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HBITMAP,
    };
    use windows::Win32::System::Com::{
        IClassFactory, IClassFactory_Impl, IStream, STATFLAG_NONAME, STATSTG,
    };
    use windows::Win32::UI::Shell::PropertiesSystem::{
        IInitializeWithStream, IInitializeWithStream_Impl,
    };
    use windows::Win32::UI::Shell::{
        IThumbnailProvider, IThumbnailProvider_Impl, WTSAT_ARGB, WTS_ALPHATYPE,
    };

    /// The provider's class id, the same as [`super::CLSID_TEXT`].
    const CLSID: GUID = GUID::from_u128(0x6B1E5D3A_2F41_4C8E_9B7A_1D2C3E4F5A60);

    /// Objects alive, for `DllCanUnloadNow`.
    static LIVE: AtomicI32 = AtomicI32::new(0);
    static FONTS: Once = Once::new();

    /// Files past this size are not drawn (Explorer shows the icon).
    const MAX_BYTES: u64 = 256 * 1024 * 1024;

    #[implement(IThumbnailProvider, IInitializeWithStream)]
    struct Provider {
        data: RefCell<Option<Vec<u8>>>,
    }

    impl Provider {
        fn new() -> Self {
            LIVE.fetch_add(1, Ordering::SeqCst);
            Provider {
                data: RefCell::new(None),
            }
        }
    }

    impl Drop for Provider {
        fn drop(&mut self) {
            LIVE.fetch_sub(1, Ordering::SeqCst);
        }
    }

    impl IInitializeWithStream_Impl for Provider_Impl {
        fn Initialize(
            &self,
            pstream: Ref<'_, IStream>,
            _grfmode: u32,
        ) -> windows::core::Result<()> {
            let stream = pstream.ok()?;
            let mut stat = STATSTG::default();
            unsafe { stream.Stat(&mut stat, STATFLAG_NONAME)? };
            if stat.cbSize > MAX_BYTES {
                return Err(E_FAIL.into());
            }
            let mut out = Vec::with_capacity(stat.cbSize as usize);
            let mut buf = vec![0u8; 64 * 1024];
            loop {
                let mut read = 0u32;
                let hr = unsafe {
                    stream.Read(buf.as_mut_ptr().cast(), buf.len() as u32, Some(&mut read))
                };
                if hr.is_err() {
                    return Err(hr.into());
                }
                if read == 0 {
                    break;
                }
                out.extend_from_slice(&buf[..read as usize]);
                if out.len() as u64 > MAX_BYTES {
                    return Err(E_FAIL.into());
                }
            }
            *self.data.borrow_mut() = Some(out);
            Ok(())
        }
    }

    impl IThumbnailProvider_Impl for Provider_Impl {
        fn GetThumbnail(
            &self,
            cx: u32,
            phbmp: *mut HBITMAP,
            pdwalpha: *mut WTS_ALPHATYPE,
        ) -> windows::core::Result<()> {
            if phbmp.is_null() || pdwalpha.is_null() {
                return Err(E_POINTER.into());
            }
            FONTS.call_once(tracedraw_text::install);
            let data = self.data.borrow();
            let bytes = data.as_deref().ok_or(windows::core::Error::from(E_FAIL))?;
            let (w, h, rgba) = std::panic::catch_unwind(|| super::thumbnail_rgba(bytes, cx))
                .ok()
                .flatten()
                .ok_or(windows::core::Error::from(E_FAIL))?;
            let info = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: w as i32,
                    // Negative: rows top-down, as the pixels are.
                    biHeight: -(h as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            let mut bits: *mut c_void = std::ptr::null_mut();
            let hbmp =
                unsafe { CreateDIBSection(None, &info, DIB_RGB_COLORS, &mut bits, None, 0)? };
            if bits.is_null() {
                return Err(E_FAIL.into());
            }
            // BGRA, premultiplied, as WTSAT_ARGB asks.
            let dst = unsafe { std::slice::from_raw_parts_mut(bits.cast::<u8>(), rgba.len()) };
            for (d, s) in dst.chunks_exact_mut(4).zip(rgba.chunks_exact(4)) {
                let a = u32::from(s[3]);
                let pm = |c: u8| ((u32::from(c) * a + 127) / 255) as u8;
                d[0] = pm(s[2]);
                d[1] = pm(s[1]);
                d[2] = pm(s[0]);
                d[3] = s[3];
            }
            unsafe {
                *phbmp = hbmp;
                *pdwalpha = WTSAT_ARGB;
            }
            Ok(())
        }
    }

    #[implement(IClassFactory)]
    struct Factory;

    impl IClassFactory_Impl for Factory_Impl {
        fn CreateInstance(
            &self,
            punkouter: Ref<'_, windows::core::IUnknown>,
            riid: *const GUID,
            ppvobject: *mut *mut c_void,
        ) -> windows::core::Result<()> {
            if ppvobject.is_null() || riid.is_null() {
                return Err(E_POINTER.into());
            }
            unsafe { *ppvobject = std::ptr::null_mut() };
            if !punkouter.is_null() {
                return Err(CLASS_E_NOAGGREGATION.into());
            }
            let unknown: windows::core::IUnknown = Provider::new().into();
            unsafe { unknown.query(&*riid, ppvobject).ok() }
        }

        fn LockServer(&self, _flock: BOOL) -> windows::core::Result<()> {
            Ok(())
        }
    }

    /// # Safety
    /// Called by COM with valid pointers.
    #[no_mangle]
    pub unsafe extern "system" fn DllGetClassObject(
        rclsid: *const GUID,
        riid: *const GUID,
        ppv: *mut *mut c_void,
    ) -> HRESULT {
        if rclsid.is_null() || riid.is_null() || ppv.is_null() {
            return E_POINTER;
        }
        *ppv = std::ptr::null_mut();
        if *rclsid != CLSID {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let factory: IClassFactory = Factory.into();
        factory.query(&*riid, ppv)
    }

    #[no_mangle]
    pub extern "system" fn DllCanUnloadNow() -> HRESULT {
        if LIVE.load(Ordering::SeqCst) == 0 {
            S_OK
        } else {
            S_FALSE
        }
    }

    #[allow(dead_code)]
    fn _assert_interfaces() {
        let _ = IThumbnailProvider::IID;
        let _ = IInitializeWithStream::IID;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sample drawing gives a picture with its long side as asked,
    /// white where the page is empty; anything else gives none.
    #[test]
    fn the_first_page_is_drawn_at_the_size_asked() {
        let bytes = std::fs::read(concat!(env!("CARGO_MANIFEST_DIR"), "/../../sample.tdraw"))
            .expect("sample");
        let (w, h, rgba) = thumbnail_rgba(&bytes, 256).expect("drawn");
        assert_eq!(w.max(h), 256);
        assert_eq!(rgba.len(), (w * h * 4) as usize);
        assert_eq!(&rgba[..4], &[255, 255, 255, 255]);
        assert!(thumbnail_rgba(b"not a drawing", 256).is_none());
        assert!(thumbnail_rgba(&[0xff, 0xfe, 0x00], 256).is_none());
    }
}
