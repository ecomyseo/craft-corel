//! TraceDraw: two decisions of the Windows backend that keep the pointer
//! where it is when a shortcut with Alt is pressed (#1922, see
//! `vendor/README.md`). The file has no dependency on the rest of the
//! crate, so that TraceDraw's tests can include and run it.
//!
//! A shortcut with Alt over a still mouse left the pointer as it was (the
//! arrow) or invisible until the mouse moved: for a moment the system
//! showed its arrow, then the window was told the pointer had left it
//! (`WM_MOUSELEAVE`) although it had not moved. The window then stopped
//! applying pointers until the next move. Both are the marks of the
//! system's keyboard menu mode, which `WM_SYSCOMMAND` with `SC_KEYMENU`
//! starts and which takes the mouse while it runs.

/// `WM_SYSCOMMAND`'s command for the window menu asked for from the
/// keyboard (Alt with a key, Alt alone or F10).
pub(crate) const SC_KEYMENU: usize = 0xF100;

/// Whether a `WM_SYSCOMMAND` with `wparam` and `lparam` goes on to the
/// system's default handling. The four low bits of `wparam` are the
/// system's own and are left out of the command. `SC_KEYMENU` carries in
/// `lparam` the character typed with Alt: in a window without a native
/// menu bar (`has_menu_bar` false) no menu answers a letter, so the menu
/// mode would only take the mouse; the program's own Alt shortcuts have
/// already had the key. Alt+Space (the window menu, to move or size the
/// window from the keyboard) and a menu asked for without a character
/// still go on, and so does every other command.
pub(crate) fn system_command_passes(wparam: usize, lparam: isize, has_menu_bar: bool) -> bool {
    if wparam & 0xFFF0 != SC_KEYMENU || has_menu_bar {
        return true;
    }
    lparam == 0 || lparam == b' ' as isize
}

/// Whether a `WM_MOUSELEAVE` means the pointer has left the window: the
/// pointer at `pointer` (client coordinates, `None` when it could not be
/// read) is outside the client area `client` (left, top, right, bottom,
/// the right and bottom edges outside), or another window is the one
/// under it (`this_window_under` false). The system also ends the leave
/// tracking when a menu mode or another window takes the mouse with the
/// pointer standing still in the window; that is no leave, and telling
/// the program so would stop its pointer updates until the next move.
pub(crate) fn pointer_left(
    pointer: Option<(i32, i32)>,
    client: (i32, i32, i32, i32),
    this_window_under: bool,
) -> bool {
    let Some((x, y)) = pointer else {
        return true;
    };
    let (left, top, right, bottom) = client;
    let inside = (left..right).contains(&x) && (top..bottom).contains(&y);
    !(inside && this_window_under)
}

#[cfg(test)]
mod tests {
    use super::{pointer_left, system_command_passes, SC_KEYMENU};

    const SC_CLOSE: usize = 0xF060;
    const SC_MOVE: usize = 0xF010;

    /// Alt with a letter starts no menu mode in a window without a menu
    /// bar (#1922); Alt+Space, a menu asked for without a character, a
    /// window with a menu bar and the other commands are left to the
    /// system.
    #[test]
    fn alt_with_a_letter_starts_no_keyboard_menu_mode() {
        let r = b'r' as isize;
        assert!(!system_command_passes(SC_KEYMENU, r, false));
        assert!(!system_command_passes(SC_KEYMENU, b'E' as isize, false));
        assert!(!system_command_passes(SC_KEYMENU, b'7' as isize, false));
        // The system's own low bits do not hide the command.
        assert!(!system_command_passes(SC_KEYMENU | 0x3, r, false));
        // The window menu stays on Alt+Space.
        assert!(system_command_passes(SC_KEYMENU, b' ' as isize, false));
        assert!(system_command_passes(SC_KEYMENU, 0, false));
        // A native menu bar answers its own letters.
        assert!(system_command_passes(SC_KEYMENU, r, true));
        // Every other command, Alt+F4 among them, goes on as before.
        assert!(system_command_passes(SC_CLOSE, 0, false));
        assert!(system_command_passes(SC_MOVE | 0x2, r, false));
    }

    /// A leave with the pointer still in the client area and this window
    /// under it is no leave; a pointer outside, over another window, or
    /// that could not be read has left.
    #[test]
    fn a_leave_with_the_pointer_still_inside_is_no_leave() {
        let client = (0, 0, 1200, 800);
        assert!(!pointer_left(Some((600, 400)), client, true));
        assert!(!pointer_left(Some((0, 0)), client, true));
        assert!(!pointer_left(Some((1199, 799)), client, true));
        // The right and bottom edges are outside.
        assert!(pointer_left(Some((1200, 400)), client, true));
        assert!(pointer_left(Some((600, 800)), client, true));
        assert!(pointer_left(Some((-1, 400)), client, true));
        // Another window over this one, under a still pointer.
        assert!(pointer_left(Some((600, 400)), client, false));
        assert!(pointer_left(None, client, true));
    }
}
