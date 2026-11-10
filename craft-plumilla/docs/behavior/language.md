# Interface language at first start

Which language TraceDraw's menus, dialogs and tips start in, and how the
Windows installer's language choice reaches the app. The languages
themselves are listed in `docs/i18n.md`.

## The saved language

`settings.json` keeps `language`, what the user picked:

| `language` | Meaning | Start language |
|---|---|---|
| a code (`de`, `pt-BR`, `zh-TW`...) | picked in Options (or on the Welcome Screen) | that language, always |
| `system` | System language picked in Options | the system's language |
| empty (or a code this version does not have) | nothing picked yet: a new install, or after Reset all settings | the installer's choice, then the system's language |

Settings files of older versions have a code, which they keep: an
English saved by an older version stays English. The language shown is
never written as a choice; only Options and the Welcome Screen write
`language`, so a language that came from the installer or the system
keeps following them.

## Precedence at start

`i18n::startup_language(saved, installer, system)`:

1. `saved` is a supported code: that language.
2. `saved` is `system`: the system's language (the installer is not
   asked).
3. Otherwise the installer's choice: a code gives that language;
   `system` (the installer left at System language) gives the system's
   language.
4. No installer choice (portable zip, Microsoft Store, macOS, Linux,
   Android, the browser) or one this version does not know: the
   system's language.
5. English when none of the system's languages is supported.

The installer's value is not consumed: it stays the default for as long
as nothing is picked in Options, so a reinstall with another language
choice changes the start language of a user who never picked one, and
the app writes nothing to the registry.

## The system's language

`system_locale::preferred_languages()` gives the system's languages, most
preferred first, and `i18n::best_language` takes the first TraceDraw has:

- Windows: `GetUserPreferredUILanguages` (the display languages of
  Settings > Time & language), else `GetUserDefaultUILanguage` mapped by
  `i18n::language_for_langid`.
- macOS: `CFLocaleCopyPreferredLanguages` (System Settings > General >
  Language & Region), else the variables below.
- Linux and other Unix systems: the `LANGUAGE` list (left aside when the
  locale is plain `C`, as gettext does), then the first set of `LC_ALL`,
  `LC_MESSAGES` and `LANG`.
- Browser: `navigator.languages`, then `navigator.language`.

`i18n::match_locale` matches one name (`pt_PT.UTF-8`, `zh-Hant-HK`,
`sr_RS@latin`) to the closest supported language: the encoding and the
modifier are dropped and the language subtag is compared whole (`arn` is
not Arabic), so another region of a shipped language gives that language
(Portuguese of Portugal gives Portuguese (Brazil), `es-MX` Spanish).
Chinese goes by script, then region: `Hant`, `TW`, `HK` and `MO` give
Traditional Chinese, anything else Simplified. `in` (the old code of
Indonesian) gives Indonesian; `C` and `POSIX` give nothing.

## The Windows installer

`packaging/windows/tracedraw.iss` (Inno Setup):

- The wizard runs in the system's language when Inno Setup ships a
  translation for it, else in English (`ShowLanguageDialog=no`).
- The page "TraceDraw's language", after Select Additional Tasks, has one
  list: "System language (<the language it gives here>)" first, then the
  eighteen interface languages in the app's order. System language is
  selected, so an install where nothing is changed follows the system.
  The language System language names comes from the Windows UI language
  (`GetUILanguage`) through `AppLanguageForLangId`, the same table as
  `i18n::language_for_langid` (English when there is none).
- A reinstall or update selects what the last install recorded;
  `/APPLANGUAGE=<code>` or `/APPLANGUAGE=system` on the command line
  selects that (silent installs).
- The choice is recorded as the string value `InstallerLanguage` of
  `Software\TraceDraw`: `system` or the code (`en`, `pt-BR`, `zh-CN`,
  `zh-TW`...). The key is under HKEY_CURRENT_USER for an install for the
  user alone, HKEY_LOCAL_MACHINE for one for all users; the app reads
  HKCU first, then HKLM. Uninstalling removes the value.
- A portable copy (a `tracedraw-settings` folder beside the program) and
  the Microsoft Store's copy do not read the value.

## Options

Options > Global > General > Language lists "System language (<the
language it gives now>)" first, then every language. System language
saves `system`; a language saves its code. Both apply at once.

## Checks

- `i18n::tests::locales_match_the_closest_supported_language`
- `i18n::tests::locales_choose_the_interface_language`
- `i18n::tests::windows_language_ids_map_to_our_codes`
- `i18n::tests::the_installer_maps_languages_as_the_app_does` (reads the
  installer script: its id table and its list of languages equal the
  app's)
- `i18n::tests::the_installer_value_is_a_code_or_system`
- `i18n::tests::the_start_language_follows_the_precedence`
- `settings::tests::old_settings_files_keep_their_language`
- `app::tests::a_language_not_picked_is_not_saved_as_picked`
- `system_locale::tests::posix_variables_are_read_in_gettext_order`
- `system_locale::tests::windows_multi_strings_split_into_names`
