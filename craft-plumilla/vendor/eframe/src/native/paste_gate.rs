// TraceDraw: the application may stop egui-winit reading the clipboard's
// text for a paste shortcut (see vendor/README.md, #1488). Kept apart so
// that TraceDraw's own tests compile and run it
// (`clipboard::eframe_paste_gate::tests`): this copy is outside the
// workspace and its own tests run in no gate.
//
// egui-winit reads the clipboard as text on every paste shortcut, before
// the application sees the key. An X11 clipboard owner that holds only a
// picture may answer that text request with its picture in parts (INCR);
// the clipboard library stops after the first part, and the owner then
// waits for the next request of that transfer forever and answers nobody
// else, so the application's own picture read timed out and the window
// froze for the waits.

use std::sync::OnceLock;

/// The application's answer to "may the clipboard's text be read now?".
static GATE: OnceLock<fn() -> bool> = OnceLock::new();

/// Ask `gate` before each paste shortcut whether egui-winit may read the
/// clipboard's text. When it answers `false` the key press is not handed
/// to egui-winit, which then neither reads the clipboard nor sends a
/// paste event, as when the clipboard holds no text: the application
/// sees the key's release and pastes by itself. Set once; later calls
/// are ignored.
#[allow(dead_code)]
pub fn set_paste_text_gate(gate: fn() -> bool) {
    let _ = GATE.set(gate);
}

/// Whether a key press is a paste shortcut as egui-winit tells it: the
/// logical key, or the physical one when the logical key is no key egui
/// knows (layouts without Latin letters), is V with Ctrl (Cmd), or the
/// Paste key. Shift+Insert counts on Windows only, as in egui-winit.
pub(crate) fn is_paste_key(
    logical: &winit::keyboard::Key,
    physical: winit::keyboard::PhysicalKey,
    command: bool,
    shift: bool,
) -> bool {
    use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
    let from_logical = match logical {
        Key::Named(NamedKey::Paste) => Some(egui::Key::Paste),
        Key::Named(NamedKey::Insert) => Some(egui::Key::Insert),
        // Other named keys are no paste key; egui knows nearly all of them.
        Key::Named(_) => return false,
        Key::Character(s) => egui::Key::from_name(s.as_str()),
        Key::Unidentified(_) | Key::Dead(_) => None,
    };
    let key = from_logical.or(match physical {
        PhysicalKey::Code(KeyCode::KeyV) => Some(egui::Key::V),
        PhysicalKey::Code(KeyCode::Paste) => Some(egui::Key::Paste),
        PhysicalKey::Code(KeyCode::Insert) => Some(egui::Key::Insert),
        _ => None,
    });
    match key {
        Some(egui::Key::Paste) => true,
        Some(egui::Key::V) => command,
        Some(egui::Key::Insert) => cfg!(target_os = "windows") && shift,
        _ => false,
    }
}

thread_local! {
    /// The modifier keys held, as the window's events last gave them.
    static MODIFIERS: std::cell::Cell<winit::keyboard::ModifiersState> =
        const { std::cell::Cell::new(winit::keyboard::ModifiersState::empty()) };
}

/// Whether `event` is a real paste key press that the application's gate
/// keeps from egui-winit. Every window event goes through here, so the
/// modifier keys are followed as egui-winit follows them.
#[allow(dead_code)]
pub(crate) fn blocks(event: &winit::event::WindowEvent) -> bool {
    use winit::event::WindowEvent;
    let event = match event {
        WindowEvent::ModifiersChanged(m) => {
            MODIFIERS.with(|cell| cell.set(m.state()));
            return false;
        }
        WindowEvent::KeyboardInput {
            event,
            is_synthetic: false,
            ..
        } => event,
        _ => return false,
    };
    if event.state != winit::event::ElementState::Pressed || GATE.get().is_none() {
        return false;
    }
    let held = MODIFIERS.with(|cell| cell.get());
    // Cmd on macOS, Ctrl elsewhere, as egui's `command`.
    let command = if cfg!(target_os = "macos") {
        held.super_key()
    } else {
        held.control_key()
    };
    is_paste_key(
        &event.logical_key,
        event.physical_key,
        command,
        held.shift_key(),
    ) && GATE.get().is_some_and(|may_read| !may_read())
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::keyboard::{Key, KeyCode, NamedKey, NativeKeyCode, PhysicalKey, SmolStr};

    fn ch(s: &str) -> Key {
        Key::Character(SmolStr::new(s))
    }

    #[test]
    fn paste_keys_are_told_as_egui_winit_tells_them() {
        let v = PhysicalKey::Code(KeyCode::KeyV);
        assert!(is_paste_key(&ch("v"), v, true, false));
        assert!(is_paste_key(&ch("V"), v, true, true));
        // V without Ctrl is typing.
        assert!(!is_paste_key(&ch("v"), v, false, false));
        // The Paste key, with or without modifiers.
        let paste = PhysicalKey::Code(KeyCode::Paste);
        assert!(is_paste_key(
            &Key::Named(NamedKey::Paste),
            paste,
            false,
            false
        ));
        // A layout without Latin letters: the physical V key decides.
        assert!(is_paste_key(&ch("м"), v, true, false));
        assert!(!is_paste_key(&ch("м"), v, false, false));
        // A Latin layout where V sits elsewhere: the logical key decides.
        let b = PhysicalKey::Code(KeyCode::KeyB);
        assert!(is_paste_key(&ch("v"), b, true, false));
        assert!(!is_paste_key(&ch("b"), v, true, false));
        // Other keys and named keys never.
        assert!(!is_paste_key(
            &ch("c"),
            PhysicalKey::Code(KeyCode::KeyC),
            true,
            false
        ));
        assert!(!is_paste_key(&Key::Named(NamedKey::Enter), v, true, false));
        let unknown = PhysicalKey::Unidentified(NativeKeyCode::Xkb(0));
        assert!(!is_paste_key(&ch("x"), unknown, true, false));
        // Shift+Insert pastes on Windows only.
        let insert = PhysicalKey::Code(KeyCode::Insert);
        assert_eq!(
            is_paste_key(&Key::Named(NamedKey::Insert), insert, false, true),
            cfg!(target_os = "windows")
        );
        assert!(!is_paste_key(
            &Key::Named(NamedKey::Insert),
            insert,
            false,
            false
        ));
    }
}
