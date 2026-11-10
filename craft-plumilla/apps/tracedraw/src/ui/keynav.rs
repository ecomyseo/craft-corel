//! Keyboard navigation of menus and lists (#1839): the menu bar, every
//! menu and submenu, the right-click menus and the drop-down lists.
//!
//! One implementation serves them all. At the start of a frame,
//! [`frame`] finds the innermost open pop-up (a menu, a submenu, a
//! context menu, a combo box's list) and moves its keyboard highlight
//! with the keys, before any other part of the window sees them: the
//! arrows never nudge the selection while a menu is open. The highlight
//! is egui's keyboard focus on a row of the pop-up: menu rows draw it
//! like the hover box, list rows like a pressed row. Enter and Space
//! click the highlighted row (an accessibility click request, the click
//! egui gives a focused widget), Esc closes one level.
//!
//! The rows are the pop-up's clickable widgets of the last frame, in
//! reading order; disabled rows and separators sense no click and are
//! skipped. Menu rows say which of them open a submenu
//! ([`note_menu_row`]); the menu bar names its titles ([`note_titles`]).
//! A pop-up that handles its own keys (the font list, whose field takes
//! the typing) says so ([`own_keys`]).
//!
//! Alt pressed and released alone highlights the menu bar's first title;
//! Left and Right then move along the bar, Down, Enter or Space open the
//! menu, a title's access key opens its menu, Esc or Alt leaves the bar.

use egui::{Context, Event, Id, Key, LayerId, Modifiers, Order, Rect};

/// How long typed letters add up to one name in a list.
const TYPE_AHEAD_SECONDS: f64 = 1.0;

/// The rows a page key moves by when the pop-up's height is unknown.
const PAGE_ROWS: usize = 10;

fn state_id() -> Id {
    Id::new("keynav_state")
}

fn titles_id() -> Id {
    Id::new("keynav_titles")
}

fn rows_id() -> Id {
    Id::new("keynav_menu_rows")
}

fn own_id() -> Id {
    Id::new("keynav_own_keys")
}

fn combos_id() -> Id {
    Id::new("keynav_combos")
}

/// What the navigation keeps between frames.
#[derive(Clone, Default, Debug)]
struct State {
    /// Alt is held.
    alt_down: bool,
    /// Nothing but Alt was pressed since Alt went down.
    alt_alone: bool,
    /// The menu bar has the keyboard (no menu open): the title highlighted.
    bar: Option<usize>,
    /// The pop-up whose first row takes the highlight once it is shown.
    first: Option<Id>,
    /// The list whose chosen row takes the highlight once it is shown.
    chosen: Option<Id>,
    /// The row to scroll into view this frame.
    scroll: Option<Id>,
    /// The keys moved the highlight last: the hover box stays off until
    /// the pointer moves.
    keyboard: bool,
    /// Letters typed in a list, and when the last one was.
    typed: String,
    typed_at: f64,
    /// The innermost pop-up the keys went to last frame.
    active: Option<Id>,
    /// The list pop-up to close once this frame drew the click on it.
    close: Option<Id>,
    /// The names of the active list's rows, read last frame.
    labels: Option<(Id, Vec<(Id, String, bool)>)>,
    /// A menu and its row whose submenu opens next frame as Right would
    /// open it (Alt and the letter of a menu under the menu bar's »).
    open_sub: Option<(Id, Id)>,
}

/// The menu bar's titles of the last frame: each menu's pop-up id and
/// its access key.
#[derive(Clone, Default, Debug)]
pub struct Titles {
    pub popups: Vec<Id>,
    pub keys: Vec<char>,
}

/// Ids noted during a frame, with the frame they were noted in.
#[derive(Clone, Debug)]
struct Noted<T: Clone> {
    pass: u64,
    last: Vec<T>,
    this: Vec<T>,
}

impl<T: Clone> Default for Noted<T> {
    fn default() -> Self {
        Self {
            pass: 0,
            last: Vec::new(),
            this: Vec::new(),
        }
    }
}

impl<T: Clone> Noted<T> {
    /// What was noted in the frame before `pass` (and so far in it).
    fn add(&mut self, pass: u64, item: T) {
        self.roll(pass);
        self.this.push(item);
    }

    fn roll(&mut self, pass: u64) {
        if self.pass != pass {
            self.last = if self.pass + 1 == pass {
                std::mem::take(&mut self.this)
            } else {
                Vec::new()
            };
            self.this.clear();
            self.pass = pass;
        }
    }

    fn seen(&mut self, pass: u64) -> impl Iterator<Item = &T> {
        self.roll(pass);
        self.last.iter().chain(self.this.iter())
    }
}

/// A combo box of the frame: its list's pop-up id, its button and the
/// name it shows.
#[derive(Clone, Debug)]
struct Combo {
    popup: Id,
    button: Id,
    shown: String,
}

fn load(ctx: &Context) -> State {
    ctx.data(|d| d.get_temp::<State>(state_id()))
        .unwrap_or_default()
}

fn store(ctx: &Context, st: State) {
    ctx.data_mut(|d| d.insert_temp(state_id(), st));
}

/// A menu row was drawn this frame; `submenu` when it opens one.
pub fn note_menu_row(ctx: &Context, id: Id, submenu: bool) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Noted<(Id, bool)>>(rows_id())
            .add(pass, (id, submenu))
    });
}

/// The menu bar's titles, in order, with their menus' pop-up ids.
pub fn note_titles(ctx: &Context, titles: Titles) {
    ctx.data_mut(|d| d.insert_temp(titles_id(), titles));
}

/// The pop-up `id` reads the keys itself this frame (a field in it has
/// the keyboard): the navigation leaves it alone.
pub fn own_keys(ctx: &Context, id: Id) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Noted<Id>>(own_id())
            .add(pass, id)
    });
}

fn note_combo(ctx: &Context, combo: Combo) {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Noted<Combo>>(combos_id())
            .add(pass, combo)
    });
}

fn combo_of(ctx: &Context, popup: Id) -> Option<Combo> {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Noted<Combo>>(combos_id())
            .seen(pass)
            .find(|c| c.popup == popup)
            .cloned()
    })
}

fn menu_rows(ctx: &Context) -> Vec<(Id, bool)> {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Noted<(Id, bool)>>(rows_id())
            .seen(pass)
            .copied()
            .collect()
    })
}

fn owns_keys(ctx: &Context, id: Id) -> bool {
    let pass = ctx.cumulative_pass_nr();
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<Noted<Id>>(own_id())
            .seen(pass)
            .any(|i| *i == id)
    })
}

/// The menu bar title that has the keyboard, if any.
pub fn bar_focus(ctx: &Context) -> Option<usize> {
    load(ctx).bar
}

/// The menu bar title whose menu is open (as of the last frame's titles).
pub fn open_title(ctx: &Context) -> Option<usize> {
    let titles: Titles = ctx.data(|d| d.get_temp::<Titles>(titles_id()))?;
    titles
        .popups
        .iter()
        .position(|p| egui::Popup::is_id_open(ctx, *p))
}

/// The keys moved a menu's highlight last (the pointer has not moved
/// since): hovered rows draw no hover box then, so one row is lit.
pub fn keyboard_mode(ctx: &Context) -> bool {
    load(ctx).keyboard
}

/// The menu bar or an open pop-up takes the keys this frame: the drawing
/// interprets none of them.
pub fn takes_keys(ctx: &Context) -> bool {
    load(ctx).bar.is_some() || egui::Popup::is_any_open(ctx)
}

/// The menu `popup` was opened from the keyboard: its first row takes
/// the highlight once it is shown.
pub fn opened_by_keys(ctx: &Context, popup: Id) {
    let mut st = load(ctx);
    st.first = Some(popup);
    st.keyboard = true;
    st.bar = None;
    store(ctx, st);
    ctx.request_repaint();
}

/// The submenu of the row `row` of the open menu `menu` opens next
/// frame, its first row lit, as the Right key opens it.
pub fn open_submenu_by_keys(ctx: &Context, menu: Id, row: Id) {
    let mut st = load(ctx);
    st.open_sub = Some((menu, row));
    st.bar = None;
    store(ctx, st);
    ctx.request_repaint();
}

/// The list `popup` was opened from the keyboard: its chosen row takes
/// the highlight once it is shown.
pub fn list_opened_by_keys(ctx: &Context, popup: Id) {
    let mut st = load(ctx);
    st.chosen = Some(popup);
    st.keyboard = true;
    store(ctx, st);
    ctx.request_repaint();
}

/// Scroll the highlighted row into view: called inside a scrolling
/// pop-up body after its rows (menus, combo boxes and the lists of the
/// bars do).
pub fn scroll_into_view(ui: &egui::Ui) {
    let ctx = ui.ctx();
    let mut st = load(ctx);
    let Some(id) = st.scroll else {
        return;
    };
    let rect = ctx.viewport(|v| {
        v.this_pass
            .widgets
            .get(id)
            .filter(|w| w.layer_id == ui.layer_id())
            .map(|w| w.rect)
    });
    if let Some(rect) = rect {
        // At once, not animated: a key held down moves row after row,
        // and the list keeps up with it.
        ui.scroll_to_rect_animation(rect, None, egui::style::ScrollAnimation::none());
        st.scroll = None;
        store(ctx, st);
    }
}

/// Take a key press with exactly `mods`.
fn take(ctx: &Context, mods: Modifiers, key: Key) -> bool {
    let taken = ctx.input_mut(|i| i.consume_key(mods, key));
    if taken
        && matches!(
            key,
            Key::ArrowDown | Key::ArrowUp | Key::ArrowLeft | Key::ArrowRight
        )
    {
        // egui's own move of the keyboard focus with this arrow (to the
        // nearest widget that way) is called off: the key was used here.
        ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
    }
    taken
}

fn take_plain(ctx: &Context, key: Key) -> bool {
    take(ctx, Modifiers::NONE, key)
}

/// Click the widget `id` this frame, as Enter or Space on a focused
/// button does.
fn click(ctx: &Context, id: Id) {
    ctx.input_mut(|i| {
        i.events.push(Event::AccessKitActionRequest(
            egui::accesskit::ActionRequest {
                action: egui::accesskit::Action::Click,
                target_node: id.accesskit_id(),
                target_tree: egui::accesskit::TreeId::ROOT,
                data: None,
            },
        ))
    });
}

fn layer(popup: Id) -> LayerId {
    LayerId::new(Order::Foreground, popup)
}

fn has_rows(ctx: &Context, popup: Id) -> bool {
    ctx.viewport(|v| v.prev_pass.widgets.get_layer(layer(popup)).next().is_some())
}

/// The open pop-ups from the outermost to the innermost: the topmost
/// open pop-up, then the submenus open in it. Empty when none is open.
fn open_chain(ctx: &Context) -> Vec<Id> {
    let layers: Vec<LayerId> = ctx.memory(|m| m.layer_ids().collect());
    let root = layers
        .into_iter()
        .rev()
        .find(|l| l.order == Order::Foreground && egui::Popup::is_id_open(ctx, l.id));
    let Some(root) = root else {
        return Vec::new();
    };
    let mut chain = vec![root.id];
    while chain.len() < 16 {
        let Some(last) = chain.last().copied() else {
            break;
        };
        let next = egui::containers::menu::MenuState::from_id(ctx, last, |s| s.open_item);
        match next {
            Some(sub) if has_rows(ctx, sub) && !chain.contains(&sub) => chain.push(sub),
            _ => break,
        }
    }
    chain
}

/// The rows of the pop-up `popup` as last drawn: its enabled widgets that
/// take a click and no drag (buttons, list and menu rows; not fields or
/// sliders), in reading order. A widget holding others (the pop-up's own
/// area, a frame that senses clicks) is left out; so are all while the
/// pop-up's first frame lays it out unseen (its rows are disabled then).
fn rows_of(ctx: &Context, popup: Id) -> Vec<(Id, Rect)> {
    let widgets: Vec<egui::WidgetRect> = ctx.viewport(|v| {
        v.prev_pass
            .widgets
            .get_layer(layer(popup))
            .copied()
            .collect()
    });
    rows_among(&widgets, popup)
}

/// [`rows_of`] among `widgets`.
fn rows_among(widgets: &[egui::WidgetRect], popup: Id) -> Vec<(Id, Rect)> {
    let holds_others = |w: &egui::WidgetRect| {
        widgets.iter().any(|o| {
            o.id != w.id
                && o.rect.is_positive()
                && w.rect.contains_rect(o.rect)
                && w.rect.area() > o.rect.area()
        })
    };
    // The pop-up's own area is never a row, even where its rows lie
    // outside it (a submenu that opens to the left of a narrow window is
    // first laid out to the right, #1839).
    let area = popup.with("move");
    let mut rows: Vec<(Id, Rect)> = widgets
        .iter()
        .filter(|w| {
            w.id != popup
                && w.id != area
                && w.enabled
                && w.sense.senses_click()
                && !w.sense.senses_drag()
                && w.rect.is_positive()
                && !holds_others(w)
        })
        .map(|w| (w.id, w.rect))
        .collect();
    rows.sort_by(|a, b| {
        a.1.top()
            .total_cmp(&b.1.top())
            .then(a.1.left().total_cmp(&b.1.left()))
    });
    rows.dedup_by_key(|r| r.0);
    rows
}

/// Whether the rows stand in one column (a menu or a list) rather than in
/// a grid.
fn one_column(rows: &[(Id, Rect)]) -> bool {
    rows.windows(2)
        .all(|w| w[1].1.center().y > w[0].1.bottom() - 1.0)
}

/// The row below (`down`) or above row `at` in a grid: the nearest row
/// of the next line, closest in x.
fn grid_step(rows: &[(Id, Rect)], at: usize, down: bool) -> Option<usize> {
    let here = rows.get(at)?.1;
    let beyond = |r: &Rect| {
        if down {
            r.center().y > here.bottom() - 1.0
        } else {
            r.center().y < here.top() + 1.0
        }
    };
    let line = rows
        .iter()
        .filter(|(_, r)| beyond(r))
        .map(|(_, r)| (r.center().y - here.center().y).abs())
        .min_by(f32::total_cmp)?;
    rows.iter()
        .enumerate()
        .filter(|(_, (_, r))| beyond(r) && (r.center().y - here.center().y).abs() < line + 2.0)
        .min_by(|a, b| {
            let da = (a.1 .1.center().x - here.center().x).abs();
            let db = (b.1 .1.center().x - here.center().x).abs();
            da.total_cmp(&db)
        })
        .map(|(i, _)| i)
}

/// The row a navigation key moves to from `at` among `n` rows; `wrap`
/// (menus) goes round from the last row to the first and back.
pub fn step(n: usize, at: Option<usize>, key: Key, page: usize, wrap: bool) -> Option<usize> {
    if n == 0 {
        return None;
    }
    let last = n - 1;
    let page = page.max(1);
    Some(match (key, at) {
        (Key::Home, _) => 0,
        (Key::End, _) => last,
        (Key::ArrowDown, None) => 0,
        (Key::ArrowUp, None) => last,
        (Key::ArrowDown, Some(i)) if i >= last => {
            if wrap {
                0
            } else {
                last
            }
        }
        (Key::ArrowDown, Some(i)) => i + 1,
        (Key::ArrowUp, Some(0)) => {
            if wrap {
                last
            } else {
                0
            }
        }
        (Key::ArrowUp, Some(i)) => i - 1,
        (Key::PageDown, None) => page.min(last),
        (Key::PageDown, Some(i)) => (i + page).min(last),
        (Key::PageUp, None) => 0,
        (Key::PageUp, Some(i)) => i.saturating_sub(page),
        _ => return at,
    })
}

/// The row typing `typed` goes to among `labels` from row `at`: a letter
/// typed again and again goes round the names starting with it; more
/// letters find the first name starting with them all. Case is ignored.
pub fn type_ahead(labels: &[String], at: Option<usize>, typed: &str) -> Option<usize> {
    let typed = typed.to_lowercase();
    let mut chars = typed.chars();
    let first = chars.next()?;
    let lower: Vec<String> = labels.iter().map(|l| l.trim().to_lowercase()).collect();
    let n = lower.len();
    if chars.all(|c| c == first) {
        // One letter (or the same one again): the next name with it.
        let start = at.map_or(0, |i| i + 1);
        let found = (0..n)
            .map(|k| (start + k) % n)
            .find(|&i| lower[i].starts_with(first));
        // Typed twice, a name may still start with both ("oo").
        if typed.chars().count() > 1 {
            if let Some(i) = at.filter(|&i| lower[i].starts_with(&typed)) {
                return Some(i);
            }
        }
        return found;
    }
    let from = at.unwrap_or(0);
    (0..n)
        .map(|k| (from + k) % n)
        .find(|&i| lower[i].starts_with(&typed))
}

/// The keys of this frame for the menu bar and the open pop-up; called
/// once a frame before anything else reads the keyboard. `alt_blocked`
/// keeps Alt alone from highlighting the bar (Options waiting for a
/// shortcut's keys).
pub fn frame(ctx: &Context, alt_blocked: bool) {
    let mut st = load(ctx);
    let (pointer_moved, pointer_pressed) = ctx.input(|i| {
        (
            i.pointer.delta() != egui::Vec2::ZERO,
            i.pointer.any_pressed(),
        )
    });
    if pointer_moved || pointer_pressed {
        st.keyboard = false;
    }
    if pointer_pressed {
        st.bar = None;
    }
    let titles: Titles = ctx
        .data(|d| d.get_temp::<Titles>(titles_id()))
        .unwrap_or_default();
    if titles.popups.is_empty() {
        st.bar = None;
    }
    if let Some((menu, row)) = st.open_sub.take() {
        open_submenu(ctx, &mut st, menu, row);
    }
    let chain = open_chain(ctx);
    if alt_alone(ctx, &mut st) && !alt_blocked {
        if !chain.is_empty() {
            egui::Popup::close_all(ctx);
            st.bar = None;
        } else if st.bar.is_some() {
            st.bar = None;
        } else if !titles.popups.is_empty() {
            st.bar = Some(0);
            st.keyboard = true;
        }
    }
    if st.bar.is_some() && !chain.is_empty() {
        // A menu opened (with the pointer, a shortcut): the bar lets go.
        st.bar = None;
    }
    if let Some(at) = st.bar {
        bar_keys(ctx, &mut st, &titles, at);
    } else if let Some(deepest) = chain.last().copied() {
        let root = chain[0];
        if !owns_keys(ctx, root) {
            popup_keys(ctx, &mut st, &titles, &chain, root, deepest);
        }
    }
    st.active = chain.last().copied();
    store(ctx, st);
}

/// Alt went down and up with nothing else pressed in between. The
/// changes of the modifiers are followed event by event: a quick tap can
/// press and release Alt within one frame.
fn alt_alone(ctx: &Context, st: &mut State) -> bool {
    let events = ctx.input(|i| i.events.clone());
    let mut fired = false;
    for e in &events {
        match e {
            Event::ModifiersChanged(m) => {
                let others = m.ctrl || m.shift || m.command || m.mac_cmd;
                if m.alt && !st.alt_down {
                    st.alt_alone = !others;
                } else if !m.alt && st.alt_down {
                    fired ^= st.alt_alone && !others;
                    st.alt_alone = false;
                } else if others {
                    st.alt_alone = false;
                }
                st.alt_down = m.alt;
            }
            Event::Key { pressed: true, .. }
            | Event::PointerButton { pressed: true, .. }
            | Event::MouseWheel { .. }
            | Event::Text(_)
            | Event::WindowFocused(_) => st.alt_alone = false,
            _ => {}
        }
    }
    fired
}

/// The keys while the menu bar is highlighted and no menu is open.
fn bar_keys(ctx: &Context, st: &mut State, titles: &Titles, at: usize) {
    let n = titles.popups.len();
    let at = at.min(n.saturating_sub(1));
    st.bar = Some(at);
    let open = |st: &mut State, i: usize| {
        if let Some(p) = titles.popups.get(i).copied() {
            egui::Popup::open_id(ctx, p);
            st.first = Some(p);
            st.keyboard = true;
            st.bar = None;
            ctx.request_repaint();
        }
    };
    if take_plain(ctx, Key::ArrowRight) {
        st.bar = Some((at + 1) % n);
    } else if take_plain(ctx, Key::ArrowLeft) {
        st.bar = Some((at + n - 1) % n);
    } else if take_plain(ctx, Key::Home) {
        st.bar = Some(0);
    } else if take_plain(ctx, Key::End) {
        st.bar = Some(n - 1);
    } else if take_plain(ctx, Key::ArrowDown)
        || take_plain(ctx, Key::ArrowUp)
        || take_plain(ctx, Key::Enter)
        || take_plain(ctx, Key::Space)
    {
        open(st, at);
    } else if take_plain(ctx, Key::Escape) {
        st.bar = None;
    } else {
        // A title's access key, without Alt, opens its menu.
        let hit = titles.keys.iter().enumerate().find(|(_, c)| {
            crate::ui::menus::access::key_of(**c).is_some_and(|k| {
                ctx.input(|i| i.key_pressed(k)) && !ctx.input(|i| i.modifiers.any())
            })
        });
        if let Some((i, c)) = hit {
            if let Some(k) = crate::ui::menus::access::key_of(*c) {
                take_plain(ctx, k);
            }
            open(st, i);
        } else if ctx.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, Event::Key { pressed: true, .. }))
        }) {
            // Any other key (a shortcut) leaves the bar and does its work.
            st.bar = None;
        }
    }
}

/// The keys while a pop-up is open: `chain` from the outermost pop-up
/// (`root`) to the innermost (`deepest`), which takes them.
fn popup_keys(ctx: &Context, st: &mut State, titles: &Titles, chain: &[Id], root: Id, deepest: Id) {
    // Another pop-up than last frame: the letters typed start again.
    if st.active != Some(deepest) {
        st.typed.clear();
    }
    // Nothing to do in a frame without keys (the rows are looked at only
    // when a key comes, so an open list costs nothing a frame).
    let pending = st.first == Some(deepest) || st.chosen == Some(deepest);
    let keyed = ctx.input(|i| {
        i.events
            .iter()
            .any(|e| matches!(e, Event::Key { pressed: true, .. } | Event::Text(_)))
    });
    if !pending && !keyed {
        return;
    }
    // A field or a slider of the pop-up with the keyboard keeps it.
    let focus = ctx.memory(|m| m.focused());
    if ctx.text_edit_focused() {
        return;
    }
    let rows = rows_of(ctx, deepest);
    if let Some(f) = focus {
        let in_popup = ctx.viewport(|v| {
            v.prev_pass
                .widgets
                .get(f)
                .is_some_and(|w| w.layer_id == layer(deepest))
        });
        if in_popup && !rows.iter().any(|(id, _)| *id == f) {
            return;
        }
    }
    let menu_rows = menu_rows(ctx);
    let is_menu = rows
        .iter()
        .any(|(id, _)| menu_rows.iter().any(|(m, _)| m == id));
    let submenu_of = |id: Id| menu_rows.iter().any(|(m, s)| *m == id && *s);
    let bar_index = titles.popups.iter().position(|p| *p == root);
    let mut at = focus.and_then(|f| rows.iter().position(|(id, _)| *id == f));
    let mut moved: Option<usize> = None;

    if st.first == Some(deepest) {
        if !rows.is_empty() {
            moved = Some(0);
            st.first = None;
        } else {
            ctx.request_repaint();
        }
    }
    let labels: Vec<(Id, String, bool)> = match &st.labels {
        Some((l, labels)) if *l == deepest => labels.clone(),
        _ => Vec::new(),
    };
    let label_at = |id: Id| labels.iter().position(|(l, _, _)| *l == id);
    // The row the list shows chosen, where the keys start from.
    let chosen = labels
        .iter()
        .find(|(_, _, c)| *c)
        .and_then(|(id, _, _)| rows.iter().position(|(r, _)| r == id));
    if st.chosen == Some(deepest) {
        if !labels.is_empty() && !rows.is_empty() {
            moved = Some(chosen.unwrap_or(0));
            st.chosen = None;
        } else {
            ctx.request_repaint();
        }
    }
    if at.is_none() && !is_menu {
        at = chosen;
    }

    let page = ctx
        .memory(|m| m.area_rect(deepest))
        .map(|area| {
            rows.iter()
                .filter(|(_, r)| area.contains_rect(*r))
                .count()
                .saturating_sub(1)
        })
        .filter(|p| *p > 0)
        .unwrap_or(PAGE_ROWS);
    let column = one_column(&rows);
    for key in [
        Key::ArrowDown,
        Key::ArrowUp,
        Key::Home,
        Key::End,
        Key::PageDown,
        Key::PageUp,
    ] {
        if take_plain(ctx, key) {
            // With no row highlighted yet, a list's keys start from its
            // chosen row (Down goes to the one after it).
            let from = moved.or(at);
            let to = if column || !matches!(key, Key::ArrowDown | Key::ArrowUp) {
                step(rows.len(), from, key, page, is_menu)
            } else {
                match from {
                    Some(i) => grid_step(&rows, i, key == Key::ArrowDown).or(Some(i)),
                    None => step(rows.len(), None, key, page, false),
                }
            };
            if to.is_some() {
                moved = to;
            }
        }
    }
    let current = moved.or(
        if focus.is_some_and(|f| rows.iter().any(|(id, _)| *id == f)) {
            at
        } else {
            None
        },
    );
    let current_id = current.and_then(|i| rows.get(i)).map(|r| r.0);

    // Right: into a submenu, else the next menu of the bar.
    if take_plain(ctx, Key::ArrowRight) {
        if let Some(id) = current_id.filter(|id| submenu_of(*id)) {
            open_submenu(ctx, st, deepest, id);
        } else if !column && !is_menu {
            moved = Some(current.map_or(0, |i| (i + 1).min(rows.len().saturating_sub(1))));
        } else if let Some(b) = bar_index {
            switch_menu(ctx, st, titles, b, true);
            return;
        }
    }
    // Left: out of a submenu, else the menu before on the bar.
    if take_plain(ctx, Key::ArrowLeft) {
        if chain.len() > 1 {
            close_submenu(ctx, st, chain, &menu_rows);
            return;
        } else if !column && !is_menu {
            moved = Some(current.map_or(0, |i| i.saturating_sub(1)));
        } else if let Some(b) = bar_index {
            switch_menu(ctx, st, titles, b, false);
            return;
        }
    }
    // Esc: one level closes.
    if take_plain(ctx, Key::Escape) {
        if chain.len() > 1 {
            close_submenu(ctx, st, chain, &menu_rows);
        } else {
            egui::Popup::close_id(ctx, root);
            give_focus_back(ctx, root);
            if let Some(b) = bar_index {
                st.bar = Some(b);
                st.keyboard = true;
            }
        }
        return;
    }
    // Enter and Space: the row's click, or its submenu.
    if take_plain(ctx, Key::Enter) | take_plain(ctx, Key::Space) {
        if let Some(id) = current_id {
            if submenu_of(id) {
                open_submenu(ctx, st, deepest, id);
            } else {
                click(ctx, id);
                if !is_menu {
                    // A list closes once the row took the click.
                    st.close = Some(root);
                }
            }
        }
        ctx.request_repaint();
    }
    // Letters in a list: the first name starting with them. (In a menu
    // they are the rows' access keys.)
    if !is_menu && !labels.is_empty() {
        let text: String = ctx.input_mut(|i| {
            let mut text = String::new();
            if !(i.modifiers.ctrl || i.modifiers.alt || i.modifiers.command) {
                i.events.retain(|e| match e {
                    Event::Text(t) => {
                        text.push_str(t);
                        false
                    }
                    _ => true,
                });
            }
            text
        });
        if !text.is_empty() {
            let now = ctx.input(|i| i.time);
            if now - st.typed_at > TYPE_AHEAD_SECONDS {
                st.typed.clear();
            }
            st.typed_at = now;
            st.typed.push_str(&text);
            let names: Vec<String> = rows
                .iter()
                .map(|(id, _)| {
                    label_at(*id)
                        .and_then(|i| labels.get(i))
                        .map(|l| l.1.clone())
                        .unwrap_or_default()
                })
                .collect();
            let start = current.or(chosen);
            if let Some(i) = type_ahead(&names, start, &st.typed) {
                moved = Some(i);
            }
            // The letters typed are not shortcuts for the drawing.
            for c in text.chars() {
                if let Some(k) = crate::ui::menus::access::key_of(c) {
                    ctx.input_mut(|i| {
                        i.events
                            .retain(|e| !matches!(e, Event::Key { key, .. } if *key == k))
                    });
                }
            }
        }
    }
    if let Some(i) = moved {
        if let Some((id, _)) = rows.get(i) {
            ctx.memory_mut(|m| {
                m.request_focus(*id);
                // egui's own move of the focus with the arrows is called
                // off: the keys were taken here.
                m.move_focus(egui::FocusDirection::None);
            });
            st.scroll = Some(*id);
            st.keyboard = true;
            ctx.request_repaint();
        }
    }
}

/// Open the submenu of the row `row` of the pop-up `menu`, its first row
/// highlighted.
fn open_submenu(ctx: &Context, st: &mut State, menu: Id, row: Id) {
    let sub = egui::containers::menu::SubMenu::id_from_widget_id(row);
    egui::containers::menu::MenuState::from_id(ctx, menu, |s| s.open_item = Some(sub));
    // Counted as shown, or egui closes it again before it is drawn.
    egui::containers::menu::MenuState::mark_shown(ctx, sub);
    st.first = Some(sub);
    st.keyboard = true;
    ctx.request_repaint();
}

/// Close the innermost submenu of `chain`; its row keeps the highlight.
fn close_submenu(ctx: &Context, st: &mut State, chain: &[Id], rows: &[(Id, bool)]) {
    let n = chain.len();
    if n < 2 {
        return;
    }
    let (parent, sub) = (chain[n - 2], chain[n - 1]);
    egui::containers::menu::MenuState::from_id(ctx, parent, |s| s.open_item = None);
    if let Some((row, _)) = rows
        .iter()
        .find(|(r, s)| *s && egui::containers::menu::SubMenu::id_from_widget_id(*r) == sub)
    {
        ctx.memory_mut(|m| m.request_focus(*row));
    }
    st.keyboard = true;
    ctx.request_repaint();
}

/// The menu after (`next`) or before the bar's menu `at`, opened in its
/// place with its first row highlighted.
fn switch_menu(ctx: &Context, st: &mut State, titles: &Titles, at: usize, next: bool) {
    let n = titles.popups.len();
    if n == 0 {
        return;
    }
    let to = if next { (at + 1) % n } else { (at + n - 1) % n };
    if let Some(p) = titles.popups.get(to).copied() {
        egui::Popup::open_id(ctx, p);
        st.first = Some(p);
        st.keyboard = true;
        ctx.request_repaint();
    }
}

/// A list closed from the keyboard gives the keyboard back to its combo
/// box.
fn give_focus_back(ctx: &Context, popup: Id) {
    if let Some(c) = combo_of(ctx, popup) {
        ctx.memory_mut(|m| m.request_focus(c.button));
    }
}

/// The end of a frame: a list picked from with Enter closes, and the
/// names of the open list's rows are read for typing ahead.
pub fn end_frame(ctx: &Context) {
    let mut st = load(ctx);
    if let Some(p) = st.close.take() {
        egui::Popup::close_id(ctx, p);
        give_focus_back(ctx, p);
    }
    let active = open_chain(ctx).last().copied();
    let keyed = ctx.input(|i| {
        i.events
            .iter()
            .any(|e| matches!(e, Event::Key { pressed: true, .. } | Event::Text(_)))
    });
    let known = st.labels.as_ref().map(|l| l.0) == active;
    // The names are read when a pop-up opens and after each key, not in
    // every frame it stays open.
    let read = active.is_some() && (!known || keyed || st.chosen.is_some() || st.first.is_some());
    if !known {
        st.labels = None;
    }
    if let Some(active) = active.filter(|_| read) {
        let widgets: Vec<egui::WidgetRect> = ctx.viewport(|v| {
            v.this_pass
                .widgets
                .get_layer(layer(active))
                .copied()
                .collect()
        });
        let rows = rows_among(&widgets, active);
        let is_menu = {
            let menu_rows = menu_rows(ctx);
            rows.iter()
                .any(|(id, _)| menu_rows.iter().any(|(m, _)| m == id))
        };
        if !rows.is_empty() {
            let shown = combo_of(ctx, active).map(|c| c.shown);
            let mut labels = row_labels(ctx, active, &rows, shown.as_deref());
            if is_menu {
                // A menu has no chosen row.
                labels.iter_mut().for_each(|l| l.2 = false);
            }
            st.labels = Some((active, labels));
        }
    }
    store(ctx, st);
}

/// The name of the row with the keyboard highlight in the open pop-up,
/// as last drawn.
#[cfg(test)]
pub fn highlighted(ctx: &Context) -> Option<String> {
    let focus = ctx.memory(|m| m.focused())?;
    let st = load(ctx);
    let (_, labels) = st.labels?;
    labels
        .into_iter()
        .find(|(id, _, _)| *id == focus)
        .map(|(_, name, _)| name)
}

/// The text drawn in each row of the pop-up `popup` this frame, and
/// which row shows as chosen: the one named like the combo box's
/// `shown` value, else the one filled with the selection colour.
fn row_labels(
    ctx: &Context,
    popup: Id,
    rows: &[(Id, Rect)],
    shown: Option<&str>,
) -> Vec<(Id, String, bool)> {
    let mut texts: Vec<(egui::Pos2, String)> = Vec::new();
    let mut fills: Vec<Rect> = Vec::new();
    let selection = ctx.global_style().visuals.selection.bg_fill;
    fn walk(
        shape: &egui::Shape,
        texts: &mut Vec<(egui::Pos2, String)>,
        fills: &mut Vec<Rect>,
        selection: egui::Color32,
    ) {
        match shape {
            egui::Shape::Text(t) => {
                let c = t.pos + t.galley.rect.center().to_vec2();
                texts.push((c, t.galley.text().to_string()));
            }
            egui::Shape::Rect(r) if r.fill == selection => fills.push(r.rect),
            egui::Shape::Vec(v) => {
                for s in v {
                    walk(s, texts, fills, selection);
                }
            }
            _ => {}
        }
    }
    ctx.graphics(|g| {
        if let Some(list) = g.get(layer(popup)) {
            for s in list.all_entries() {
                walk(&s.shape, &mut texts, &mut fills, selection);
            }
        }
    });
    let mut out: Vec<(Id, String, bool)> = rows
        .iter()
        .map(|(id, r)| {
            let mut parts: Vec<&(egui::Pos2, String)> =
                texts.iter().filter(|(p, _)| r.contains(*p)).collect();
            parts.sort_by(|a, b| a.0.x.total_cmp(&b.0.x));
            let name = parts
                .iter()
                .map(|(_, t)| t.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            (*id, name, false)
        })
        .collect();
    let by_name = shown
        .filter(|s| !s.is_empty())
        .and_then(|s| out.iter().position(|(_, n, _)| n.trim() == s.trim()));
    let by_fill = || {
        rows.iter()
            .position(|(_, r)| fills.iter().any(|f| f.contains(r.center())))
    };
    if let Some(i) = by_name.or_else(by_fill) {
        if let Some(o) = out.get_mut(i) {
            o.2 = true;
        }
    }
    out
}

/// A combo box: egui's, with the keys of the desktop's lists. While its
/// list is open the arrows, Home, End and the page keys move the
/// highlight, letters jump to the first name starting with them, Enter
/// picks and Esc closes (see [`frame`]); the button with the keyboard
/// opens its list with Down, Up, Alt+Down, F4, Enter or Space, the
/// chosen row highlighted.
#[must_use = "You should call .show*"]
pub struct ComboBox {
    inner: egui::ComboBox,
    shown: String,
}

impl ComboBox {
    pub fn from_label(label: impl Into<egui::WidgetText>) -> Self {
        Self {
            inner: egui::ComboBox::from_label(label),
            shown: String::new(),
        }
    }

    pub fn from_id_salt(id_salt: impl egui::AsIdSalt) -> Self {
        Self {
            inner: egui::ComboBox::from_id_salt(id_salt),
            shown: String::new(),
        }
    }

    #[inline]
    pub fn width(mut self, width: f32) -> Self {
        self.inner = self.inner.width(width);
        self
    }

    #[inline]
    pub fn height(mut self, height: f32) -> Self {
        self.inner = self.inner.height(height);
        self
    }

    pub fn selected_text(mut self, text: impl Into<egui::WidgetText>) -> Self {
        let text = text.into();
        self.shown = text.text().to_string();
        self.inner = self.inner.selected_text(text);
        self
    }

    pub fn show_ui<R>(
        self,
        ui: &mut egui::Ui,
        add: impl FnOnce(&mut egui::Ui) -> R,
    ) -> egui::InnerResponse<Option<R>> {
        let Self { inner, shown } = self;
        let ir = inner.show_ui(ui, |ui| {
            let r = add(ui);
            scroll_into_view(ui);
            r
        });
        combo_keys(ui.ctx(), &ir.response, shown);
        ir
    }
}

/// The keys of a combo box's button: `button` is its response, `shown`
/// the name it shows.
fn combo_keys(ctx: &Context, button: &egui::Response, shown: String) {
    let popup = button.id.with("popup");
    note_combo(
        ctx,
        Combo {
            popup,
            button: button.id,
            shown,
        },
    );
    button_keys(ctx, button, popup);
}

/// The keys of the button of a drop-down list whose pop-up is `popup`
/// (a combo box, a field's arrow): while it has the keyboard and the
/// list is closed, Down, Up, Alt+Down and F4 open the list; Enter and
/// Space (the button's click) open it too. Either way the chosen row
/// takes the highlight.
pub fn list_button_keys(ctx: &Context, button: &egui::Response, popup: Id) {
    // Noted so that Esc and Enter give the keyboard back to the button.
    note_combo(
        ctx,
        Combo {
            popup,
            button: button.id,
            shown: String::new(),
        },
    );
    button_keys(ctx, button, popup);
}

/// [`list_button_keys`] for a button already noted.
fn button_keys(ctx: &Context, button: &egui::Response, popup: Id) {
    let open = egui::Popup::is_id_open(ctx, popup);
    if button.clicked() && !button.clicked_by(egui::PointerButton::Primary) && open {
        list_opened_by_keys(ctx, popup);
        return;
    }
    if open || !button.has_focus() {
        return;
    }
    let keys = take_plain(ctx, Key::ArrowDown)
        | take_plain(ctx, Key::ArrowUp)
        | take(ctx, Modifiers::ALT, Key::ArrowDown)
        | take_plain(ctx, Key::F4);
    if keys {
        egui::Popup::open_id(ctx, popup);
        ctx.memory_mut(|m| m.move_focus(egui::FocusDirection::None));
        list_opened_by_keys(ctx, popup);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stop_at_the_ends_of_a_list_and_go_round_a_menu() {
        assert_eq!(step(3, None, Key::ArrowDown, 1, false), Some(0));
        assert_eq!(step(3, None, Key::ArrowUp, 1, false), Some(2));
        assert_eq!(step(3, Some(2), Key::ArrowDown, 1, false), Some(2));
        assert_eq!(step(3, Some(2), Key::ArrowDown, 1, true), Some(0));
        assert_eq!(step(3, Some(0), Key::ArrowUp, 1, true), Some(2));
        assert_eq!(step(30, Some(3), Key::PageDown, 10, false), Some(13));
        assert_eq!(step(30, Some(25), Key::PageDown, 10, false), Some(29));
        assert_eq!(step(30, Some(5), Key::PageUp, 10, false), Some(0));
        assert_eq!(step(30, Some(5), Key::Home, 10, false), Some(0));
        assert_eq!(step(30, Some(5), Key::End, 10, false), Some(29));
        assert_eq!(step(0, None, Key::ArrowDown, 1, false), None);
    }

    #[test]
    fn typing_finds_names_and_a_letter_again_goes_round_them() {
        let names: Vec<String> = ["Arial", "Bahnschrift", "Book", "Bookman", "Courier"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(type_ahead(&names, None, "b"), Some(1));
        assert_eq!(type_ahead(&names, Some(1), "b"), Some(2));
        assert_eq!(type_ahead(&names, Some(3), "b"), Some(1));
        assert_eq!(type_ahead(&names, None, "bookm"), Some(3));
        assert_eq!(type_ahead(&names, None, "C"), Some(4));
        assert_eq!(type_ahead(&names, None, "z"), None);
        assert_eq!(type_ahead(&names, Some(2), "bo"), Some(2));
    }

    use crate::app::App;
    use crate::i18n::tr;
    use tracedraw_core::document::ShapeKind;
    use tracedraw_core::geometry::Rect as DocRect;

    fn press(key: Key, modifiers: Modifiers) -> Event {
        Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }

    fn typed(c: char) -> Vec<Event> {
        let mut out = Vec::new();
        if let Some(k) = crate::ui::menus::access::key_of(c) {
            out.push(press(k, Modifiers::NONE));
        }
        out.push(Event::Text(c.to_string()));
        out
    }

    /// One frame of the whole window, `mods` held.
    fn window(ctx: &Context, app: &mut App, events: Vec<Event>, mods: Modifiers) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1400.0, 900.0),
            )),
            events: std::iter::once(Event::ModifiersChanged(mods))
                .chain(events)
                .collect(),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| crate::ui::root(app, ui));
        out.textures_delta.clear();
    }

    fn keys(ctx: &Context, app: &mut App, events: Vec<Event>) {
        window(ctx, app, events, Modifiers::NONE);
        // The pop-ups the keys opened draw their rows.
        for _ in 0..3 {
            window(ctx, app, Vec::new(), Modifiers::NONE);
        }
    }

    fn key(ctx: &Context, app: &mut App, k: Key) {
        keys(ctx, app, vec![press(k, Modifiers::NONE)]);
    }

    fn lit(ctx: &Context) -> String {
        highlighted(ctx).unwrap_or_default()
    }

    /// Down until the highlighted row starts with `name` (at most `n`
    /// presses).
    fn down_to(ctx: &Context, app: &mut App, name: &str) {
        for _ in 0..60 {
            if lit(ctx).starts_with(name) {
                return;
            }
            key(ctx, app, Key::ArrowDown);
        }
        panic!("no row {name}, at {}", lit(ctx));
    }

    fn app_with_shapes() -> App {
        let mut app = App::headless();
        app.settings.show_new_document_dialog = true;
        let a = app
            .new_shape(ShapeKind::Rect {
                rect: DocRect::new(10.0, 10.0, 60.0, 40.0),
                radius: 0.0,
                corners: None,
            })
            .expect("rect");
        app.new_shape(ShapeKind::Ellipse {
            rect: DocRect::new(70.0, 10.0, 120.0, 40.0),
            arc: None,
        });
        app.select(vec![a]);
        app
    }

    fn titles(ctx: &Context) -> Titles {
        ctx.data(|d| d.get_temp::<Titles>(titles_id()))
            .unwrap_or_default()
    }

    fn open_menu(ctx: &Context) -> Option<usize> {
        titles(ctx)
            .popups
            .iter()
            .position(|p| egui::Popup::is_id_open(ctx, *p))
    }

    /// Alt pressed and let go alone lights the menu bar; the arrows walk
    /// the titles and the File menu's rows, Enter runs a row (#1839).
    #[test]
    fn alt_lights_the_menu_bar_and_the_arrows_walk_the_file_menu() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        keys(&ctx, &mut app, Vec::new());
        assert_eq!(bar_focus(&ctx), None);
        window(&ctx, &mut app, Vec::new(), Modifiers::ALT);
        window(&ctx, &mut app, Vec::new(), Modifiers::NONE);
        assert_eq!(bar_focus(&ctx), Some(0));
        key(&ctx, &mut app, Key::ArrowRight);
        assert_eq!(bar_focus(&ctx), Some(1));
        key(&ctx, &mut app, Key::ArrowLeft);
        key(&ctx, &mut app, Key::ArrowLeft);
        assert_eq!(bar_focus(&ctx), Some(titles(&ctx).popups.len() - 1));
        key(&ctx, &mut app, Key::Home);
        assert_eq!(bar_focus(&ctx), Some(0));
        // Down opens File, its first row lit.
        key(&ctx, &mut app, Key::ArrowDown);
        assert_eq!(open_menu(&ctx), Some(0));
        assert_eq!(bar_focus(&ctx), None);
        let new = tr("menu.file.new");
        assert!(lit(&ctx).starts_with(&new), "{}", lit(&ctx));
        key(&ctx, &mut app, Key::ArrowDown);
        assert!(lit(&ctx).starts_with(&tr("menu.file.new_from_template")));
        // Up from the first row goes round to the last; Home and End.
        key(&ctx, &mut app, Key::ArrowUp);
        key(&ctx, &mut app, Key::ArrowUp);
        let last = lit(&ctx);
        assert!(!last.starts_with(&new));
        key(&ctx, &mut app, Key::Home);
        assert!(lit(&ctx).starts_with(&new));
        key(&ctx, &mut app, Key::End);
        assert_eq!(lit(&ctx), last);
        // Right in a row with no submenu: the next menu, Left back.
        key(&ctx, &mut app, Key::ArrowRight);
        assert_eq!(open_menu(&ctx), Some(1));
        assert!(!lit(&ctx).is_empty());
        key(&ctx, &mut app, Key::ArrowLeft);
        assert_eq!(open_menu(&ctx), Some(0));
        assert!(lit(&ctx).starts_with(&new));
        // Esc closes the menu and leaves its title lit; Esc again lets
        // the bar go.
        key(&ctx, &mut app, Key::Escape);
        assert_eq!(open_menu(&ctx), None);
        assert_eq!(bar_focus(&ctx), Some(0));
        key(&ctx, &mut app, Key::Escape);
        assert_eq!(bar_focus(&ctx), None);
        // Alt+F opens File with its first row lit; Enter runs it.
        let f = titles(&ctx).keys[0];
        let fk = crate::ui::menus::access::key_of(f).expect("access key");
        keys(&ctx, &mut app, vec![press(fk, Modifiers::ALT)]);
        assert_eq!(open_menu(&ctx), Some(0));
        assert!(lit(&ctx).starts_with(&new));
        key(&ctx, &mut app, Key::Enter);
        assert!(matches!(
            app.dialog,
            crate::ui::dialogs::Dialog::NewDocument(_)
        ));
        assert!(!egui::Popup::is_any_open(&ctx));
    }

    /// On a window too narrow for the whole menu bar, Alt and the access
    /// key of a menu under » open » with that menu open in it, its first
    /// row lit, as on a wide bar.
    #[test]
    fn alt_and_a_letter_open_a_menu_under_more() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        let narrow = |ctx: &Context, app: &mut App, events: Vec<Event>| {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(560.0, 900.0),
                )),
                events,
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| crate::ui::root(app, ui));
            out.textures_delta.clear();
        };
        for _ in 0..4 {
            narrow(&ctx, &mut app, Vec::new());
        }
        let (_, more) = ctx
            .data(|d| d.get_temp::<(Vec<Rect>, usize)>(Id::new(crate::ui::menus::MENU_BAR_LAYOUT)))
            .expect("the menu bar was drawn");
        assert!(more >= 2, "the narrow bar puts menus under »");
        let more_keys = ctx
            .data(|d| d.get_temp::<Vec<char>>(Id::new(crate::ui::menus::MENU_MORE_KEYS)))
            .unwrap_or_default();
        assert_eq!(more_keys.len(), more);
        // Help is the last menu, under ».
        let help = *more_keys.last().expect("a key for Help");
        assert_ne!(help, ' ');
        assert!(!titles(&ctx).keys.contains(&help), "its own letter");
        let hk = crate::ui::menus::access::key_of(help).expect("access key");
        narrow(
            &ctx,
            &mut app,
            vec![
                Event::ModifiersChanged(Modifiers::ALT),
                press(hk, Modifiers::ALT),
            ],
        );
        for _ in 0..4 {
            narrow(
                &ctx,
                &mut app,
                vec![Event::ModifiersChanged(Modifiers::NONE)],
            );
        }
        let more_popup = *titles(&ctx).popups.last().expect("the » title");
        assert!(egui::Popup::is_id_open(&ctx, more_popup), "» opens");
        assert_eq!(open_chain(&ctx).len(), 2, "with Help open in it");
        let first = lit(&ctx);
        assert!(!first.is_empty(), "Help's first row is lit");
        assert!(
            !crate::ui::menus::all_menus()
                .iter()
                .any(|(k, _)| tr(k) == first),
            "a row of Help, not a menu under »: {first}"
        );
    }

    /// Right opens a submenu with its first row lit, Left closes it,
    /// Enter runs its row.
    #[test]
    fn the_arrows_open_and_close_a_submenu() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        keys(&ctx, &mut app, Vec::new());
        window(&ctx, &mut app, Vec::new(), Modifiers::ALT);
        window(&ctx, &mut app, Vec::new(), Modifiers::NONE);
        key(&ctx, &mut app, Key::ArrowRight);
        key(&ctx, &mut app, Key::Enter);
        assert_eq!(open_menu(&ctx), Some(1), "Edit");
        let select_all = tr("menu.edit.select_all");
        down_to(&ctx, &mut app, &select_all);
        assert_eq!(open_chain(&ctx).len(), 1);
        key(&ctx, &mut app, Key::ArrowRight);
        assert_eq!(open_chain(&ctx).len(), 2, "the submenu opens");
        assert!(!lit(&ctx).starts_with(&select_all), "{}", lit(&ctx));
        key(&ctx, &mut app, Key::ArrowLeft);
        assert_eq!(open_chain(&ctx).len(), 1, "Left closes it");
        assert!(lit(&ctx).starts_with(&select_all));
        assert_eq!(open_menu(&ctx), Some(1));
        // Enter opens it too; Esc closes one level only.
        key(&ctx, &mut app, Key::Enter);
        assert_eq!(open_chain(&ctx).len(), 2);
        key(&ctx, &mut app, Key::Escape);
        assert_eq!(open_chain(&ctx).len(), 1);
        assert_eq!(open_menu(&ctx), Some(1));
        key(&ctx, &mut app, Key::ArrowRight);
        down_to(&ctx, &mut app, &tr("menu.edit.select_all_objects"));
        key(&ctx, &mut app, Key::Enter);
        assert_eq!(app.selection.len(), 2);
        assert!(!egui::Popup::is_any_open(&ctx));
    }

    /// The canvas's right-click menu: the arrows light its rows and
    /// never move the selected object, Esc closes it.
    #[test]
    fn a_context_menu_takes_the_arrows_from_the_drawing() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        keys(&ctx, &mut app, Vec::new());
        let before = app.selection_bounds();
        app.context_menu = Some((
            egui::pos2(700.0, 450.0),
            tracedraw_core::geometry::Point::new(30.0, 20.0),
        ));
        keys(&ctx, &mut app, Vec::new());
        assert!(egui::Popup::is_id_open(
            &ctx,
            Id::new("canvas_context_menu")
        ));
        assert_eq!(highlighted(&ctx), None);
        key(&ctx, &mut app, Key::ArrowDown);
        let first = lit(&ctx);
        assert!(!first.is_empty());
        key(&ctx, &mut app, Key::ArrowDown);
        assert_ne!(lit(&ctx), first);
        for k in [Key::ArrowLeft, Key::ArrowRight, Key::ArrowUp] {
            key(&ctx, &mut app, k);
        }
        assert_eq!(app.selection_bounds(), before, "nothing nudged");
        key(&ctx, &mut app, Key::Escape);
        assert!(app.context_menu.is_none());
        assert!(!egui::Popup::is_any_open(&ctx));
        // With the menu closed the arrows nudge again.
        key(&ctx, &mut app, Key::ArrowRight);
        assert_ne!(app.selection_bounds(), before);
    }

    /// The arrows never nudge the selection while a menu of the bar is
    /// open.
    #[test]
    fn arrows_do_not_nudge_while_a_menu_is_open() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        keys(&ctx, &mut app, Vec::new());
        let before = app.selection_bounds();
        let f = titles(&ctx).keys[0];
        let fk = crate::ui::menus::access::key_of(f).expect("access key");
        keys(&ctx, &mut app, vec![press(fk, Modifiers::ALT)]);
        assert_eq!(open_menu(&ctx), Some(0));
        for k in [
            Key::ArrowDown,
            Key::ArrowUp,
            Key::ArrowRight,
            Key::ArrowLeft,
            Key::ArrowLeft,
        ] {
            key(&ctx, &mut app, k);
            assert_eq!(app.selection_bounds(), before, "{k:?}");
        }
        // Nor while the bar is lit.
        key(&ctx, &mut app, Key::Escape);
        assert!(bar_focus(&ctx).is_some());
        key(&ctx, &mut app, Key::ArrowRight);
        key(&ctx, &mut app, Key::ArrowDown);
        key(&ctx, &mut app, Key::Escape);
        key(&ctx, &mut app, Key::Escape);
        assert_eq!(app.selection_bounds(), before);
        assert!(!takes_keys(&ctx));
    }

    /// Down and Up pass over a menu's greyed rows (and its separators):
    /// with nothing selected and nothing to redo, Edit's Redo, Cut and
    /// Copy are never lit (#1839).
    #[test]
    fn the_arrows_skip_greyed_rows() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        app.select(Vec::new());
        keys(&ctx, &mut app, Vec::new());
        assert!(app.engine.redo_label().is_none());
        // Alt alone, Right, Down: Edit (Alt+E is a shortcut of its own).
        window(&ctx, &mut app, Vec::new(), Modifiers::ALT);
        window(&ctx, &mut app, Vec::new(), Modifiers::NONE);
        key(&ctx, &mut app, Key::ArrowRight);
        key(&ctx, &mut app, Key::ArrowDown);
        assert_eq!(open_menu(&ctx), Some(1), "Edit");
        let greyed: Vec<String> = [
            ("menu.edit.redo", "Ctrl+Shift+Z"),
            ("menu.edit.cut", "Ctrl+X"),
            ("menu.edit.copy", "Ctrl+C"),
        ]
        .iter()
        .map(|(k, s)| format!("{} {s}", tr(k)))
        .collect();
        let first = lit(&ctx);
        assert!(!first.is_empty());
        let mut seen = vec![first.clone()];
        for _ in 0..80 {
            key(&ctx, &mut app, Key::ArrowDown);
            let now = lit(&ctx);
            assert!(!greyed.contains(&now), "{now} is greyed");
            if now == first {
                break;
            }
            seen.push(now);
        }
        assert!(seen.len() > 3, "{seen:?}");
        assert!(
            seen.iter().any(|s| s.starts_with(&tr("menu.edit.paste"))),
            "{seen:?}"
        );
        // Up from the first lit row goes round to the last one.
        key(&ctx, &mut app, Key::ArrowUp);
        assert_eq!(lit(&ctx), *seen.last().expect("rows"));
    }

    /// The zoom box's list of levels: Down on its arrow opens it, the
    /// arrows walk it, Enter sets the zoom, Esc closes it unchanged.
    #[test]
    fn the_zoom_levels_open_and_pick_from_the_keyboard() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        app.set_tool(crate::tools::Tool::Zoom);
        keys(&ctx, &mut app, Vec::new());
        let arrow = Id::new(("bar_zoom", "arrow"));
        let levels = Id::new(("bar_zoom", "levels"));
        assert!(
            ctx.viewport(|v| v.prev_pass.widgets.get(arrow).is_some()),
            "the zoom box is on the property bar"
        );
        ctx.memory_mut(|m| m.request_focus(arrow));
        keys(&ctx, &mut app, Vec::new());
        let before = app.zoom_percent();
        key(&ctx, &mut app, Key::ArrowDown);
        assert!(egui::Popup::is_id_open(&ctx, levels));
        key(&ctx, &mut app, Key::Home);
        assert!(lit(&ctx).starts_with(&tr("toolbar.zoom_to_selected")));
        key(&ctx, &mut app, Key::Escape);
        assert!(!egui::Popup::is_id_open(&ctx, levels));
        assert_eq!(app.zoom_percent(), before);
        assert_eq!(ctx.memory(|m| m.focused()), Some(arrow));
        // Alt+Down opens it again; 400% picked with the arrows and Enter.
        keys(&ctx, &mut app, vec![press(Key::ArrowDown, Modifiers::ALT)]);
        assert!(egui::Popup::is_id_open(&ctx, levels));
        key(&ctx, &mut app, Key::Home);
        down_to(&ctx, &mut app, "400%");
        key(&ctx, &mut app, Key::Enter);
        assert!(!egui::Popup::is_id_open(&ctx, levels));
        assert!(
            (app.zoom_percent() - 400.0).abs() < 0.5,
            "{}",
            app.zoom_percent()
        );
    }

    /// A long list scrolls to the row the keys light: End in the page
    /// size list shows its last size.
    #[test]
    fn a_long_list_scrolls_to_the_lit_row() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        app.select(Vec::new());
        keys(&ctx, &mut app, Vec::new());
        let pass = ctx.cumulative_pass_nr();
        let combos: Vec<Combo> = ctx.data_mut(|d| {
            d.get_temp_mut_or_default::<Noted<Combo>>(combos_id())
                .seen(pass)
                .cloned()
                .collect()
        });
        // The longest list of the page bar.
        let mut longest: Option<(Combo, usize)> = None;
        for c in combos {
            ctx.memory_mut(|m| m.request_focus(c.button));
            key(&ctx, &mut app, Key::ArrowDown);
            if !egui::Popup::is_id_open(&ctx, c.popup) {
                continue;
            }
            let n = rows_of(&ctx, c.popup).len();
            key(&ctx, &mut app, Key::Escape);
            if longest.as_ref().is_none_or(|l| n > l.1) {
                longest = Some((c, n));
            }
        }
        let (combo, n) = longest.expect("a list on the page bar");
        assert!(n > 10, "{n} rows");
        ctx.memory_mut(|m| m.request_focus(combo.button));
        key(&ctx, &mut app, Key::ArrowDown);
        key(&ctx, &mut app, Key::End);
        let focus = ctx.memory(|m| m.focused()).expect("a row lit");
        let in_view = |focus: Id| {
            let row = ctx.viewport(|v| v.prev_pass.widgets.get(focus).map(|w| w.rect));
            let area = ctx.memory(|m| m.area_rect(combo.popup));
            // Up and down: a long name may run past the list's sides.
            row.zip(area)
                .map(|(r, a)| a.top() <= r.top() + 1.0 && r.bottom() - 1.0 <= a.bottom())
        };
        assert_eq!(
            in_view(focus),
            Some(true),
            "the last row is scrolled into view"
        );
        let rows = rows_of(&ctx, combo.popup);
        let first_row = rows.first().expect("rows").0;
        let tall = rows.last().expect("rows").1.bottom() - rows[0].1.top()
            > ctx
                .memory(|m| m.area_rect(combo.popup))
                .map_or(f32::INFINITY, |a| a.height());
        if tall {
            assert_eq!(in_view(first_row), Some(false), "the list scrolled");
        }
        key(&ctx, &mut app, Key::Home);
        let focus = ctx.memory(|m| m.focused()).expect("a row lit");
        assert_eq!(in_view(focus), Some(true), "and back to the first");
        key(&ctx, &mut app, Key::Escape);
    }

    const FRUIT: [&str; 6] = ["Apple", "Banana", "Blueberry", "Cherry", "Date", "Elder"];

    /// A frame of a combo box of fruit at `time`.
    fn fruit(ctx: &Context, value: &mut usize, events: Vec<Event>, time: f64) {
        let input = egui::RawInput {
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 600.0),
            )),
            events,
            time: Some(time),
            ..Default::default()
        };
        let mut out = ctx.run_ui(input, |ui| {
            frame(ui.ctx(), false);
            egui::Area::new(Id::new("fruit_bar"))
                .fixed_pos(egui::pos2(50.0, 50.0))
                .show(ui.ctx(), |ui| {
                    ComboBox::from_id_salt("fruit")
                        .selected_text(FRUIT[*value])
                        .show_ui(ui, |ui| {
                            for (i, f) in FRUIT.iter().enumerate() {
                                ui.selectable_value(value, i, *f);
                            }
                        });
                });
            end_frame(ui.ctx());
        });
        out.textures_delta.clear();
    }

    /// Up and Down, letters typed, Home, End, Enter and Esc in a combo
    /// box's list; the button with the keyboard opens it.
    #[test]
    fn a_combo_box_list_works_from_the_keyboard() {
        let ctx = crate::theme::ui_context();
        let mut value = 1;
        let t = std::cell::Cell::new(0.0);
        let run = |value: &mut usize, events: Vec<Event>| {
            let mut events = Some(events);
            for _ in 0..5 {
                let e = events.take().unwrap_or_default();
                t.set(t.get() + 0.1);
                fruit(&ctx, value, e, t.get());
            }
        };
        run(&mut value, Vec::new());
        let combo = {
            let pass = ctx.cumulative_pass_nr();
            ctx.data_mut(|d| {
                d.get_temp_mut_or_default::<Noted<Combo>>(combos_id())
                    .seen(pass)
                    .next()
                    .cloned()
            })
            .expect("combo noted")
        };
        ctx.memory_mut(|m| m.request_focus(combo.button));
        run(&mut value, Vec::new());
        // Down on the button opens the list on the chosen row.
        run(&mut value, vec![press(Key::ArrowDown, Modifiers::NONE)]);
        assert!(egui::Popup::is_id_open(&ctx, combo.popup));
        assert_eq!(lit(&ctx), "Banana");
        run(&mut value, vec![press(Key::ArrowDown, Modifiers::NONE)]);
        assert_eq!(lit(&ctx), "Blueberry");
        run(&mut value, vec![press(Key::End, Modifiers::NONE)]);
        assert_eq!(lit(&ctx), "Elder");
        run(&mut value, vec![press(Key::Home, Modifiers::NONE)]);
        assert_eq!(lit(&ctx), "Apple");
        run(&mut value, vec![press(Key::PageDown, Modifiers::NONE)]);
        assert_eq!(lit(&ctx), "Elder", "a page down the short list");
        // Letters: the first name with them, the same letter again the
        // next one.
        run(&mut value, typed('c'));
        assert_eq!(lit(&ctx), "Cherry");
        t.set(t.get() + 2.0);
        run(&mut value, typed('b'));
        assert_eq!(lit(&ctx), "Banana");
        run(&mut value, typed('b'));
        assert_eq!(lit(&ctx), "Blueberry");
        assert_eq!(value, 1, "nothing picked yet");
        // Esc closes it as it was, the button keeps the keyboard.
        run(&mut value, vec![press(Key::Escape, Modifiers::NONE)]);
        assert!(!egui::Popup::is_id_open(&ctx, combo.popup));
        assert_eq!(value, 1);
        assert_eq!(ctx.memory(|m| m.focused()), Some(combo.button));
        // Alt+Down opens it again; Enter picks.
        run(&mut value, vec![press(Key::ArrowDown, Modifiers::ALT)]);
        assert!(egui::Popup::is_id_open(&ctx, combo.popup));
        run(&mut value, vec![press(Key::ArrowDown, Modifiers::NONE)]);
        run(&mut value, vec![press(Key::ArrowDown, Modifiers::NONE)]);
        assert_eq!(lit(&ctx), "Cherry");
        run(&mut value, vec![press(Key::Enter, Modifiers::NONE)]);
        assert_eq!(value, 3);
        assert!(!egui::Popup::is_id_open(&ctx, combo.popup));
        assert_eq!(ctx.memory(|m| m.focused()), Some(combo.button));
        // Enter on the button opens it too.
        run(&mut value, vec![press(Key::Enter, Modifiers::NONE)]);
        assert!(egui::Popup::is_id_open(&ctx, combo.popup));
        assert_eq!(lit(&ctx), "Cherry");
    }

    /// Tab goes from a field of the property bar to its lists, and the
    /// list with the keyboard opens with Down (#1839).
    #[test]
    fn tab_reaches_the_lists_of_the_property_bar() {
        let ctx = crate::theme::ui_context();
        let mut app = app_with_shapes();
        app.select(Vec::new());
        keys(&ctx, &mut app, Vec::new());
        let combos: Vec<Combo> = {
            let pass = ctx.cumulative_pass_nr();
            ctx.data_mut(|d| {
                d.get_temp_mut_or_default::<Noted<Combo>>(combos_id())
                    .seen(pass)
                    .cloned()
                    .collect()
            })
        };
        assert!(!combos.is_empty(), "the page bar has lists");
        let target = combos[0].clone();
        // The focusable widget drawn just before the list on its row.
        let before = ctx
            .viewport(|v| {
                let b = v.this_pass.widgets.get(target.button)?;
                let row = b.rect.y_range();
                let all: Vec<egui::WidgetRect> =
                    v.this_pass.widgets.get_layer(b.layer_id).copied().collect();
                let at = all.iter().position(|w| w.id == target.button)?;
                all[..at]
                    .iter()
                    .rev()
                    .find(|w| {
                        w.sense.senses_click()
                            && w.enabled
                            && w.rect.is_positive()
                            && row.contains(w.rect.center().y)
                    })
                    .map(|w| w.id)
            })
            .expect("a widget before the list");
        ctx.memory_mut(|m| m.request_focus(before));
        keys(&ctx, &mut app, Vec::new());
        let mut reached = false;
        for _ in 0..12 {
            key(&ctx, &mut app, Key::Tab);
            if ctx.memory(|m| m.focused()) == Some(target.button) {
                reached = true;
                break;
            }
        }
        assert!(reached, "Tab reaches the list");
        key(&ctx, &mut app, Key::ArrowDown);
        assert!(egui::Popup::is_id_open(&ctx, target.popup));
        assert!(!lit(&ctx).is_empty(), "its chosen row is lit");
        key(&ctx, &mut app, Key::Escape);
        assert!(!egui::Popup::is_id_open(&ctx, target.popup));
        assert_eq!(ctx.memory(|m| m.focused()), Some(target.button));
    }
}
