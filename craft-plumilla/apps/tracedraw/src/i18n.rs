//! User interface strings. `tr("menu.file.open")` returns the string in
//! the active language, falling back to English, then to the key itself.
//!
//! Language files are flat `key = "value"` tables in `lang/` (a TOML
//! subset) embedded at build time, so the binary stays self-contained.

use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

/// Languages shipped with the app, in display order.
pub const LANGUAGES: [(&str, &str); 18] = [
    ("en", "English"),
    ("zh-CN", "中文（简体）"),
    ("hi", "हिन्दी"),
    ("es", "Español"),
    ("fr", "Français"),
    ("ar", "العربية"),
    ("bn", "বাংলা"),
    ("ru", "Русский"),
    ("pt-BR", "Português (Brasil)"),
    ("id", "Bahasa Indonesia"),
    ("de", "Deutsch"),
    ("ja", "日本語"),
    ("zh-TW", "中文（繁體）"),
    ("it", "Italiano"),
    ("nl", "Nederlands"),
    ("pl", "Polski"),
    ("tr", "Türkçe"),
    ("cs", "Čeština"),
];

const FILES: [(&str, &str); 18] = [
    ("en", include_str!("../lang/en.toml")),
    ("zh-CN", include_str!("../lang/zh-CN.toml")),
    ("hi", include_str!("../lang/hi.toml")),
    ("es", include_str!("../lang/es.toml")),
    ("fr", include_str!("../lang/fr.toml")),
    ("ar", include_str!("../lang/ar.toml")),
    ("bn", include_str!("../lang/bn.toml")),
    ("ru", include_str!("../lang/ru.toml")),
    ("pt-BR", include_str!("../lang/pt-BR.toml")),
    ("id", include_str!("../lang/id.toml")),
    ("de", include_str!("../lang/de.toml")),
    ("ja", include_str!("../lang/ja.toml")),
    ("zh-TW", include_str!("../lang/zh-TW.toml")),
    ("it", include_str!("../lang/it.toml")),
    ("nl", include_str!("../lang/nl.toml")),
    ("pl", include_str!("../lang/pl.toml")),
    ("tr", include_str!("../lang/tr.toml")),
    ("cs", include_str!("../lang/cs.toml")),
];

struct State {
    code: String,
    table: HashMap<String, String>,
    english: HashMap<String, String>,
}

fn state() -> &'static RwLock<State> {
    static STATE: OnceLock<RwLock<State>> = OnceLock::new();
    STATE.get_or_init(|| {
        let english = parse(FILES[0].1);
        RwLock::new(State {
            code: "en".into(),
            table: english.clone(),
            english,
        })
    })
}

/// Parse the flat `key = "value"` format. Lines starting with `#` and
/// blank lines are ignored; values support `\"`, `\\` and `\n`.
pub fn parse(src: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some((key, rest)) = line.split_once('=') else {
            continue;
        };
        let rest = rest.trim();
        let Some(body) = rest.strip_prefix('"') else {
            continue;
        };
        let mut value = String::new();
        let mut chars = body.chars();
        while let Some(c) = chars.next() {
            match c {
                '\\' => match chars.next() {
                    Some('n') => value.push('\n'),
                    Some('"') => value.push('"'),
                    Some('\\') => value.push('\\'),
                    Some(o) => {
                        value.push('\\');
                        value.push(o);
                    }
                    None => {}
                },
                '"' => break,
                o => value.push(o),
            }
        }
        map.insert(key.trim().to_string(), value);
    }
    map
}

/// Switch the active language; unknown codes fall back to English.
pub fn set_language(code: &str) {
    let src = FILES
        .iter()
        .find(|(c, _)| *c == code)
        .map(|(_, s)| *s)
        .unwrap_or(FILES[0].1);
    if let Ok(mut st) = state().write() {
        st.code = code.to_string();
        st.table = parse(src);
    }
}

/// Every key of the English table.
pub fn english_keys() -> Vec<String> {
    state()
        .read()
        .map(|s| s.english.keys().cloned().collect())
        .unwrap_or_default()
}

pub fn language() -> String {
    #[cfg(test)]
    if let Some(code) = THREAD_CODE.with(|t| t.borrow().clone()) {
        return code;
    }
    state().read().map(|s| s.code.clone()).unwrap_or_default()
}

/// Right-to-left languages flip the docker side and text alignment.
pub fn is_rtl() -> bool {
    language() == "ar"
}

#[cfg(test)]
thread_local! {
    /// A language for this thread only, so that a test can lay the
    /// interface out in another language without changing it for the
    /// tests running beside it.
    static THREAD_TABLE: std::cell::RefCell<Option<HashMap<String, String>>> =
        const { std::cell::RefCell::new(None) };
    static THREAD_CODE: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// Run `f` with `code` as the language of this thread only (tests).
#[cfg(test)]
pub fn with_thread_language<R>(code: &str, f: impl FnOnce() -> R) -> R {
    let (chosen_code, src) = FILES
        .iter()
        .find(|(c, _)| *c == code)
        .copied()
        .unwrap_or(FILES[0]);
    let before_code = THREAD_CODE.with(|t| t.replace(Some(chosen_code.to_owned())));
    let before = THREAD_TABLE.with(|t| t.replace(Some(parse(src))));
    let out = f();
    THREAD_TABLE.with(|t| *t.borrow_mut() = before);
    THREAD_CODE.with(|t| *t.borrow_mut() = before_code);
    out
}

/// Translate a key.
pub fn tr(key: &str) -> String {
    #[cfg(test)]
    if let Some(v) = THREAD_TABLE.with(|t| t.borrow().as_ref().and_then(|m| m.get(key).cloned())) {
        return v;
    }
    if let Ok(st) = state().read() {
        if let Some(v) = st.table.get(key) {
            return v.clone();
        }
        if let Some(v) = st.english.get(key) {
            return v.clone();
        }
    }
    log::debug!("missing i18n key {key}");
    key.rsplit('.').next().unwrap_or(key).to_string()
}

/// Whether the English table has `key` (generated keys, optional texts).
pub fn has_english(key: &str) -> bool {
    state()
        .read()
        .map(|st| st.english.contains_key(key))
        .unwrap_or(false)
}

/// Translate with `{name}` placeholders replaced.
pub fn trf(key: &str, args: &[(&str, &str)]) -> String {
    let mut s = tr(key);
    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    s
}

/// The saved language that means "follow the system's language": the
/// user picked System language in Options. An empty saved language means
/// no language was ever picked: the installer's choice, then the system's.
pub const SYSTEM: &str = "system";

/// The supported language with this code, compared without case.
pub fn supported(code: &str) -> Option<&'static str> {
    LANGUAGES
        .iter()
        .map(|(c, _)| *c)
        .find(|c| c.eq_ignore_ascii_case(code.trim()))
}

/// The interface language for one locale name: a POSIX locale
/// (`pt_BR.UTF-8`, `sr_RS@latin`), a BCP 47 tag (`zh-Hant-HK`, `pt-PT`)
/// or a bare language (`de`). The closest supported language: the same
/// language in another region (Portuguese of Portugal reads Brazilian
/// Portuguese), Chinese by its script or region (Hant, TW, HK, MO:
/// Traditional; otherwise Simplified). `None` for `C`, `POSIX`, an empty
/// name or a language Craft Plumilla does not have.
pub fn match_locale(locale: &str) -> Option<&'static str> {
    // Drop the encoding and the modifier, then split into subtags.
    let name = locale.trim();
    let name = name.split(['.', '@']).next().unwrap_or("");
    let lower = name.to_ascii_lowercase().replace('_', "-");
    let mut parts = lower.split('-').filter(|p| !p.is_empty());
    let lang = parts.next()?;
    let rest: Vec<&str> = parts.collect();
    match lang {
        "c" | "posix" => None,
        "zh" => {
            let traditional = rest
                .iter()
                .any(|p| matches!(*p, "hant" | "tw" | "hk" | "mo"));
            let simplified = rest.contains(&"hans");
            Some(if traditional && !simplified {
                "zh-TW"
            } else {
                "zh-CN"
            })
        }
        // The older code of Indonesian, still sent by some systems.
        "in" => Some("id"),
        _ => LANGUAGES.iter().map(|(c, _)| *c).find(|c| {
            c.split('-')
                .next()
                .is_some_and(|base| base.eq_ignore_ascii_case(lang))
        }),
    }
}

/// The first of the system's languages, in its order of preference,
/// that Craft Plumilla has.
pub fn best_language<S: AsRef<str>>(locales: &[S]) -> Option<&'static str> {
    locales.iter().find_map(|l| match_locale(l.as_ref()))
}

/// The interface language for a Windows language identifier (LANGID,
/// as `GetUserDefaultUILanguage` gives it). The installer has the same
/// table in Pascal (`AppLanguageForLangId` in
/// `packaging/windows/tracedraw.iss`); a test keeps the two equal.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn language_for_langid(id: u16) -> Option<&'static str> {
    let primary = id & 0x3ff;
    Some(match primary {
        0x09 => "en",
        0x04 => match id {
            // Taiwan, Hong Kong, Macau and the neutral Traditional.
            0x0404 | 0x0c04 | 0x1404 | 0x7c04 => "zh-TW",
            _ => "zh-CN",
        },
        0x39 => "hi",
        0x0a => "es",
        0x0c => "fr",
        0x01 => "ar",
        0x45 => "bn",
        0x19 => "ru",
        0x16 => "pt-BR",
        0x21 => "id",
        0x07 => "de",
        0x11 => "ja",
        0x10 => "it",
        0x13 => "nl",
        0x15 => "pl",
        0x1f => "tr",
        0x05 => "cs",
        _ => return None,
    })
}

/// The language the Windows installer recorded: [`SYSTEM`] when the
/// user left System language, else one of our codes. `None` for nothing
/// recorded or a value this version does not know.
pub fn installer_choice(value: &str) -> Option<&'static str> {
    let v = value.trim();
    if v.eq_ignore_ascii_case(SYSTEM) {
        return Some(SYSTEM);
    }
    supported(v)
}

/// The interface language at start. The language picked in Options
/// (`saved`, a supported code) always wins; [`SYSTEM`] follows the
/// system. Nothing picked (an empty or unknown `saved`): the installer's
/// choice (`installer`, as recorded), then the system's languages
/// (`system`, in order of preference), then English.
pub fn startup_language<S: AsRef<str>>(
    saved: &str,
    installer: Option<&str>,
    system: &[S],
) -> &'static str {
    let from_system = || best_language(system).unwrap_or("en");
    if saved.trim().eq_ignore_ascii_case(SYSTEM) {
        return from_system();
    }
    if let Some(code) = supported(saved) {
        return code;
    }
    match installer.and_then(installer_choice) {
        Some(SYSTEM) | None => from_system(),
        Some(code) => code,
    }
}

/// [`startup_language`] with the installer's choice and the system's
/// languages read from this machine.
pub fn startup_language_here(saved: &str) -> &'static str {
    if supported(saved).is_some() {
        return startup_language::<&str>(saved, None, &[]);
    }
    let installer = crate::system_locale::installer_language();
    let system = crate::system_locale::preferred_languages();
    let lang = startup_language(saved, installer.as_deref(), &system);
    log::info!(
        "interface language {lang} (saved {saved:?}, installer {installer:?}, system {system:?})"
    );
    lang
}

/// The language the system prefers, as one of ours (English if none).
pub fn system_language() -> &'static str {
    best_language(&crate::system_locale::preferred_languages()).unwrap_or("en")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn language_for_locale(locale: &str) -> &'static str {
        match_locale(locale).unwrap_or("en")
    }

    #[test]
    fn locales_choose_the_interface_language() {
        assert_eq!(language_for_locale("zh_TW.UTF-8"), "zh-TW");
        assert_eq!(language_for_locale("zh_HK.UTF-8"), "zh-TW");
        assert_eq!(language_for_locale("zh_CN.UTF-8"), "zh-CN");
        assert_eq!(language_for_locale("zh_SG"), "zh-CN");
        assert_eq!(language_for_locale("cs_CZ.UTF-8"), "cs");
        assert_eq!(language_for_locale("nl_BE"), "nl");
        assert_eq!(language_for_locale("it_IT"), "it");
        assert_eq!(language_for_locale("pl_PL"), "pl");
        assert_eq!(language_for_locale("tr_TR.UTF-8"), "tr");
        assert_eq!(language_for_locale("pt_BR.UTF-8"), "pt-BR");
        assert_eq!(language_for_locale("sv_SE"), "en");
        assert_eq!(language_for_locale(""), "en");
    }

    #[test]
    fn locales_match_the_closest_supported_language() {
        // Another region of a shipped language.
        assert_eq!(match_locale("pt-PT"), Some("pt-BR"));
        assert_eq!(match_locale("pt_PT.UTF-8"), Some("pt-BR"));
        assert_eq!(match_locale("es-MX"), Some("es"));
        assert_eq!(match_locale("fr-CA"), Some("fr"));
        assert_eq!(match_locale("de-CH"), Some("de"));
        assert_eq!(match_locale("en-GB"), Some("en"));
        // Chinese by script, then region.
        assert_eq!(match_locale("zh-Hant"), Some("zh-TW"));
        assert_eq!(match_locale("zh-Hant-HK"), Some("zh-TW"));
        assert_eq!(match_locale("zh-Hans-HK"), Some("zh-CN"));
        assert_eq!(match_locale("zh-Hans"), Some("zh-CN"));
        assert_eq!(match_locale("zh-MO"), Some("zh-TW"));
        assert_eq!(match_locale("zh"), Some("zh-CN"));
        // Codes compared whole, not by their first letters.
        assert_eq!(match_locale("arn-CL"), None);
        assert_eq!(match_locale("ido"), None);
        assert_eq!(match_locale("in_ID"), Some("id"));
        assert_eq!(match_locale("sr_RS@latin"), None);
        assert_eq!(match_locale("C"), None);
        assert_eq!(match_locale("POSIX"), None);
        assert_eq!(match_locale("C.UTF-8"), None);
        assert_eq!(match_locale("  "), None);
        // The first shipped language of a list of preferences.
        assert_eq!(best_language(&["sv-SE", "nb", "de-AT"]), Some("de"));
        assert_eq!(best_language(&["sv-SE"]), None);
        assert_eq!(best_language::<&str>(&[]), None);
    }

    #[test]
    fn windows_language_ids_map_to_our_codes() {
        assert_eq!(language_for_langid(0x0409), Some("en"));
        assert_eq!(language_for_langid(0x0809), Some("en"));
        assert_eq!(language_for_langid(0x0416), Some("pt-BR"));
        assert_eq!(language_for_langid(0x0816), Some("pt-BR"));
        assert_eq!(language_for_langid(0x0404), Some("zh-TW"));
        assert_eq!(language_for_langid(0x0c04), Some("zh-TW"));
        assert_eq!(language_for_langid(0x1404), Some("zh-TW"));
        assert_eq!(language_for_langid(0x0804), Some("zh-CN"));
        assert_eq!(language_for_langid(0x1004), Some("zh-CN"));
        assert_eq!(language_for_langid(0x0407), Some("de"));
        assert_eq!(language_for_langid(0x0c0a), Some("es"));
        assert_eq!(language_for_langid(0x0405), Some("cs"));
        assert_eq!(language_for_langid(0x041d), None); // Swedish
                                                       // Every shipped language has an id.
        for (code, _) in LANGUAGES {
            assert!(
                (0..0x8000u16).any(|id| language_for_langid(id) == Some(code)),
                "{code}"
            );
        }
    }

    /// The installer's Pascal table (`AppLanguageForLangId`) gives the
    /// same code as `language_for_langid` for every language id, and its
    /// list of languages to pick from is ours.
    #[test]
    fn the_installer_maps_languages_as_the_app_does() {
        let iss = include_str!("../../../packaging/windows/tracedraw.iss");
        let mut primaries = Vec::new();
        for line in iss.lines() {
            let line = line.trim();
            // `$09: Result := 'en';`
            let Some((id, rest)) = line.split_once(": Result := '") else {
                continue;
            };
            let Some(id) = id.strip_prefix('$') else {
                continue;
            };
            let id = u16::from_str_radix(id, 16).expect("hex id");
            let code = rest.trim_end_matches("';");
            if id <= 0x3ff {
                // A primary language, in another region than the first.
                assert_eq!(language_for_langid(0x0800 | id), Some(code), "{id:#x}");
                primaries.push(id);
            } else {
                assert_eq!(language_for_langid(id), Some(code), "{id:#x}");
            }
        }
        assert_eq!(
            primaries.len(),
            LANGUAGES.len() - 1,
            "one per language, Chinese apart"
        );
        // The choices of the installer's page: every code, in our order.
        let codes: Vec<&str> = iss
            .lines()
            .filter_map(|l| l.trim().strip_prefix("AddAppLanguage('"))
            .filter_map(|l| l.split('\'').next())
            .collect();
        let ours: Vec<&str> = LANGUAGES.iter().map(|(c, _)| *c).collect();
        assert_eq!(codes, ours);
    }

    #[test]
    fn the_installer_value_is_a_code_or_system() {
        assert_eq!(installer_choice("system"), Some(SYSTEM));
        assert_eq!(installer_choice("System"), Some(SYSTEM));
        assert_eq!(installer_choice("pt-BR"), Some("pt-BR"));
        assert_eq!(installer_choice("pt-br"), Some("pt-BR"));
        assert_eq!(installer_choice(" de "), Some("de"));
        assert_eq!(installer_choice("klingon"), None);
        assert_eq!(installer_choice(""), None);
    }

    /// Picked in Options > installer > system > English.
    #[test]
    fn the_start_language_follows_the_precedence() {
        let system = ["sv-SE", "fr-FR"];
        // A language picked in Options always wins.
        assert_eq!(startup_language("de", Some("ja"), &system), "de");
        assert_eq!(startup_language("zh-TW", Some("system"), &system), "zh-TW");
        // An English that was picked (or saved by an older version) stays.
        assert_eq!(startup_language("en", Some("ja"), &system), "en");
        // System language picked in Options: the system, not the installer.
        assert_eq!(startup_language("system", Some("ja"), &system), "fr");
        assert_eq!(startup_language("system", None, &["sv"]), "en");
        // Nothing picked: the installer's language.
        assert_eq!(startup_language("", Some("ja"), &system), "ja");
        // The installer left at System language: the system's.
        assert_eq!(startup_language("", Some("system"), &system), "fr");
        // No installer (zip, Linux, macOS, Store), or a value we do not
        // know: the system's.
        assert_eq!(startup_language("", None, &system), "fr");
        assert_eq!(startup_language("", Some("xx"), &system), "fr");
        assert_eq!(startup_language("", None, &["pt-PT"]), "pt-BR");
        // An unknown saved language counts as none picked.
        assert_eq!(startup_language("sv", Some("it"), &system), "it");
        // Nothing at all: English.
        assert_eq!(startup_language::<&str>("", None, &[]), "en");
    }

    #[test]
    fn parses_escapes_and_comments() {
        let m = parse("# c\na.b = \"x \\\"y\\\" \\n z\"\nbad\n");
        assert_eq!(m.get("a.b").map(String::as_str), Some("x \"y\" \n z"));
        assert_eq!(m.len(), 1);
    }

    #[test]
    fn every_language_covers_english_keys() {
        let en = parse(FILES[0].1);
        assert!(en.len() > 100);
        for (code, src) in FILES.iter().skip(1) {
            let t = parse(src);
            let missing: Vec<_> = en.keys().filter(|k| !t.contains_key(*k)).collect();
            assert!(missing.is_empty(), "{code} is missing {missing:?}");
        }
    }

    #[test]
    fn fallback_to_english_then_key() {
        set_language("zz");
        assert_eq!(tr("menu.file"), "File");
        assert_eq!(tr("no.such.key"), "key");
    }
}

#[cfg(test)]
mod jpeg_export_tests {
    use super::*;
    #[test]
    fn jpeg_export_labels_and_worker_status_cover_every_packaged_language() {
        let english = parse(FILES[0].1);
        let statuses = [
            "status.export_rendering",
            "status.export_encoding",
            "status.export_writing",
            "status.export_cancelled",
            "status.export_busy",
            "status.export_worker_failed",
            "status.export_progress",
            "status.export_cancelling",
        ];
        let keys: Vec<_> = english
            .keys()
            .filter(|key| key.starts_with("jpeg.") || statuses.contains(&key.as_str()))
            .collect();
        assert!(keys.len() >= 40);
        for (code, source) in FILES {
            let table = parse(source);
            for key in &keys {
                assert!(
                    table
                        .get(*key)
                        .is_some_and(|value| !value.trim().is_empty()),
                    "{code}: {key}"
                );
            }
            for token in ["{phase}", "{percent}"] {
                assert!(table["status.export_progress"].contains(token), "{code}");
            }
            assert!(
                table["status.export_worker_failed"].contains("{e}"),
                "{code}"
            );
            assert!(table["jpeg.transfer_time"].contains("{time}"), "{code}");
        }
        let es = parse(FILES.iter().find(|(code, _)| *code == "es").unwrap().1);
        assert_eq!(es["jpeg.settings"], "Configuración");
        assert_eq!(es["jpeg.progressive"], "Progresivo");
        assert_eq!(es["jpeg.keep_size"], "Conservar tamaño");
        assert_eq!(es["jpeg.crop_to_page"], "Recortar por página al exportar");
        assert_eq!(es["jpeg.medium"], "Medio");
        assert_eq!(es["jpeg.subformat_standard"], "Estándar (4:2:2)");
        assert_eq!(es["jpeg.colour_settings"], "Configuración de color:");
        assert_eq!(es["jpeg.cmyk"], "Color CMYK (32 bits)");
        assert_eq!(es["jpeg.ok"], "OK");
        assert_eq!(es["dialog.ok"], "Aceptar");
        assert_eq!(es["jpeg.speed"], "Velocidad:");
    }
}
