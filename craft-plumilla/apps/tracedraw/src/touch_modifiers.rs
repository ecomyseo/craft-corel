//! Shift, Ctrl and Alt for a finger (#1499). A touch screen has no keys
//! to hold, so a finger could not add an object or a node to the
//! selection, keep a drag's angle or proportions, pick inside a group or
//! the object behind. The Standard toolbar shows a Shift, a Ctrl and an
//! Alt button where there is a touch screen (Android, or once a finger
//! has touched the window): a tap holds the key down until the next tap
//! on it, the button drawn pressed meanwhile. The keys held this way join
//! every frame's input (the app's raw input hook), so every tool reads
//! them as it reads the keyboard's. See docs/behavior/android.md.

use crate::app::App;

/// The keys held down from the toolbar.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TouchModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

/// One of the three keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModKey {
    Shift,
    Ctrl,
    Alt,
}

impl ModKey {
    #[cfg(test)]
    pub const ALL: [ModKey; 3] = [ModKey::Shift, ModKey::Ctrl, ModKey::Alt];

    /// The key of a Standard toolbar button's id.
    pub fn of_item(id: &str) -> Option<ModKey> {
        Some(match id {
            "shift" => ModKey::Shift,
            "ctrl" => ModKey::Ctrl,
            "alt" => ModKey::Alt,
            _ => return None,
        })
    }

    /// The button's id on the Standard toolbar.
    #[cfg(test)]
    pub fn item(self) -> &'static str {
        match self {
            ModKey::Shift => "shift",
            ModKey::Ctrl => "ctrl",
            ModKey::Alt => "alt",
        }
    }

    /// What the button reads: the key's name, as on a keyboard (key names
    /// are not translated, `docs/i18n.md`).
    pub fn label(self) -> &'static str {
        match self {
            ModKey::Shift => "Shift",
            ModKey::Ctrl => "Ctrl",
            ModKey::Alt => "Alt",
        }
    }

    /// The i18n key of the button's name (its tip is `<key>_desc`).
    pub fn name_key(self) -> &'static str {
        match self {
            ModKey::Shift => "toolbar.mod_shift",
            ModKey::Ctrl => "toolbar.mod_ctrl",
            ModKey::Alt => "toolbar.mod_alt",
        }
    }
}

impl TouchModifiers {
    /// Whether `key` is held.
    pub fn held(self, key: ModKey) -> bool {
        match key {
            ModKey::Shift => self.shift,
            ModKey::Ctrl => self.ctrl,
            ModKey::Alt => self.alt,
        }
    }

    /// A tap on `key`'s button: held down, or let go.
    pub fn toggle(&mut self, key: ModKey) {
        match key {
            ModKey::Shift => self.shift = !self.shift,
            ModKey::Ctrl => self.ctrl = !self.ctrl,
            ModKey::Alt => self.alt = !self.alt,
        }
    }

    /// Whether any key is held.
    #[cfg(test)]
    pub fn any(self) -> bool {
        self.shift || self.ctrl || self.alt
    }

    /// The keys as egui's modifiers. Ctrl is also the command key, as
    /// the Ctrl key is everywhere but on a Mac.
    pub fn modifiers(self) -> egui::Modifiers {
        egui::Modifiers {
            alt: self.alt,
            ctrl: self.ctrl,
            shift: self.shift,
            mac_cmd: false,
            command: self.ctrl,
        }
    }

    /// Add the held keys to a frame's input (the app's raw input hook):
    /// to the keys the frame holds, which egui takes from the frame's
    /// `ModifiersChanged` events, and to those of each press, release and
    /// key event in it. The keyboard's own keys are kept between frames
    /// (in the context), so that letting a key go on the toolbar gives
    /// back what the keyboard holds.
    pub fn apply(self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        let id = egui::Id::new("tracedraw-touch-modifiers");
        let (mut keyboard, applied) = ctx.data(|d| {
            d.get_temp::<(egui::Modifiers, TouchModifiers)>(id)
                .unwrap_or_default()
        });
        let held = self.modifiers();
        let mut told = false;
        for e in &mut raw.events {
            match e {
                egui::Event::ModifiersChanged(m) => {
                    keyboard = *m;
                    *m = keyboard | held;
                    told = true;
                }
                egui::Event::PointerButton { modifiers, .. }
                | egui::Event::Key { modifiers, .. }
                | egui::Event::MouseWheel { modifiers, .. } => *modifiers |= held,
                _ => {}
            }
        }
        if !told && applied != self {
            raw.events
                .insert(0, egui::Event::ModifiersChanged(keyboard | held));
        }
        ctx.data_mut(|d| d.insert_temp(id, (keyboard, self)));
    }
}

/// Whether the buttons are shown: on Android, and wherever a finger has
/// touched the window (a touch screen laptop, a tablet in a browser).
pub fn offered(ctx: &egui::Context) -> bool {
    crate::files::ANDROID || ctx.input(|i| i.has_touch_screen())
}

/// Whether Alt is held from the toolbar, not on the keyboard: then it
/// does not put the bars in their customize mode (Alt held moves their
/// buttons), which would leave the Alt button unable to let go.
pub fn alt_from_toolbar(app: &App) -> bool {
    app.touch_mods.alt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::PointerButton {
            pos: egui::pos2(1.0, 2.0),
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers,
        }
    }

    /// A tap holds the key, the next one lets it go.
    #[test]
    fn a_tap_holds_and_the_next_lets_go() {
        let mut m = TouchModifiers::default();
        assert!(!m.any());
        for key in ModKey::ALL {
            m.toggle(key);
            assert!(m.held(key));
            assert_eq!(ModKey::of_item(key.item()), Some(key));
        }
        assert!(m.shift && m.ctrl && m.alt);
        m.toggle(ModKey::Ctrl);
        assert!(m.shift && !m.ctrl && m.alt);
        assert_eq!(ModKey::of_item("delete"), None);
    }

    /// The keys egui holds after a frame given `events` through `apply`.
    fn held_after(
        ctx: &egui::Context,
        m: TouchModifiers,
        events: Vec<egui::Event>,
    ) -> (egui::Modifiers, Vec<egui::Event>) {
        let mut raw = egui::RawInput {
            events,
            ..Default::default()
        };
        m.apply(ctx, &mut raw);
        let events = raw.events.clone();
        let mut out = ctx.run_ui(raw, |_| {});
        out.textures_delta.clear();
        (ctx.input(|i| i.modifiers), events)
    }

    /// The held keys join the frame's keys and every press in it; the
    /// keyboard's own keys stay, and letting go gives them back.
    #[test]
    fn held_keys_join_the_frames_input() {
        let ctx = egui::Context::default();
        let none = TouchModifiers::default();
        // Nothing held: the input is left as it came.
        let events = vec![
            egui::Event::ModifiersChanged(egui::Modifiers::ALT),
            press(egui::Modifiers::ALT),
        ];
        let (mods, out) = held_after(&ctx, none, events.clone());
        assert_eq!(out, events);
        assert_eq!(mods, egui::Modifiers::ALT);

        let m = TouchModifiers {
            shift: true,
            ctrl: true,
            alt: false,
        };
        let (mods, out) = held_after(
            &ctx,
            m,
            vec![
                press(egui::Modifiers::NONE),
                egui::Event::PointerMoved(egui::pos2(3.0, 4.0)),
            ],
        );
        assert!(mods.shift && mods.ctrl && mods.command, "{mods:?}");
        assert!(mods.alt, "the keyboard's Alt stays");
        match &out[1] {
            egui::Event::PointerButton { modifiers, .. } => {
                assert!(modifiers.shift && modifiers.ctrl && !modifiers.alt);
            }
            e => panic!("{e:?}"),
        }
        // Still held on a frame with no events.
        let (mods, _) = held_after(&ctx, m, Vec::new());
        assert!(mods.shift && mods.ctrl);
        // The keyboard lets Alt go while Shift is held from the toolbar.
        let (mods, _) = held_after(
            &ctx,
            m,
            vec![egui::Event::ModifiersChanged(egui::Modifiers::NONE)],
        );
        assert!(mods.shift && !mods.alt);
        // Let go on the toolbar: nothing is held.
        let (mods, _) = held_after(&ctx, none, Vec::new());
        assert_eq!(mods, egui::Modifiers::NONE);
    }
}
