//! Stable C entry point for the small native Windows portable launcher.
//! No Rust objects cross the DLL boundary; the library owns the whole editor.

#[no_mangle]
pub extern "C" fn craft_corel_runtime_abi_v1() -> u32 {
    1
}

#[no_mangle]
pub extern "C" fn craft_corel_run_v1() -> i32 {
    // Keep unwinding inside the DLL. The startup log records application failures.
    match std::panic::catch_unwind(crate::run) {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            eprintln!("Craft Corel: {error}");
            1
        }
        Err(_) => 2,
    }
}
