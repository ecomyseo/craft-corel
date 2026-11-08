//! Keyboard shortcuts as text ("Ctrl+Shift+E", "F5", "Alt+Enter"), the
//! way the menus show them and the preferences keep them
//! (`Settings::shortcuts`: a tool id or a menu item's key, and its
//! shortcut). Menu commands with a shortcut of the user's run through
//! `ui::menus::run_command`.

use egui::{Key, Modifiers};

/// The name a key goes by in shortcut text.
pub fn key_name(key: Key) -> String {
    match key {
        Key::PageUp => "PgUp".into(),
        Key::PageDown => "PgDn".into(),
        Key::Plus => "+".into(),
        Key::Minus => "-".into(),
        Key::Equals => "=".into(),
        Key::Escape => "Esc".into(),
        Key::ArrowLeft => "Left".into(),
        Key::ArrowRight => "Right".into(),
        Key::ArrowUp => "Up".into(),
        Key::ArrowDown => "Down".into(),
        other => other.name().to_string(),
    }
}

fn key_from_name(name: &str) -> Option<Key> {
    Some(match name {
        "PgUp" => Key::PageUp,
        "PgDn" => Key::PageDown,
        "+" => Key::Plus,
        "-" => Key::Minus,
        "=" => Key::Equals,
        "Esc" => Key::Escape,
        "Left" => Key::ArrowLeft,
        "Right" => Key::ArrowRight,
        "Up" => Key::ArrowUp,
        "Down" => Key::ArrowDown,
        other => {
            return Key::from_name(other).or_else(|| Key::from_name(&other.to_ascii_uppercase()))
        }
    })
}

/// Shortcut text: Ctrl, Alt and Shift in that order, then the key.
pub fn format(mods: Modifiers, key: Key) -> String {
    let mut s = String::new();
    if mods.command || mods.ctrl {
        s.push_str("Ctrl+");
    }
    if mods.alt {
        s.push_str("Alt+");
    }
    if mods.shift {
        s.push_str("Shift+");
    }
    s.push_str(&key_name(key));
    s
}

/// The modifiers and key of shortcut text; `None` when it names no key.
pub fn parse(text: &str) -> Option<(Modifiers, Key)> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    // "Ctrl++": the key itself is a plus.
    let (mods_part, key_part) = match text.strip_suffix("++") {
        Some(rest) => (rest, "+"),
        None => match text.rfind('+') {
            Some(i) if i + 1 < text.len() => (&text[..i], &text[i + 1..]),
            _ => ("", text),
        },
    };
    let mut mods = Modifiers::NONE;
    for m in mods_part
        .split('+')
        .map(str::trim)
        .filter(|m| !m.is_empty())
    {
        match m.to_ascii_lowercase().as_str() {
            "ctrl" | "cmd" | "command" => mods |= Modifiers::COMMAND,
            "alt" | "option" => mods.alt = true,
            "shift" => mods.shift = true,
            _ => return None,
        }
    }
    Some((mods, key_from_name(key_part.trim())?))
}

/// Whether two shortcut texts name the same keys.
pub fn same(a: &str, b: &str) -> bool {
    match (parse(a), parse(b)) {
        (Some((ma, ka)), Some((mb, kb))) => {
            ka == kb
                && (ma.command || ma.ctrl) == (mb.command || mb.ctrl)
                && ma.alt == mb.alt
                && ma.shift == mb.shift
        }
        _ => false,
    }
}

/// The keys that run `id` (a menu command's key or a tool's id): the
/// user's keys for it when there are any (a command may have several,
/// #822), else its `built_in` keys. An empty entry of the user's means
/// the built-in keys were deleted.
pub fn keys_for(user: &[(String, String)], id: &str, built_in: Option<&str>) -> Vec<String> {
    let mut mine = user.iter().filter(|(k, _)| k == id).peekable();
    if mine.peek().is_none() {
        return built_in
            .filter(|b| !b.is_empty())
            .map(|b| vec![b.to_string()])
            .unwrap_or_default();
    }
    mine.filter(|(_, v)| !v.is_empty())
        .map(|(_, v)| v.clone())
        .collect()
}

/// The editor's shortcuts a browser may keep for itself (#1409), with the
/// key that runs the same command in the browser version: `None` where
/// the command needs none there (Exit: close the tab). The page keeps
/// every Ctrl key it can from the browser (`web_page`); these are the
/// ones browsers never hand to a page (Ctrl+N, Ctrl+T, Ctrl+W,
/// Ctrl+Shift+T, Ctrl+F4, Alt+F4) or keep in some browsers or systems
/// (Ctrl+PgUp/PgDn and Ctrl+1 to Ctrl+9 switch tabs, Ctrl+Q and
/// Ctrl+Shift+Q quit on Linux). Each alternative is a combination the
/// editor binds to nothing else and the browsers leave to pages
/// (`tests::browser_alternatives_are_free`).
pub const BROWSER_KEYS: &[(&str, Option<&str>)] = &[
    ("Ctrl+N", Some("Alt+Shift+N")),
    ("Ctrl+T", Some("Alt+Shift+X")),
    ("Ctrl+Shift+T", Some("Alt+Shift+T")),
    ("Ctrl+W", Some("Ctrl+F5")),
    ("Ctrl+F4", Some("Alt+Shift+W")),
    ("Alt+F4", None),
    ("Ctrl+PgUp", Some("Alt+PgUp")),
    ("Ctrl+PgDn", Some("Alt+PgDn")),
    ("Ctrl+Q", Some("Alt+Shift+C")),
    ("Ctrl+Shift+Q", Some("Alt+Shift+Q")),
    ("Ctrl+0", Some("Alt+Shift+0")),
    ("Ctrl+1", Some("Alt+Shift+1")),
    ("Ctrl+2", Some("Alt+Shift+2")),
    ("Ctrl+3", Some("Alt+Shift+3")),
    ("Ctrl+4", Some("Alt+Shift+4")),
    ("Ctrl+5", Some("Alt+Shift+5")),
    ("Ctrl+6", Some("Alt+Shift+6")),
    ("Ctrl+7", Some("Alt+Shift+7")),
    ("Ctrl+8", Some("Alt+Shift+8")),
    ("Ctrl+9", Some("Alt+Shift+9")),
];

/// The browser version's key for a command whose `shortcut` the browser
/// may keep ([`BROWSER_KEYS`]); `None` for any other shortcut, or one
/// with no alternative.
pub fn browser_alternative(shortcut: &str) -> Option<&'static str> {
    BROWSER_KEYS
        .iter()
        .find(|(k, _)| same(k, shortcut))
        .and_then(|(_, alt)| *alt)
}

#[cfg(test)]
thread_local! {
    /// Tests switch the browser version's shortcuts on in a desktop build.
    static WEB_KEYS_FOR_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the browser version's shortcuts apply: in the browser build,
/// and in tests that ask for them.
pub fn web_keys() -> bool {
    #[cfg(test)]
    if WEB_KEYS_FOR_TEST.with(|w| w.get()) {
        return true;
    }
    crate::files::WEB
}

#[cfg(test)]
pub fn set_web_keys_for_test(on: bool) {
    WEB_KEYS_FOR_TEST.with(|w| w.set(on));
}

/// The Command keys macOS keeps for itself (#1409), in shortcut text
/// (`Ctrl` is the Command key there): Quit and Log Out (Command+Q,
/// Command+Shift+Q), Hide and Hide Others (Command+H, Command+Option+H),
/// Minimize (Command+M), the application switcher (Command+Tab,
/// Command+Shift+Tab), Spotlight and the Finder's search
/// (Command+Space, Command+Option+Space), Force Quit
/// (Command+Option+Esc), the screenshots (Command+Shift+3, 4 and 5) and
/// hiding the Dock (Command+Option+D). The application menu or the
/// system takes them before the window sees them.
pub const MAC_SYSTEM_KEYS: &[&str] = &[
    "Ctrl+Q",
    "Ctrl+Shift+Q",
    "Ctrl+H",
    "Ctrl+Alt+H",
    "Ctrl+M",
    "Ctrl+Tab",
    "Ctrl+Shift+Tab",
    "Ctrl+Space",
    "Ctrl+Alt+Space",
    "Ctrl+Alt+Esc",
    "Ctrl+Shift+3",
    "Ctrl+Shift+4",
    "Ctrl+Shift+5",
    "Ctrl+Alt+D",
];

/// The editor's own keys among [`MAC_SYSTEM_KEYS`], with the key that
/// runs the same command in the macOS desktop app (#1409): Convert to
/// Curves, Convert Outline to Object, Merge Cells, the Pick tool from any
/// tool and Force Justify while typing. Each alternative is bound to
/// nothing else (`ui::options::tests::no_two_default_commands_share_a_shortcut`
/// in its macOS run). Quit keeps Command+Q ([`MAC_QUIT`]).
pub const MAC_KEYS: &[(&str, &str)] = &[
    ("Ctrl+Q", "Alt+Shift+C"),
    ("Ctrl+Shift+Q", "Alt+Shift+Q"),
    ("Ctrl+M", "Ctrl+Shift+M"),
    ("Ctrl+Space", "Ctrl+Shift+Space"),
    ("Ctrl+H", "Ctrl+Shift+H"),
];

/// File > Exit's key in the macOS desktop app: Command+Q, the application
/// menu's Quit, which asks about unsaved drawings as Exit does (#1669).
pub const MAC_QUIT: &str = "Ctrl+Q";

/// File > Exit's key on the other systems.
const EXIT_KEY: &str = "Alt+F4";

#[cfg(test)]
thread_local! {
    /// Tests switch the macOS desktop app's shortcuts on elsewhere.
    static MAC_KEYS_FOR_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Whether the macOS desktop app's shortcuts apply: in the desktop build
/// for macOS, and in tests that ask for them.
pub fn mac_keys() -> bool {
    #[cfg(test)]
    if MAC_KEYS_FOR_TEST.with(|w| w.get()) {
        return true;
    }
    cfg!(target_os = "macos") && !crate::files::WEB
}

#[cfg(test)]
pub fn set_mac_keys_for_test(on: bool) {
    MAC_KEYS_FOR_TEST.with(|w| w.set(on));
}

/// Whether `shortcut` is one of the Command keys macOS keeps
/// ([`MAC_SYSTEM_KEYS`]).
pub fn mac_system_key(shortcut: &str) -> bool {
    MAC_SYSTEM_KEYS.iter().any(|k| same(k, shortcut))
}

/// The macOS desktop app's key for a command whose `shortcut` macOS
/// keeps ([`MAC_KEYS`]); `None` for any other shortcut.
pub fn mac_alternative(shortcut: &str) -> Option<&'static str> {
    MAC_KEYS
        .iter()
        .find(|(k, _)| same(k, shortcut))
        .map(|(_, alt)| *alt)
}

/// A key press in the macOS desktop app: the editor shortcut it stands
/// for when it is the alternative of a key macOS keeps ([`MAC_KEYS`]);
/// `None` in other builds and for other keys.
pub fn mac_key_for(key: Key, mods: Modifiers) -> Option<(Modifiers, Key)> {
    if !mac_keys() {
        return None;
    }
    let text = format(mods, key);
    MAC_KEYS
        .iter()
        .find(|(_, alt)| same(alt, &text))
        .and_then(|(k, _)| parse(k))
}

/// Whether a key press is one macOS keeps, in the macOS desktop app: the
/// window does not act on it should it arrive.
pub fn mac_kept_press(key: Key, mods: Modifiers) -> bool {
    mac_keys() && mac_system_key(&format(mods, key))
}

/// The shortcut a menu row shows: in the browser version the alternative
/// of a key the browser keeps, in the macOS desktop app the alternative
/// of a key macOS keeps (and Exit's Command+Q), else `shortcut` itself.
pub fn shown(shortcut: String) -> String {
    if web_keys() {
        return browser_alternative(&shortcut)
            .map(str::to_string)
            .unwrap_or(shortcut);
    }
    mac_shown(shortcut)
}

/// The key a command's built-in `shortcut` stands for in the macOS
/// desktop app: the alternative of a key macOS keeps, Exit's Command+Q
/// ([`MAC_QUIT`]), else `shortcut` itself. Other builds keep `shortcut`.
/// The menus and Options > Customization > Commands show it there.
pub fn mac_shown(shortcut: String) -> String {
    if !mac_keys() {
        return shortcut;
    }
    if same(&shortcut, EXIT_KEY) {
        return MAC_QUIT.to_string();
    }
    mac_alternative(&shortcut)
        .map(str::to_string)
        .unwrap_or(shortcut)
}

/// A tip that names its shortcut in brackets ("New (Ctrl+N)", "(Ctrl+Q
/// converts to curves)"): in the browser version the alternative of a
/// key the browser keeps takes its place ("New (Alt+Shift+N)"), in the
/// macOS desktop app the alternative of a key macOS keeps.
pub fn shown_in(text: String) -> String {
    let swap = |t: String, k: &str, alt: &str| {
        t.replace(&format!("({k})"), &format!("({alt})"))
            .replace(&format!("({k} "), &format!("({alt} "))
    };
    if web_keys() {
        return BROWSER_KEYS.iter().fold(text, |t, (k, alt)| match alt {
            Some(alt) => swap(t, k, alt),
            None => t,
        });
    }
    if mac_keys() {
        let text = swap(text, EXIT_KEY, MAC_QUIT);
        return MAC_KEYS.iter().fold(text, |t, (k, alt)| swap(t, k, alt));
    }
    text
}

/// The browser's keys and their alternatives as the Options notice lists
/// them: "Ctrl+N: Alt+Shift+N, ...", with the digits in one entry.
pub fn browser_keys_list() -> String {
    let mut out: Vec<String> = Vec::new();
    let mut digits = false;
    for (k, alt) in BROWSER_KEYS {
        let Some(alt) = alt else { continue };
        let digit = k
            .strip_prefix("Ctrl+")
            .is_some_and(|d| d.chars().all(|c| c.is_ascii_digit()));
        if digit {
            if !digits {
                out.push("Ctrl+0..9: Alt+Shift+0..9".into());
                digits = true;
            }
            continue;
        }
        out.push(format!("{k}: {alt}"));
    }
    out.join(", ")
}

/// The editor shortcut a key press stands for in the browser version: the
/// key the browser keeps when the press is its alternative. The digits
/// are read from the physical key, since Shift turns them into other
/// characters (`!`, `@`, `§`) depending on the keyboard layout.
pub fn browser_key_for(
    key: Key,
    physical: Option<Key>,
    mods: Modifiers,
) -> Option<(Modifiers, Key)> {
    let digit = |k: Key| {
        matches!(
            k,
            Key::Num0
                | Key::Num1
                | Key::Num2
                | Key::Num3
                | Key::Num4
                | Key::Num5
                | Key::Num6
                | Key::Num7
                | Key::Num8
                | Key::Num9
        )
    };
    let key = physical.filter(|p| digit(*p)).unwrap_or(key);
    let text = format(mods, key);
    BROWSER_KEYS
        .iter()
        .find(|(_, alt)| alt.is_some_and(|a| same(a, &text)))
        .and_then(|(k, _)| parse(k))
}

/// Shift, Ctrl, Alt and the system key pressed on their own (they arrive
/// as key presses too, before the key they modify).
pub fn is_modifier(key: Key) -> bool {
    matches!(
        key,
        Key::ShiftLeft
            | Key::ShiftRight
            | Key::ControlLeft
            | Key::ControlRight
            | Key::AltLeft
            | Key::AltRight
            | Key::SuperLeft
            | Key::SuperRight
    )
}

/// Whether a key press can be a shortcut: not a modifier alone, and not
/// a key that edits text, moves focus or closes dialogs without Ctrl or
/// Alt. In the macOS desktop app the Command keys macOS keeps are no
/// shortcut either: they would never reach the window (#1409).
pub fn allowed(mods: Modifiers, key: Key) -> bool {
    let bare = !(mods.command || mods.ctrl || mods.alt);
    !is_modifier(key)
        && !mac_kept_press(key, mods)
        && !(bare
            && matches!(
                key,
                Key::Escape | Key::Tab | Key::Enter | Key::Backspace | Key::Space
            ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcut_text_round_trips() {
        let ctrl_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        assert_eq!(format(ctrl_shift, Key::E), "Ctrl+Shift+E");
        assert_eq!(format(Modifiers::ALT, Key::Enter), "Alt+Enter");
        assert_eq!(format(Modifiers::COMMAND, Key::PageDown), "Ctrl+PgDn");
        assert_eq!(format(Modifiers::NONE, Key::F5), "F5");
        for text in [
            "Ctrl+Shift+E",
            "Alt+Enter",
            "Ctrl+PgDn",
            "F5",
            "Ctrl++",
            "Alt+Shift+A",
            "Shift+F12",
        ] {
            let (m, k) = parse(text).expect(text);
            assert_eq!(format(m, k), text);
        }
        assert_eq!(parse("Ctrl++").map(|(_, k)| k), Some(Key::Plus));
        assert!(same("ctrl+shift+e", "Ctrl+Shift+E"));
        assert!(!same("Ctrl+E", "Ctrl+Shift+E"));
        assert!(
            parse("Hyper+E").is_none() && parse("").is_none() && parse("Ctrl+Nothing").is_none()
        );
        assert!(!allowed(Modifiers::NONE, Key::Escape));
        assert!(allowed(Modifiers::COMMAND, Key::Enter) && allowed(Modifiers::NONE, Key::F9));
        assert!(
            !allowed(Modifiers::ALT, Key::AltLeft)
                && !allowed(Modifiers::COMMAND, Key::ControlRight)
        );
    }

    /// Whether `shortcut` may stay with the browser in the browser version.
    fn browser_keeps(shortcut: &str) -> bool {
        BROWSER_KEYS.iter().any(|(k, _)| same(k, shortcut))
    }

    /// #1409: the keys browsers keep map to the browser version's keys,
    /// and those keys back to the command's own shortcut.
    #[test]
    fn browser_kept_keys_map_to_alternatives() {
        assert_eq!(browser_alternative("Ctrl+N"), Some("Alt+Shift+N"));
        assert_eq!(browser_alternative("ctrl+w"), Some("Ctrl+F5"));
        assert_eq!(browser_alternative("Ctrl+PgUp"), Some("Alt+PgUp"));
        assert_eq!(browser_alternative("Ctrl+PgDn"), Some("Alt+PgDn"));
        assert_eq!(browser_alternative("Ctrl+F4"), Some("Alt+Shift+W"));
        assert_eq!(browser_alternative("Ctrl+3"), Some("Alt+Shift+3"));
        // Exit needs none (close the tab); keys the page keeps need none.
        assert!(browser_keeps("Alt+F4") && browser_alternative("Alt+F4").is_none());
        for k in [
            "Ctrl+E",
            "Ctrl+R",
            "F5",
            "Ctrl+Home",
            "Shift+PgUp",
            "Ctrl+Shift+N",
        ] {
            assert!(!browser_keeps(k) && browser_alternative(k).is_none(), "{k}");
        }
        // Each alternative is given once and keeps none of the taken keys.
        let alts: Vec<&str> = BROWSER_KEYS.iter().filter_map(|(_, a)| *a).collect();
        for (i, a) in alts.iter().enumerate() {
            assert!(parse(a).is_some(), "{a}");
            assert!(!browser_keeps(a), "{a}");
            assert!(!alts[..i].iter().any(|b| same(a, b)), "{a}");
        }
        // Pressed in the browser, the alternative stands for the key the
        // browser keeps; the digits by their physical key (Shift+1 is `!`
        // on many layouts).
        assert_eq!(
            browser_key_for(Key::PageUp, None, Modifiers::ALT),
            Some((Modifiers::COMMAND, Key::PageUp))
        );
        assert_eq!(
            browser_key_for(Key::N, Some(Key::N), Modifiers::ALT | Modifiers::SHIFT),
            Some((Modifiers::COMMAND, Key::N))
        );
        assert_eq!(
            browser_key_for(Key::F5, None, Modifiers::COMMAND),
            Some((Modifiers::COMMAND, Key::W))
        );
        assert_eq!(
            browser_key_for(
                Key::Exclamationmark,
                Some(Key::Num1),
                Modifiers::ALT | Modifiers::SHIFT
            ),
            Some((Modifiers::COMMAND, Key::Num1))
        );
        assert_eq!(browser_key_for(Key::PageUp, None, Modifiers::COMMAND), None);
        assert_eq!(browser_key_for(Key::E, None, Modifiers::ALT), None);
        // The menus show the alternative in the browser version only.
        set_web_keys_for_test(false);
        assert_eq!(shown("Ctrl+PgUp".into()), "Ctrl+PgUp");
        set_web_keys_for_test(true);
        assert_eq!(shown("Ctrl+PgUp".into()), "Alt+PgUp");
        assert_eq!(shown("Alt+F4".into()), "Alt+F4");
        assert_eq!(shown("Ctrl+E".into()), "Ctrl+E");
        assert_eq!(shown_in("New (Ctrl+N)".into()), "New (Alt+Shift+N)");
        assert_eq!(
            shown_in("Edit Text (Ctrl+Shift+T)".into()),
            "Edit Text (Alt+Shift+T)"
        );
        assert_eq!(shown_in("Text (Ctrl+T)".into()), "Text (Alt+Shift+X)");
        assert_eq!(shown_in("Export (Ctrl+E)".into()), "Export (Ctrl+E)");
        set_web_keys_for_test(false);
        assert_eq!(shown_in("New (Ctrl+N)".into()), "New (Ctrl+N)");
        assert_eq!(
            browser_keys_list(),
            "Ctrl+N: Alt+Shift+N, Ctrl+T: Alt+Shift+X, Ctrl+Shift+T: Alt+Shift+T, \
             Ctrl+W: Ctrl+F5, Ctrl+F4: Alt+Shift+W, Ctrl+PgUp: Alt+PgUp, \
             Ctrl+PgDn: Alt+PgDn, Ctrl+Q: Alt+Shift+C, Ctrl+Shift+Q: Alt+Shift+Q, \
             Ctrl+0..9: Alt+Shift+0..9"
        );
    }
}
