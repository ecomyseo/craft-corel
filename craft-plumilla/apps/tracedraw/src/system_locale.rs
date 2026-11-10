//! The system's interface languages and the language the Windows
//! installer recorded, read where Craft Plumilla starts (see
//! `docs/behavior/language.md`). The readers are small and platform
//! specific; what they read is turned into names by the pure parsers
//! below, and matched to a supported language by `i18n::best_language`.

/// The registry key and value the Windows installer writes its language
/// choice to (under HKEY_CURRENT_USER for an install for the user alone,
/// HKEY_LOCAL_MACHINE for one for all users).
#[cfg_attr(not(windows), allow(dead_code))]
pub const INSTALLER_KEY: &str = r"Software\Craft Plumilla";
#[cfg_attr(not(windows), allow(dead_code))]
pub const INSTALLER_VALUE: &str = "InstallerLanguage";

/// The locale names of a POSIX environment in order of preference:
/// the `LANGUAGE` list (colon separated, as gettext reads it), then the
/// first set of `LC_ALL`, `LC_MESSAGES` and `LANG`. `get` reads one
/// variable (the environment, or a table in tests).
pub fn posix_languages(get: impl Fn(&str) -> Option<String>) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let set = |name: &str| get(name).filter(|v| !v.trim().is_empty());
    let effective = set("LC_ALL")
        .or_else(|| set("LC_MESSAGES"))
        .or_else(|| set("LANG"));
    // gettext leaves `LANGUAGE` aside when the locale is plain C.
    let plain_c = effective
        .as_deref()
        .is_some_and(|v| matches!(v.trim(), "C" | "POSIX"));
    if !plain_c {
        if let Some(list) = set("LANGUAGE") {
            out.extend(
                list.split(':')
                    .map(str::trim)
                    .filter(|l| !l.is_empty())
                    .map(String::from),
            );
        }
    }
    out.extend(effective);
    out
}

/// The strings of a Windows multi-string (each ends in a zero, the list
/// in a second zero), as `GetUserPreferredUILanguages` fills it.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn parse_multi_sz(buf: &[u16]) -> Vec<String> {
    buf.split(|c| *c == 0)
        .take_while(|s| !s.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}

/// The interface languages the system prefers, most preferred first, as
/// locale names (`pt-PT`, `zh-Hant-TW`, `de_DE.UTF-8`). Empty when the
/// system says nothing.
pub fn preferred_languages() -> Vec<String> {
    #[cfg(windows)]
    {
        let list = windows::preferred_ui_languages();
        if !list.is_empty() {
            return list;
        }
    }
    #[cfg(target_os = "macos")]
    {
        let list = macos::preferred_languages();
        if !list.is_empty() {
            return list;
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        browser_languages()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        posix_languages(|name| std::env::var(name).ok())
    }
}

/// `navigator.languages` (then `navigator.language`) of the browser.
#[cfg(target_arch = "wasm32")]
fn browser_languages() -> Vec<String> {
    let Some(navigator) = web_sys::window().map(|w| w.navigator()) else {
        return Vec::new();
    };
    let mut out: Vec<String> = navigator
        .languages()
        .iter()
        .filter_map(|v| v.as_string())
        .collect();
    if out.is_empty() {
        out.extend(navigator.language());
    }
    out
}

/// What the Windows installer recorded (`system` or one of our codes),
/// for the copy it installed: not for a portable copy (a settings folder
/// next to the program) or the Microsoft Store's, which no installer of
/// ours set up. `None` everywhere else.
pub fn installer_language() -> Option<String> {
    #[cfg(windows)]
    {
        let portable = std::env::current_exe()
            .ok()
            .and_then(|e| crate::settings::portable_dir(&e))
            .is_some();
        if portable || crate::updates::store_install() {
            return None;
        }
        windows::installer_language()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
mod windows {
    use std::ffi::c_void;

    #[link(name = "kernel32", kind = "raw-dylib")]
    unsafe extern "system" {
        fn GetUserPreferredUILanguages(
            flags: u32,
            count: *mut u32,
            buffer: *mut u16,
            length: *mut u32,
        ) -> i32;
        fn GetUserDefaultUILanguage() -> u16;
    }

    #[link(name = "advapi32", kind = "raw-dylib")]
    unsafe extern "system" {
        fn RegGetValueW(
            key: *mut c_void,
            subkey: *const u16,
            value: *const u16,
            flags: u32,
            kind: *mut u32,
            data: *mut c_void,
            length: *mut u32,
        ) -> i32;
    }

    const MUI_LANGUAGE_NAME: u32 = 0x8;
    const RRF_RT_REG_SZ: u32 = 0x2;
    const HKEY_CURRENT_USER: u32 = 0x8000_0001;
    const HKEY_LOCAL_MACHINE: u32 = 0x8000_0002;

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// The user's preferred interface languages (`de-DE`, `pt-PT`...),
    /// or the default UI language's name when the list is not given.
    pub fn preferred_ui_languages() -> Vec<String> {
        let (mut count, mut length) = (0u32, 0u32);
        // SAFETY: with no buffer the call only reports the length needed.
        let ok = unsafe {
            GetUserPreferredUILanguages(
                MUI_LANGUAGE_NAME,
                &mut count,
                std::ptr::null_mut(),
                &mut length,
            )
        };
        if ok != 0 && length > 0 && length < 4096 {
            let mut buf = vec![0u16; length as usize];
            // SAFETY: `buf` holds the `length` characters just reported.
            let ok = unsafe {
                GetUserPreferredUILanguages(
                    MUI_LANGUAGE_NAME,
                    &mut count,
                    buf.as_mut_ptr(),
                    &mut length,
                )
            };
            if ok != 0 {
                let list = super::parse_multi_sz(&buf);
                if !list.is_empty() {
                    return list;
                }
            }
        }
        // SAFETY: takes no arguments.
        let id = unsafe { GetUserDefaultUILanguage() };
        crate::i18n::language_for_langid(id)
            .map(|c| vec![c.to_string()])
            .unwrap_or_default()
    }

    fn read(root: u32, key: &str, value: &str) -> Option<String> {
        let (key, value) = (wide(key), wide(value));
        // A predefined key handle is the sign-extended 32-bit constant.
        let root = root as i32 as isize as *mut c_void;
        let mut buf = vec![0u16; 64];
        let mut length = (buf.len() * 2) as u32;
        // SAFETY: the strings end in a zero; `buf` holds `length` bytes.
        let status = unsafe {
            RegGetValueW(
                root,
                key.as_ptr(),
                value.as_ptr(),
                RRF_RT_REG_SZ,
                std::ptr::null_mut(),
                buf.as_mut_ptr().cast(),
                &mut length,
            )
        };
        if status != 0 {
            return None;
        }
        let chars = (length as usize / 2).min(buf.len());
        super::parse_multi_sz(buf.get(..chars)?).into_iter().next()
    }

    /// The installer's value: the user's own install first, then the one
    /// for all users.
    pub fn installer_language() -> Option<String> {
        [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE]
            .into_iter()
            .find_map(|root| read(root, super::INSTALLER_KEY, super::INSTALLER_VALUE))
            .filter(|v| !v.trim().is_empty())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::ffi::{c_char, c_void};

    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFLocaleCopyPreferredLanguages() -> *const c_void;
        fn CFArrayGetCount(array: *const c_void) -> isize;
        fn CFArrayGetValueAtIndex(array: *const c_void, index: isize) -> *const c_void;
        fn CFStringGetCString(
            string: *const c_void,
            buffer: *mut c_char,
            size: isize,
            encoding: u32,
        ) -> u8;
        fn CFRelease(object: *const c_void);
    }

    const UTF8: u32 = 0x0800_0100;

    /// The languages of System Settings > General > Language & Region,
    /// in their order (`pt-PT`, `zh-Hant-TW`...).
    pub fn preferred_languages() -> Vec<String> {
        // SAFETY: returns a new array (or null) that is released below.
        let array = unsafe { CFLocaleCopyPreferredLanguages() };
        if array.is_null() {
            return Vec::new();
        }
        let mut out = Vec::new();
        // SAFETY: `array` is a CFArray of CFStrings until released.
        let count = unsafe { CFArrayGetCount(array) }.clamp(0, 64);
        for i in 0..count {
            let mut buf = [0 as c_char; 64];
            // SAFETY: `i` is in range; `buf` holds 64 bytes.
            let ok = unsafe {
                let s = CFArrayGetValueAtIndex(array, i);
                !s.is_null() && CFStringGetCString(s, buf.as_mut_ptr(), 64, UTF8) != 0
            };
            if ok {
                // SAFETY: CFStringGetCString wrote a string ending in zero.
                let s = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
                out.push(s.to_string_lossy().into_owned());
            }
        }
        // SAFETY: the array was copied for us.
        unsafe { CFRelease(array) };
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::i18n::best_language;

    fn env<'a>(vars: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |name| {
            vars.iter()
                .find(|(n, _)| *n == name)
                .map(|(_, v)| v.to_string())
        }
    }

    #[test]
    fn posix_variables_are_read_in_gettext_order() {
        let vars = [
            ("LANGUAGE", "sv:pt_PT:de"),
            ("LC_MESSAGES", "fr_FR.UTF-8"),
            ("LANG", "it_IT.UTF-8"),
        ];
        let list = posix_languages(env(&vars));
        assert_eq!(list, ["sv", "pt_PT", "de", "fr_FR.UTF-8"]);
        // Swedish is not shipped: the next one, Portuguese, is.
        assert_eq!(best_language(&list), Some("pt-BR"));
        // LC_ALL wins over LC_MESSAGES and LANG.
        let vars = [("LC_ALL", "ja_JP.UTF-8"), ("LANG", "de_DE.UTF-8")];
        assert_eq!(posix_languages(env(&vars)), ["ja_JP.UTF-8"]);
        // A C locale leaves LANGUAGE aside, and matches nothing.
        let vars = [("LANGUAGE", "de"), ("LANG", "C")];
        assert_eq!(best_language(&posix_languages(env(&vars))), None);
        // Nothing set at all.
        assert!(posix_languages(env(&[])).is_empty());
        // Empty variables count as unset.
        let vars = [("LC_ALL", ""), ("LANG", "nl_BE.UTF-8")];
        assert_eq!(posix_languages(env(&vars)), ["nl_BE.UTF-8"]);
    }

    #[test]
    fn windows_multi_strings_split_into_names() {
        let mut buf: Vec<u16> = "pt-PT\0en-US\0\0".encode_utf16().collect();
        assert_eq!(parse_multi_sz(&buf), ["pt-PT", "en-US"]);
        buf.extend([0, 0]);
        assert_eq!(parse_multi_sz(&buf), ["pt-PT", "en-US"]);
        assert!(parse_multi_sz(&[]).is_empty());
        assert!(parse_multi_sz(&[0, 0]).is_empty());
        // A registry string without its final zero.
        let reg: Vec<u16> = "zh-TW".encode_utf16().collect();
        assert_eq!(parse_multi_sz(&reg), ["zh-TW"]);
    }

    #[test]
    fn no_installer_language_off_windows() {
        #[cfg(not(windows))]
        assert_eq!(installer_language(), None);
    }
}
