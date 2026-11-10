# What's new

- Help > What's New opens the Welcome Screen's What's New page: the
  history of the last ten releases, newest first. Each row (a collapsing
  header) shows the version, its date and a short title; a click opens
  one plain paragraph about what changed for users and a "Want to know
  more?" link to that version's GitHub release page
  (`https://github.com/tracedraw/tracedraw/releases/tag/v<version>`),
  opened in the system browser (a new tab in the browser version). The
  newest row is open at first, the others closed.
- The rows are `RELEASES` in `ui/welcome.rs` (version, date, title key,
  paragraph key); the texts are `welcome.news.v<major>_<minor>_title`
  and `_body` in the language files. Each release adds its row on top and
  drops the oldest (`MAINTAINING.md`).
- Help > Mark New Features (a check item, off at start, not kept
  between sessions) marks what is new in this version in the menus: a
  small accent dot at the end of each new row, left of the arrow of a
  submenu leading to one, and over the title of a menu leading to one.
  The rows are listed in `NEW_IN_THIS_VERSION` (`ui/menus.rs`); each
  version replaces the list with its own.
- In this version: View > Scrollbars, View > Page > Page Shadow,
  Bitmaps > Crop to Frame.

## Checks

- `ui::welcome::tests::the_newest_release_is_this_version` (the top row
  is the version being built)
- `ui::welcome::tests::the_history_holds_ten_releases_with_their_texts`
- `ui::welcome::tests::each_release_links_to_its_own_page`
- `ui::welcome::tests::help_whats_new_opens_the_history`
- `ui::welcome::tests::a_click_opens_a_closed_release`

- `ui::menus::tests::menu_commands_are_listed_and_run_by_key` (every
  marked row is in the menus; marks show only while the item is on)
