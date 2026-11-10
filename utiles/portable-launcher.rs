//! Native Windows launcher. The editor lives in a sibling DLL, loaded by its
//! absolute path with a restricted dependency search; no scripts or extraction.
#![windows_subsystem = "windows"]

use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::process::ExitCode;

#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryExW(path: *const u16, file: *mut c_void, flags: u32) -> *mut c_void;
    fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
}

#[link(name = "user32")]
extern "system" {
    fn MessageBoxW(window: *mut c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
}

fn wide(text: &OsStr) -> Vec<u16> {
    text.encode_wide().chain(Some(0)).collect()
}

fn launch(verify_only: bool) -> Result<i32, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let folder = exe
        .parent()
        .ok_or("No se encuentra la carpeta de la aplicación.")?;
    let dll = folder.join("craft-plumilla-engine.dll");
    let dll_path = wide(dll.as_os_str());
    // LOAD_LIBRARY_SEARCH_DLL_LOAD_DIR | LOAD_LIBRARY_SEARCH_SYSTEM32.
    // Dependencies can come only from this DLL's folder and Windows system32.
    let module = unsafe { LoadLibraryExW(dll_path.as_ptr(), std::ptr::null_mut(), 0x100 | 0x800) };
    if module.is_null() {
        return Err(format!(
            "No se pudo cargar {}.\n{}\nExtrae todos los archivos del ZIP en la misma carpeta.",
            dll.display(),
            std::io::Error::last_os_error()
        ));
    }
    let version = unsafe { GetProcAddress(module, c"craft_plumilla_runtime_abi_v1".as_ptr().cast()) };
    let entry = unsafe { GetProcAddress(module, c"craft_plumilla_run_v1".as_ptr().cast()) };
    if version.is_null() || entry.is_null() {
        return Err("La DLL no corresponde a esta versión de Craft Plumilla.".into());
    }
    let abi: unsafe extern "C" fn() -> u32 = unsafe { std::mem::transmute(version) };
    if unsafe { abi() } != 1 {
        return Err("La versión del motor no es compatible con este ejecutable.".into());
    }
    if verify_only {
        println!("Craft Plumilla: motor DLL cargado; ABI 1 verificada.");
        return Ok(0);
    }
    let run: unsafe extern "C" fn() -> i32 = unsafe { std::mem::transmute(entry) };
    // Retain the DLL until process exit: editor workers may still be finishing.
    Ok(unsafe { run() })
}

fn main() -> ExitCode {
    let verify_only = std::env::args_os().any(|arg| arg == "--verify-runtime");
    let headless = std::env::args_os().any(|arg| is_headless_argument(&arg));
    let result = launch(verify_only).and_then(|code| {
        if code == 0 { Ok(()) } else { Err(format!("La aplicación terminó con el código {code}. Consulta el registro de la carpeta craft-plumilla-settings.")) }
    });
    if let Err(message) = result {
        eprintln!("{message}");
        if !headless {
            let text = wide(OsStr::new(&message));
            let title = wide(OsStr::new("Craft Plumilla"));
            unsafe {
                MessageBoxW(std::ptr::null_mut(), text.as_ptr(), title.as_ptr(), 0x10);
            }
        }
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn is_headless_argument(argument: &OsStr) -> bool {
    ["--verify-runtime", "--mcp", "--mcp-window", "--profile-export", "--profile-cdr-open"]
        .iter().any(|flag| argument == OsStr::new(flag))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_windowless_modes_suppress_native_error_dialogs() {
        for flag in ["--verify-runtime", "--mcp", "--mcp-window", "--profile-export", "--profile-cdr-open"] {
            assert!(is_headless_argument(OsStr::new(flag)), "{flag}");
        }
        assert!(!is_headless_argument(OsStr::new("drawing.cdr")));
        assert!(!is_headless_argument(OsStr::new("--no-splash")));
    }
}
