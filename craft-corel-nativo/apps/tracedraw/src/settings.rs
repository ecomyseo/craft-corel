//! User settings persisted between sessions: language, recent files,
//! workspace flags, tool defaults. Stored as JSON in the platform config
//! directory (`%APPDATA%\Craft Corel`, `~/Library/Application Support/Craft Corel`,
//! `$XDG_CONFIG_HOME/tracedraw`).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// How large the buttons of the bars are (the bars' menu > Customize),
/// five sizes from Extra Small to Extra Large. Ctrl+wheel over a bar
/// steps through the same sizes. Saved as `extra_small`, `small`,
/// `medium`, `large` and `extra_large`; settings saved with three sizes
/// keep their meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ButtonSize {
    #[serde(alias = "ExtraSmall")]
    ExtraSmall,
    #[default]
    #[serde(alias = "Small")]
    Small,
    #[serde(alias = "Medium")]
    Medium,
    #[serde(alias = "Large")]
    Large,
    #[serde(alias = "ExtraLarge")]
    ExtraLarge,
}

/// The pixel sizes (points, at interface size 100 %) of one button size:
/// the toolbox's icon box (the side of its 24 unit design grid, see
/// `ui::icon_paint::Pen`), the toolbox's and the standard bars' buttons,
/// the standard bars' icon box, and the property bar's buttons and
/// icons. All are whole, even pixel sizes so an icon sits centred on
/// whole pixels in its button.
///
/// The toolbox's icon boxes are 16, 20, 24, 32 and 40, the padding
/// around them growing more slowly than the icon. The standard and
/// property bars' icons are drawn smaller at Large and Extra Large,
/// about 5/6 of the toolbox's box (standard bars 28 and 34, property bar
/// 24 and 28), so their heavier line icons keep 7 and 9 px of margin in
/// the button instead of nearly filling it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ButtonMetrics {
    pub icon: f32,
    pub button: f32,
    pub bar_icon: f32,
    pub prop_icon: f32,
    pub prop_button: f32,
}

impl ButtonSize {
    pub const ALL: [ButtonSize; 5] = [
        ButtonSize::ExtraSmall,
        ButtonSize::Small,
        ButtonSize::Medium,
        ButtonSize::Large,
        ButtonSize::ExtraLarge,
    ];

    /// The sizes of the buttons and icons at this size.
    pub fn metrics(self) -> ButtonMetrics {
        let (icon, button, bar_icon, prop_icon, prop_button) = match self {
            ButtonSize::ExtraSmall => (16.0, 22.0, 16.0, 14.0, 20.0),
            ButtonSize::Small => (20.0, 26.0, 20.0, 18.0, 24.0),
            ButtonSize::Medium => (24.0, 32.0, 24.0, 22.0, 28.0),
            ButtonSize::Large => (32.0, 42.0, 28.0, 24.0, 38.0),
            ButtonSize::ExtraLarge => (40.0, 52.0, 34.0, 28.0, 46.0),
        };
        ButtonMetrics {
            icon,
            button,
            bar_icon,
            prop_icon,
            prop_button,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            ButtonSize::ExtraSmall => "bars.size_extra_small",
            ButtonSize::Small => "bars.size_small",
            ButtonSize::Medium => "bars.size_medium",
            ButtonSize::Large => "bars.size_large",
            ButtonSize::ExtraLarge => "bars.size_extra_large",
        }
    }

    /// The size `steps` sizes up (down for a negative count) from this
    /// one, held at Extra Small and Extra Large: Ctrl+wheel over a bar
    /// steps its buttons' size by the notches turned.
    pub fn stepped(self, steps: i32) -> ButtonSize {
        let at = ButtonSize::ALL.iter().position(|s| *s == self).unwrap_or(0) as i64;
        let last = (ButtonSize::ALL.len() - 1) as i64;
        let to = (at + i64::from(steps)).clamp(0, last);
        ButtonSize::ALL[to as usize]
    }
}

/// What the toolbox's buttons show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Captions {
    #[default]
    ImageOnly,
    CaptionOnly,
    CaptionRight,
}

impl Captions {
    pub const ALL: [Captions; 3] = [
        Captions::ImageOnly,
        Captions::CaptionOnly,
        Captions::CaptionRight,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Captions::ImageOnly => "bars.image_only",
            Captions::CaptionOnly => "bars.caption_only",
            Captions::CaptionRight => "bars.caption_right",
        }
    }

    /// The icon shows.
    pub fn image(self) -> bool {
        self != Captions::CaptionOnly
    }

    /// The tool's name shows.
    pub fn caption(self) -> bool {
        self != Captions::ImageOnly
    }
}

/// A page size the user saved under a name, in millimetres (portrait or
/// landscape as saved).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageSizePreset {
    pub name: String,
    pub width_mm: f64,
    pub height_mm: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub language: String,
    pub recent_files: Vec<PathBuf>,
    pub workspace: String,
    pub show_welcome_on_start: bool,
    /// The Welcome Screen tab was closed: it shows as its house alone
    /// until the Welcome Screen is shown again (#677).
    pub welcome_tab_closed: bool,
    /// The Crop tool cuts an upright picture's pixels instead of putting
    /// it in a clip frame (#1913).
    pub crop_picture_pixels: bool,
    /// Options > Tools > Table: what Tab does in a cell.
    #[serde(default)]
    pub table: crate::table_tool::TablePrefs,
    pub units: String,
    pub nudge_mm: f64,
    /// Shift+Arrow distance (super nudge), mm.
    #[serde(default = "default_super_nudge")]
    pub super_nudge_mm: f64,
    /// Ctrl+Arrow distance (micro nudge), mm.
    #[serde(default = "default_micro_nudge")]
    pub micro_nudge_mm: f64,
    pub duplicate_offset_mm: [f64; 2],
    pub default_dpi: f64,
    pub snap: SnapPrefs,
    pub shortcuts: Vec<(String, String)>,
    /// The program Edit Bitmap opens pictures in (a path; empty for the
    /// system's program for PNG files, #593).
    #[serde(default)]
    pub bitmap_editor: String,
    pub last_dir: Option<PathBuf>,
    /// Folders bookmarked in the Save Drawing dialog (#1391).
    pub folder_bookmarks: Vec<PathBuf>,
    pub color: ColorPrefs,
    /// Show the Outline flyout in the toolbox (hidden by default).
    pub show_outline_flyout: bool,
    /// Tool shown by each toolbox group (tool ids, one per group): the one
    /// last used from its flyout.
    #[serde(default)]
    pub toolbox: Vec<String>,
    /// Default action of the mouse wheel: zoom (the default) or scroll.
    #[serde(default = "default_true")]
    pub wheel_zooms: bool,
    /// The view a drawing opens in (#1134).
    #[serde(default)]
    pub open_view: OpenView,
    /// Arrowheads created with Object > Create > Arrowhead.
    #[serde(default)]
    pub custom_arrowheads: Vec<tracedraw_core::Arrowhead>,
    /// Dash patterns made with Edit Line Style (on, off, ... lengths in
    /// outline widths).
    #[serde(default)]
    pub custom_dashes: Vec<Vec<f64>>,
    /// Page sizes saved in Document Setup > Page Size, listed in every
    /// page size list after the favourites.
    #[serde(default)]
    pub page_sizes: Vec<PageSizePreset>,
    /// The unit outline widths are shown and typed in (a unit id, "pt"
    /// by default); widths are kept in millimetres.
    #[serde(default = "default_outline_units")]
    pub outline_units: String,
    /// Options > Text > Display > Text units: the unit font sizes are shown
    /// and typed in (a unit id, "pt" by default); sizes are kept in
    /// points (#529).
    #[serde(default = "default_outline_units")]
    pub text_units: String,
    /// Grid, ruler and guideline settings new drawings start with
    /// (Document Setup > Save as Default).
    #[serde(default)]
    pub document_defaults: DocumentDefaults,
    /// Options > Text > Autocorrect.
    #[serde(default)]
    pub autocorrect: crate::autocorrect::AutocorrectPrefs,
    /// Options > Text > Spelling.
    #[serde(default)]
    pub spelling: SpellingPrefs,
    /// Thesaurus file picked by the user (Text > Proofing > Thesaurus).
    #[serde(default)]
    pub thesaurus_file: Option<PathBuf>,
    /// Options > General: show the Create a New Document dialog for File >
    /// New (off: new drawings use the last settings at once).
    #[serde(default = "default_true")]
    pub show_new_document_dialog: bool,
    /// View > Scrollbars: the drawing window's scrollbars shown (off:
    /// the drawing takes their room; the page tabs and the Navigator
    /// button stay, #1344).
    #[serde(default = "default_true")]
    pub show_scrollbars: bool,
    /// The Create a New Document dialog's last used settings.
    #[serde(default)]
    pub new_document: crate::new_document::NewDocSettings,
    /// Colour of the area around the page (Customization > Appearance).
    #[serde(default = "default_desktop")]
    pub desktop_rgb: [u8; 3],
    /// The desktop colour was picked: white too is then white, not the
    /// theme's colour that white stood for (#530).
    #[serde(default)]
    pub desktop_custom: bool,
    /// What the status bar's first field shows.
    #[serde(default)]
    pub status_info: crate::ui::status::StatusInfo,
    /// Toolbox flyouts hidden with the toolbox's "+" button (the id of
    /// each flyout's first tool).
    #[serde(default)]
    pub toolbox_hidden: Vec<String>,
    /// The bars' menu > Customize > Default Button Size: how large the
    /// buttons of every bar are (Small at first, as in the reference
    /// program; a size already saved stays).
    #[serde(default)]
    pub button_size: ButtonSize,
    /// The property bar's Autofit Page: the margin (mm) left around the
    /// drawing when the page is fitted to it.
    #[serde(default)]
    pub fit_page_margin: f64,
    /// Autofit Page's popup shows the page it would make while open
    /// (#659).
    #[serde(default = "default_true")]
    pub fit_page_preview: bool,
    /// The toolbox's own button size, when it does not follow the default.
    #[serde(default)]
    pub toolbox_button_size: Option<ButtonSize>,
    /// What the toolbox's buttons show: the icon, the tool's name, or both.
    #[serde(default)]
    pub toolbox_captions: Captions,
    /// Customization > Appearance > Interface size, relative to the
    /// scaling the system gives the monitor (1.0 = 100 %, the size of
    /// other programs, a zoom of `ui::BASE_ZOOM`); 0 is 100 % too.
    /// Preferences written before sizes were relative kept absolute sizes
    /// as `ui_scale`, which is ignored.
    #[serde(default)]
    pub ui_size: f32,
    /// Options > General: show the splash screen while Craft Corel starts.
    #[serde(default = "default_true")]
    pub show_splash: bool,
    /// Options > General: draw the window with the graphics card (off:
    /// its software adapter), from the next start (#445).
    #[serde(default = "default_true")]
    pub hardware_acceleration: bool,
    /// The graphics card chosen by name; empty, the fastest one.
    #[serde(default)]
    pub gpu_adapter: String,
    /// The renderer that opened the window when the first choice could
    /// not ("software" or "gl"); cleared when the choice changes.
    #[serde(default)]
    pub working_renderer: String,
    /// Customization > Appearance: dialogs open in the middle of the
    /// window and stay there; off, they move by their title bar and open
    /// where they were left.
    #[serde(default = "default_true")]
    pub center_dialogs: bool,
    /// Customization > Appearance: shadows under menus and dialogs (off
    /// draws less, for slower graphics).
    #[serde(default = "default_true")]
    pub ui_shadows: bool,
    /// Appearance: tool icons in a colour per group instead of the
    /// theme's icon colour.
    #[serde(default)]
    pub colored_icons: bool,
    /// Customization > Appearance > Theme: "light", "medium_light",
    /// "medium_dark" or "dark".
    #[serde(default)]
    pub theme: String,
    /// Where the toolbox, palette, toolbars and status bar sit.
    #[serde(default)]
    pub bars: crate::ui::bars::BarLayout,
    /// The buttons of the Standard and Zoom toolbars, by bar, when
    /// changed (see ui/bar_items.rs).
    #[serde(default)]
    pub bar_items: std::collections::BTreeMap<String, Vec<String>>,
    /// The main window as it was left: maximized, else its inner size in
    /// points ([0, 0] until the first exit).
    #[serde(default = "default_true")]
    pub window_maximized: bool,
    #[serde(default)]
    pub window_size: [f32; 2],
    /// Options > General: what opens at start-up.
    #[serde(default)]
    pub startup: Startup,
    /// Options > General: undo levels.
    #[serde(default = "default_undo_levels")]
    pub undo_levels: usize,
    /// Options > Display.
    #[serde(default = "default_true")]
    pub show_tooltips: bool,
    /// The Font Manager's folders, installed files, Favorites and
    /// collections.
    #[serde(default)]
    pub font_manager: crate::ui::font_manager::FontManagerPrefs,
    /// TraceCut: shown or not (Options > General) and the tool picked
    /// last.
    #[serde(default)]
    pub tracecut: crate::tracecut::TraceCutPrefs,
    #[serde(default)]
    pub hide_bbox_curve_tools: bool,
    /// Options > Display: the Pick tool selects and drags the nodes of a
    /// selected curve (node tracking).
    #[serde(default)]
    pub node_tracking: bool,
    #[serde(default = "default_true")]
    pub preview_page_border: bool,
    /// Options > Edit: Ctrl constrains rotation and line angles to
    /// multiples of this angle (degrees).
    #[serde(default = "default_constrain_angle")]
    pub constrain_angle: f64,
    /// The symmetry bar's Show symmetry lines (on at first).
    #[serde(default = "default_true")]
    pub symmetry_show_lines: bool,
    /// The symmetry bar's Show full preview: copies drawn in full while a
    /// symmetry is edited (off at first: outlines in red).
    #[serde(default)]
    pub symmetry_full_preview: bool,
    /// The Corners docker's Preview: the result drawn on the page as a
    /// dashed outline while the docker is open (on at first, #1759).
    #[serde(default = "default_true")]
    pub corners_preview: bool,
    /// Options > Edit: decimal places of distances in the property bar
    /// and the status bar.
    #[serde(default = "default_precision")]
    pub precision: u32,
    /// Options > Nodes and Handles.
    #[serde(default)]
    pub nodes: NodePrefs,
    /// Options > Clip Frames.
    #[serde(default)]
    pub clip_frame: ClipFramePrefs,
    /// Options > Save.
    #[serde(default)]
    pub backup: BackupPrefs,
    /// Options > Text: Ctrl+Numpad 8 and 2 change the font size by this
    /// many points.
    #[serde(default = "default_text_increment")]
    pub text_increment_pt: f64,
    /// Options > Text > Paragraph Text: the distance between the default
    /// tab stops of a paragraph with no stops of its own, mm (#1852).
    #[serde(default = "default_tab_spacing")]
    pub default_tab_mm: f64,
    /// Options > Tools > Pick: unfilled objects can be picked inside (on
    /// by default).
    #[serde(default = "default_true")]
    pub treat_all_filled: bool,
    /// Options > Craft Corel > Edit: how visible the objects outside the
    /// one in focus stay, in percent.
    #[serde(default = "default_focus_visibility")]
    pub focus_visibility: u8,
    /// Options > Craft Corel > Edit: a double click on a group brings it
    /// into focus.
    #[serde(default = "default_true")]
    pub focus_double_click: bool,
    /// The name comments are signed with (empty: the drawing's author).
    #[serde(default)]
    pub comment_author: String,
    /// Appearance: the interface grows with the desktop's text size
    /// (Windows' "Make text bigger").
    #[serde(default = "default_true")]
    pub follow_text_size: bool,
    /// Appearance: the high contrast theme while the desktop has one on.
    #[serde(default = "default_true")]
    pub follow_high_contrast: bool,
    /// Options > Tools > Pick: a marquee also takes the objects it only
    /// touches (off by default; Alt does it either way round).
    #[serde(default)]
    pub marquee_touching: bool,
    /// The one-time changes already made to preferences saved by older
    /// versions (see `Settings::upgrade`); 0 in files that predate them.
    #[serde(default)]
    pub upgrades: u32,
    /// Options > Tools > Pick: the Pick tool's pointer is a cross hair.
    #[serde(default)]
    pub crosshair_cursor: bool,
    /// Options > Display > Ruler calibration: pixels per inch of the
    /// screen at the system's scaling, so that zoom 100% shows real sizes
    /// (96 by default; see `screen_points_per_inch`).
    #[serde(default = "default_screen_ppi")]
    pub screen_ppi: f32,
    /// Options > Tools > Zoom/Pan: a right click with the Zoom tool zooms
    /// out (off: it opens the context menu).
    #[serde(default = "default_true")]
    pub zoom_right_click_out: bool,
    /// Options > Tools > Zoom/Pan: zoom 100% is the real size at the
    /// calibrated screen resolution (on, the default); off, it is 96 per
    /// inch whatever the calibration.
    #[serde(default = "default_true")]
    pub zoom_relative_1to1: bool,
    /// Options > Tools > Zoom/Pan: a wheel zoom brings the point under
    /// the pointer to the middle of the view and moves the pointer there
    /// (off by default: the point stays under the pointer).
    #[serde(default)]
    pub wheel_zoom_centers: bool,
    /// Options > Tools > Zoom/Pan: a right click with the Pan tool zooms
    /// out (off, the default: it opens the context menu).
    #[serde(default)]
    pub pan_right_click_out: bool,
    /// Options > Customization > Color Palette.
    #[serde(default)]
    pub palette: PalettePrefs,
    /// The palette bars open besides the default one, in order, each
    /// with its own place, rows and swatch size. Preferences written
    /// before they existed have none: one palette at the bottom.
    #[serde(default)]
    pub palette_bars: Vec<PaletteBarPrefs>,
    /// The palette the default palette bar shows ([`palette_id`]); empty
    /// for the default palette.
    #[serde(default)]
    pub main_palette: String,
    /// Names the user gave palettes, by [`palette_id`] (the document
    /// palette by [`DOCUMENT_PALETTE`]); a palette missing here keeps
    /// its own name. Built-in palettes' files are never changed.
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub palette_names: std::collections::BTreeMap<String, String>,
    /// The palettes the user made in Craft Corel (New Palette, Duplicate),
    /// which have no file: their names and colours, in the list's order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub user_palettes: Vec<UserPalette>,
    /// Tool settings new objects take.
    #[serde(default)]
    pub tools: ToolPrefs,
    /// What the drawing window shows.
    #[serde(default)]
    pub view: ViewPrefs,
    /// Dockers' own choices.
    #[serde(default)]
    pub panels: PanelPrefs,
    /// The Time Tracker's toolbar and idle time.
    #[serde(default)]
    pub time_tracker: TimeTrackerPrefs,
    /// File > Preflight: the checks to run and their thresholds.
    #[serde(default)]
    pub preflight: crate::preflight::PreflightPrefs,
    /// File > Export as PDF: the choices used last (never the passwords).
    #[serde(default)]
    pub publish_pdf: crate::ui::publish_pdf::PublishPrefs,
    /// What a fill or outline chosen with nothing selected changes.
    #[serde(default)]
    pub defaults: crate::defaults::DefaultsPrefs,
    /// The bars shown when Craft Corel was left; none in preferences written
    /// before they were kept (the workspace's own bars then show).
    #[serde(default)]
    pub shown_bars: Option<ShownBars>,
    /// The dockers in the tab column, in order, and the one in front, as
    /// they were left; none in preferences written before they were kept
    /// (the default dockers then show).
    #[serde(default)]
    pub dockers: Option<DockerSetup>,
    /// The font lists' fonts used last, newest first.
    #[serde(default)]
    pub recent_fonts: Vec<String>,
    /// Options > Text > Fonts: the fonts used last shown at the top of
    /// the font lists, and how many (1 to 20).
    #[serde(default = "default_true")]
    pub show_recent_fonts: bool,
    #[serde(default = "default_recent_font_count")]
    pub recent_font_count: usize,
    /// The Properties docker's sections closed ("outline", "fill" ...).
    #[serde(default)]
    pub properties_closed: Vec<String>,
    /// The Text docker's sections closed ("character", "paragraph",
    /// "frame").
    #[serde(default)]
    pub text_sections_closed: Vec<String>,
    /// The Text docker's first section shown (0 Character, 1 Paragraph,
    /// 2 Frame): the buttons at its top show from that section on.
    #[serde(default)]
    pub text_docker_from: u8,
    /// The Text docker shows the boxes at the right of its rows, which
    /// pick what "Apply to default text" takes.
    #[serde(default)]
    pub text_docker_checks: bool,
    /// The rows ticked for "Apply to default text" ("font", "size" ...).
    #[serde(default)]
    pub text_docker_ticked: Vec<String>,
    /// The Glyphs docker's recent glyphs, newest first: (font, glyph).
    #[serde(default)]
    pub recent_glyphs: Vec<(String, String)>,
    /// Custom optional hyphens: (language, word, hyphenated word).
    #[serde(default)]
    pub custom_hyphens: Vec<(String, String, String)>,
    /// Text typed (finished) or pasted and imported with a defined word
    /// turns hyphenation on, so the custom hyphens break its lines.
    #[serde(default = "default_true")]
    pub hyphens_when_typing: bool,
    #[serde(default = "default_true")]
    pub hyphens_when_pasting: bool,
    /// The Objects docker's thumbnail size, interface points (16 to 96).
    #[serde(default = "default_object_thumb")]
    pub object_thumb_size: f32,
    /// New master layers showing on all, odd or even pages are drawn in
    /// front of the page's own layers (#1132, #1477).
    #[serde(default)]
    pub master_in_front: [bool; 3],
    /// The Objects docker lists every page, not only the current one
    /// (#1477).
    #[serde(default = "default_true")]
    pub objects_all_pages: bool,
    /// The font lists' sample size, in points.
    #[serde(default = "default_font_sample")]
    pub font_sample_px: f32,
    /// Options > Workspaces: the parts a workspace file exports and
    /// imports.
    #[serde(default)]
    pub workspace_parts: crate::workspace_file::WorkspaceParts,
    /// Options > Global > Printing.
    #[serde(default)]
    pub printing: crate::prefs::PrintPrefs,
    /// Options > Global > File Formats.
    #[serde(default)]
    pub formats: crate::prefs::FormatPrefs,
    /// AI Drawing: the service, address, model and (when asked) the key.
    #[serde(default)]
    pub ai: crate::ai::AiPrefs,
    /// Tools > Options > AI: AI features on (they still need a service
    /// set up with the user's key).
    #[serde(default = "ai_enabled_default")]
    pub ai_enabled: bool,
    /// The separate image service older versions saved. Read once and
    /// folded into `ai` (see `migrate_ai`); it stays in the file only
    /// when it holds a remembered key the one service could not take, so
    /// that key is never lost.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ai_image: Option<crate::ai_image::ImagePrefs>,
    /// Options > Global > General: AI agents on this computer may edit the
    /// open drawing (MCP, `tracedraw --mcp-window`).
    #[serde(default)]
    pub mcp_window: bool,
    /// Options > Global > General: look for a newer release once a day at
    /// start (desktop). Nothing is checked before the user answered the
    /// question at first start (`update_check_asked`).
    #[serde(default = "default_true")]
    pub check_updates: bool,
    /// The user answered the first-start question about checking for
    /// updates (Always check / Never check). Until then no check runs on
    /// its own, so the first start makes no network connection unasked.
    #[serde(default)]
    pub update_check_asked: bool,
    /// When that check last ran, seconds since 1970.
    #[serde(default)]
    pub last_update_check: u64,
    /// A release the user chose to skip: not offered again at start.
    #[serde(default)]
    pub skipped_update: String,
    /// The newest release the last check found that this version is
    /// older than: while it is not installed or skipped, every start asks
    /// again, so the update card shows at each start until it is done.
    #[serde(default)]
    pub pending_update: String,
    /// Scripts panel > Install as Filter: file formats that scripts read
    /// or write (see `behavior/script-filters.md`).
    #[serde(default)]
    pub script_filters: Vec<crate::scripting::ScriptFilter>,
    /// File > GitHub's last repository, branch, path (and the token, when
    /// the user asked to remember it).
    #[serde(default)]
    pub github: crate::github::GithubPrefs,
    /// Options > Global > Bitmap Tracing.
    #[serde(default)]
    pub tracing: crate::prefs::TracePrefs,
    /// Options > Global > Warnings.
    #[serde(default)]
    pub warnings: crate::prefs::WarningPrefs,
    /// Options > Tools > Curves.
    #[serde(default)]
    pub curves: crate::prefs::CurvePrefs,
    /// Text > Show Text Frames: paragraph text frames show as dashed
    /// lines (else only the selected ones).
    #[serde(default = "default_true")]
    pub show_text_frames: bool,
    /// Text fitted to a path: a drag off the path moves the text away
    /// from it in steps of `path_tick_mm` (tick snapping).
    #[serde(default)]
    pub path_tick_snap: bool,
    /// The step of tick snapping, mm.
    #[serde(default = "default_path_tick")]
    pub path_tick_mm: f64,
    /// Options > Display, Screen margins (Android, #1917): the interface
    /// keeps clear of the camera cutout and the screen's rounded corners.
    #[serde(default)]
    pub screen_margins: crate::ui::screen_margins::ScreenMargins,
}

fn default_path_tick() -> f64 {
    1.0
}

/// The dockers as they were left: the default column's dockers, in
/// order, and the one in front (by their name keys, "docker.hints" ...);
/// the other column's, its front and whether it was open; the second
/// column on each side (between the column at the window's edge and the
/// drawing), its front and whether it was open; and the floating ones
/// with their windows.
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct DockerSetup {
    pub open: Vec<String>,
    pub front: String,
    pub left: Vec<String>,
    pub left_front: String,
    pub left_shown: bool,
    /// The second column on the default column's side.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inner: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub inner_front: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub inner_shown: bool,
    /// The second column on the other side.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub left_inner: Vec<String>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub left_inner_front: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub left_inner_shown: bool,
    pub floating: Vec<FloatingSetup>,
}

/// A floating docker as it was left: its name key, the window's top left
/// corner and its size (screen points).
#[derive(Debug, Clone, PartialEq, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FloatingSetup {
    pub tab: String,
    pub pos: [f32; 2],
    pub size: [f32; 2],
}

/// Which bars are shown (Window > Toolbars, the bars' menu). The palette
/// is kept with the palette options (`PalettePrefs::show_palette`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShownBars {
    pub standard_toolbar: bool,
    pub property_bar: bool,
    pub toolbox: bool,
    pub status_bar: bool,
    pub dockers: bool,
    pub rulers: bool,
    pub text_toolbar: bool,
    pub zoom_toolbar: bool,
    pub transform_toolbar: bool,
}

impl Default for ShownBars {
    /// The bars of the Default workspace.
    fn default() -> Self {
        ShownBars {
            standard_toolbar: true,
            property_bar: true,
            toolbox: true,
            status_bar: true,
            dockers: true,
            rulers: true,
            text_toolbar: false,
            zoom_toolbar: false,
            transform_toolbar: false,
        }
    }
}

fn default_desktop() -> [u8; 3] {
    [255, 255, 255]
}

/// How many one-time upgrades `Settings::upgrade` knows.
const UPGRADES: u32 = 5;

fn default_outline_units() -> String {
    "pt".into()
}

fn default_true() -> bool {
    true
}

fn default_screen_ppi() -> f32 {
    96.0
}

/// The calibrated screen resolution, kept within 48 to 400 per inch. It
/// counts pixels at the system's scaling (the unit other programs size
/// themselves in), as the interface at 100 % would if its measures were
/// those of other programs.
pub fn screen_ppi(s: &Settings) -> f32 {
    if s.screen_ppi.is_finite() {
        s.screen_ppi.clamp(48.0, 400.0)
    } else {
        96.0
    }
}

/// Interface points per inch of the screen: the calibrated resolution
/// over the interface's zoom, so that drawings at zoom 100 % keep their
/// real size whatever the interface size.
pub fn screen_points_per_inch(s: &Settings) -> f32 {
    screen_ppi(s) / crate::ui::ui_zoom(effective_ui_size(s))
}

/// Interface points per inch at zoom 100 %: the calibrated resolution, or
/// 96 per inch when Options > Tools > Zoom/Pan > Zoom relative to 1:1 is
/// off.
pub fn zoom_points_per_inch(s: &Settings) -> f32 {
    if s.zoom_relative_1to1 {
        screen_points_per_inch(s)
    } else {
        96.0 / crate::ui::ui_zoom(effective_ui_size(s))
    }
}

/// How much larger the Touch workspace draws the interface (bars,
/// buttons, panels and text), for fingers.
pub const TOUCH_SCALE: f32 = 1.4;

/// The interface size in use: the chosen one, larger in the Touch
/// workspace (at most 400 %).
pub fn effective_ui_size(s: &Settings) -> f32 {
    effective_ui_size_with(s, crate::system_prefs::get())
}

/// The theme in use: the high contrast one while the desktop has high
/// contrast on (unless turned off), else the one picked.
pub fn effective_theme(s: &Settings) -> String {
    effective_theme_with(s, crate::system_prefs::get())
}

pub fn effective_theme_with(s: &Settings, sys: crate::system_prefs::SystemPrefs) -> String {
    if s.follow_high_contrast && sys.high_contrast.is_some() {
        crate::theme::Theme::HighContrast.id().to_string()
    } else {
        s.theme.clone()
    }
}

pub fn effective_ui_size_with(s: &Settings, sys: crate::system_prefs::SystemPrefs) -> f32 {
    let text = if s.follow_text_size {
        sys.text_scale.clamp(1.0, 2.25)
    } else {
        1.0
    };
    let base = (crate::ui::interface_size(s.ui_size) * text).min(4.0);
    // Not on a phone: 1.4 times a 360 dp screen leaves too little room
    // for any bar (#1301, #1691).
    if s.workspace == "touch" && !crate::ui::phone::phone() {
        (base * TOUCH_SCALE).min(4.0)
    } else {
        base
    }
}

fn default_undo_levels() -> usize {
    150
}

fn default_constrain_angle() -> f64 {
    15.0
}

fn default_precision() -> u32 {
    3
}

fn default_text_increment() -> f64 {
    1.0
}

fn default_tab_spacing() -> f64 {
    tracedraw_core::DEFAULT_TAB_STEP
}

/// The view a drawing opens in (Options > Display, #1134).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum OpenView {
    /// The whole page in the window.
    #[default]
    FitPage,
    /// The objects on the page, or the page when it is empty.
    FitObjects,
    /// The page's width across the window.
    PageWidth,
    /// The page's height down the window.
    PageHeight,
    /// Real size, the page in the middle.
    Actual,
    /// The page, place and zoom it was saved with (Craft Corel files), else
    /// the whole page.
    AsSaved,
}

impl OpenView {
    pub const ALL: [OpenView; 6] = [
        OpenView::FitPage,
        OpenView::FitObjects,
        OpenView::PageWidth,
        OpenView::PageHeight,
        OpenView::Actual,
        OpenView::AsSaved,
    ];

    pub fn key(self) -> &'static str {
        match self {
            OpenView::FitPage => "options.open_view_page",
            OpenView::FitObjects => "options.open_view_objects",
            OpenView::PageWidth => "options.open_view_width",
            OpenView::PageHeight => "options.open_view_height",
            OpenView::Actual => "options.open_view_actual",
            OpenView::AsSaved => "options.open_view_saved",
        }
    }
}

/// What opens when the application starts (Options > General).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Startup {
    #[default]
    WelcomeScreen,
    NewDocument,
    LastDocument,
}

impl Startup {
    pub const ALL: [Startup; 3] = [
        Startup::WelcomeScreen,
        Startup::NewDocument,
        Startup::LastDocument,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Startup::WelcomeScreen => "options.startup_welcome",
            Startup::NewDocument => "options.startup_new",
            Startup::LastDocument => "options.startup_last",
        }
    }
}

/// How nodes are drawn: size, shape per node type, colours.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum NodeSize {
    #[default]
    Small,
    Medium,
    Large,
}

impl NodeSize {
    pub const ALL: [NodeSize; 3] = [NodeSize::Small, NodeSize::Medium, NodeSize::Large];

    /// Side of a node square, screen pixels.
    pub fn px(self) -> f32 {
        match self {
            NodeSize::Small => 7.0,
            NodeSize::Medium => 9.0,
            NodeSize::Large => 11.0,
        }
    }

    /// Length of a control handle's arrow head, screen pixels: as long
    /// as a node is wide, so a larger node size gives larger heads too
    /// (#1643), and as wide.
    pub fn handle_head(self) -> f32 {
        self.px()
    }

    pub fn key(self) -> &'static str {
        match self {
            NodeSize::Small => "options.node_small",
            NodeSize::Medium => "options.node_medium",
            NodeSize::Large => "options.node_large",
        }
    }
}

/// How large the Pick tool's selection handles are drawn (#1285).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HandleSize {
    Small,
    Medium,
    #[default]
    Large,
}

impl HandleSize {
    pub const ALL: [HandleSize; 3] = [HandleSize::Small, HandleSize::Medium, HandleSize::Large];

    /// Side of a handle square, interface points.
    pub fn px(self) -> f32 {
        match self {
            HandleSize::Small => 6.0,
            HandleSize::Medium => 8.0,
            HandleSize::Large => 10.0,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            HandleSize::Small => "options.node_small",
            HandleSize::Medium => "options.node_medium",
            HandleSize::Large => "options.node_large",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeShape {
    Square,
    Circle,
    Diamond,
}

impl NodeShape {
    pub const ALL: [NodeShape; 3] = [NodeShape::Square, NodeShape::Circle, NodeShape::Diamond];
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NodePrefs {
    pub size: NodeSize,
    /// The Pick tool's selection handles (#1285).
    pub handles: HandleSize,
    pub cusp: NodeShape,
    pub smooth: NodeShape,
    pub symmetrical: NodeShape,
    /// Selected nodes and handles.
    pub main_rgb: [u8; 3],
    /// The first node of a curve.
    pub secondary_rgb: [u8; 3],
    /// Unselected nodes are filled white (Ctrl+Shift+G).
    pub unselected_filled: bool,
    /// An arrow at the start node shows the curve's direction.
    pub show_direction: bool,
    /// How near, in screen pixels, a press must land on a node or a
    /// control handle to take it (Shape tool, and the curve tools'
    /// nodes).
    pub pick_distance: f32,
    /// How far, in screen pixels, a click may land from an object's
    /// outline and still pick it (Pick tool and the other tools that
    /// pick objects by clicking them); 0 for none beyond the outline
    /// itself (#1679).
    pub object_pick_distance: f32,
    /// The Shape tool shows a corner widget inside the corners of a curve
    /// (or of the chosen nodes), dragged to round them (#635). On by
    /// default; it can be turned off in Options > Nodes and Handles so it
    /// does not get in the way of node editing (#884). Rectangles keep
    /// their own corner rounding either way.
    pub corner_widgets: bool,
    /// Dragging a corner widget rounds its own corner, Ctrl+click
    /// chooses corners to round together, Shift+drag rounds every
    /// corner. Off: a drag rounds every corner, or the chosen nodes'
    /// corners, as before.
    pub corner_one_at_a_time: bool,
}

impl Default for NodePrefs {
    fn default() -> Self {
        NodePrefs {
            size: NodeSize::Small,
            handles: HandleSize::Large,
            cusp: NodeShape::Square,
            smooth: NodeShape::Circle,
            symmetrical: NodeShape::Diamond,
            main_rgb: [0x00, 0x8C, 0xFF],
            secondary_rgb: [0xFF, 0x00, 0x00],
            unselected_filled: true,
            show_direction: false,
            pick_distance: 6.0,
            object_pick_distance: 3.0,
            corner_widgets: true,
            corner_one_at_a_time: true,
        }
    }
}

impl NodePrefs {
    /// The pick distance in screen pixels, kept sensible and never less
    /// than the node marker reaches.
    pub fn pick_px(&self) -> f64 {
        let set = if self.pick_distance.is_finite() {
            self.pick_distance.clamp(2.0, 20.0)
        } else {
            6.0
        };
        f64::from(set.max(self.size.px() / 2.0 + 1.0))
    }

    /// How near a press must land on a control handle's end to take it:
    /// the pick distance, and never less than its arrow head is long, so
    /// the whole head takes a press (#1643).
    pub fn handle_pick_px(&self) -> f64 {
        self.pick_px().max(f64::from(self.size.handle_head()))
    }

    /// The object click distance in screen pixels, kept sensible
    /// (#1679).
    pub fn object_pick_px(&self) -> f64 {
        if self.object_pick_distance.is_finite() {
            f64::from(self.object_pick_distance.clamp(0.0, 20.0))
        } else {
            3.0
        }
    }
}

/// When new clip frame content is centred in its frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AutoCenter {
    /// When the content lies completely outside the frame.
    #[default]
    WhenOutside,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ClipFramePrefs {
    pub auto_center: AutoCenter,
    /// An empty frame shows crossing lines on screen.
    pub empty_lines: bool,
}

impl Default for ClipFramePrefs {
    fn default() -> Self {
        ClipFramePrefs {
            auto_center: AutoCenter::WhenOutside,
            empty_lines: true,
        }
    }
}

/// The colour palettes docked under the drawing window (Options >
/// Customization > Color Palette).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PalettePrefs {
    /// Colours applied to objects join the document palette.
    pub auto_update_document: bool,
    /// A right click on a swatch sets the outline colour (off: it opens
    /// the palette menu).
    pub right_click_outline: bool,
    /// The "No Color" well at the start of the palette.
    pub show_no_color: bool,
    /// The document palette row under the default palette.
    pub show_document: bool,
    /// The default palette itself (Window > Toolbars, the bars' menu).
    pub show_palette: bool,
    /// Rows of swatches the default palette shows (columns docked on a
    /// side), 1 to [`MAX_ROWS`] (#610); [`ROWS_FIT`] for as many as
    /// show every colour (at most [`MAX_ROWS`]).
    pub rows: u8,
    /// The side of a swatch (whole points), [`SWATCH_MIN`] to
    /// [`SWATCH_MAX`]: Ctrl+wheel over a palette changes it by
    /// [`SWATCH_STEP`] a notch. Preferences written before it existed
    /// keep the 20 px swatches.
    pub swatch_size: u8,
    /// The room asked for the floating palette's swatches (points, the
    /// title left out): its rows or columns flow from it (see
    /// `ui::palette::floating_flow`).
    pub float_size: [f32; 2],
}

/// The most rows (or columns) a docked palette bar shows.
pub const MAX_ROWS: u8 = 8;
/// [`PalettePrefs::rows`] for "Fit": the rows every colour needs.
pub const ROWS_FIT: u8 = 0;
/// The swatch sizes the palette menu and the Palette Manager offer, with
/// their names (points, all even so Ctrl+wheel reaches each from the
/// first size): Medium is the first size.
pub const SWATCH_STEPS: [(u8, &str); 5] = [
    (10, "palette.size_xs"),
    (14, "palette.size_s"),
    (20, "palette.size_m"),
    (28, "palette.size_l"),
    (40, "palette.size_xl"),
];
/// The room a floating palette asks for at first.
pub const FLOAT_SIZE: [f32; 2] = [460.0, 25.0];

/// The smallest and the largest swatch, and the step a wheel notch
/// changes the side by (points).
pub const SWATCH_MIN: u8 = 8;
pub const SWATCH_MAX: u8 = 48;
pub const SWATCH_STEP: i32 = 2;
/// The side the swatches have at first.
pub const SWATCH_DEFAULT: u8 = 20;

impl PalettePrefs {
    /// [`Self::swatch_size`] brought into its range, as points.
    pub fn swatch_size(&self) -> f32 {
        f32::from(self.swatch_size.clamp(SWATCH_MIN, SWATCH_MAX))
    }
}

/// The swatch side `steps` notches of Ctrl+wheel on from `size`: up
/// larger, down smaller, [`SWATCH_STEP`] a notch, held between
/// [`SWATCH_MIN`] and [`SWATCH_MAX`].
/// A rows setting as rows: None for [`ROWS_FIT`], else 1 to
/// [`MAX_ROWS`].
pub fn rows_of(rows: u8) -> Option<usize> {
    (rows != ROWS_FIT).then(|| rows.min(MAX_ROWS) as usize)
}

/// The named size of [`SWATCH_STEPS`] a swatch side is, if any.
pub fn swatch_step_of(size: u8) -> Option<usize> {
    SWATCH_STEPS.iter().position(|(s, _)| *s == size)
}

/// The palette's place in the Palette Manager's list stated so it holds
/// across sessions: its file, else its built-in key, else its name.
pub fn palette_id(p: &crate::palette::Palette) -> String {
    match (&p.path, p.key) {
        (Some(path), _) => path.display().to_string(),
        (None, key) if !key.is_empty() => key.to_string(),
        _ => p.name.clone(),
    }
}

/// A palette made in Craft Corel (no file behind it), kept in the
/// preferences: its name (its [`palette_id`]) and its colours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct UserPalette {
    pub name: String,
    pub colors: Vec<(String, tracedraw_core::Color)>,
}

/// [`PaletteBarPrefs::palette`] of a bar showing the document palette.
pub const DOCUMENT_PALETTE: &str = "document";

/// A palette bar besides the default one: which palette it shows, where
/// it is, and its own rows and swatch size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaletteBarPrefs {
    /// [`palette_id`] of its palette, or [`DOCUMENT_PALETTE`].
    pub palette: String,
    pub dock: crate::ui::bars::Dock,
    /// Its window's top-left corner while floating (screen points).
    pub pos: [f32; 2],
    /// As [`PalettePrefs::float_size`].
    pub float_size: [f32; 2],
    /// As [`PalettePrefs::rows`].
    pub rows: u8,
    /// As [`PalettePrefs::swatch_size`].
    pub swatch_size: u8,
}

impl Default for PaletteBarPrefs {
    fn default() -> Self {
        PaletteBarPrefs {
            palette: String::new(),
            dock: crate::ui::bars::Dock::Bottom,
            pos: [320.0, 420.0],
            float_size: FLOAT_SIZE,
            rows: 1,
            swatch_size: SWATCH_DEFAULT,
        }
    }
}

impl PaletteBarPrefs {
    /// Out-of-range values of a hand-edited file brought back.
    pub fn sanitize(&mut self) {
        let d = PaletteBarPrefs::default();
        if !self.pos.iter().all(|v| v.is_finite() && v.abs() <= 1.0e5) {
            self.pos = d.pos;
        }
        if !self
            .float_size
            .iter()
            .all(|v| v.is_finite() && *v > 0.0 && *v <= 1.0e5)
        {
            self.float_size = d.float_size;
        }
        self.rows = self.rows.min(MAX_ROWS);
        self.swatch_size = self.swatch_size.clamp(SWATCH_MIN, SWATCH_MAX);
    }
}

pub fn swatch_resized(size: u8, steps: i32) -> u8 {
    let to = i32::from(size.clamp(SWATCH_MIN, SWATCH_MAX))
        .saturating_add(steps.saturating_mul(SWATCH_STEP));
    to.clamp(i32::from(SWATCH_MIN), i32::from(SWATCH_MAX)) as u8
}

impl Default for PalettePrefs {
    fn default() -> Self {
        PalettePrefs {
            auto_update_document: true,
            right_click_outline: true,
            show_no_color: true,
            show_document: true,
            show_palette: true,
            rows: 1,
            swatch_size: SWATCH_DEFAULT,
            float_size: FLOAT_SIZE,
        }
    }
}

/// The Time Tracker (Window > Toolbars > Time Tracker, Tools > Time
/// Tracker).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TimeTrackerPrefs {
    /// The Time Tracker toolbar is shown.
    pub show_bar: bool,
    /// The clock stops counting this many minutes after the last input.
    pub idle_minutes: u32,
}

impl Default for TimeTrackerPrefs {
    fn default() -> Self {
        TimeTrackerPrefs {
            show_bar: false,
            idle_minutes: 5,
        }
    }
}

impl TimeTrackerPrefs {
    /// The idle limit in seconds (1 to 120 minutes).
    pub fn idle_seconds(&self) -> f64 {
        f64::from(self.idle_minutes.clamp(1, 120)) * 60.0
    }
}

/// Backups (Options > Save).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BackupPrefs {
    /// Copy the file being replaced to `backup_of_<name>` before saving.
    pub before_save: bool,
    /// Where those copies go; none: next to the original.
    pub before_save_dir: Option<PathBuf>,
    /// Save open drawings with unsaved changes every `minutes`.
    pub auto: bool,
    pub minutes: u32,
    /// Where auto-backups go; none: the temporary folder.
    pub auto_dir: Option<PathBuf>,
}

impl Default for BackupPrefs {
    fn default() -> Self {
        BackupPrefs {
            before_save: true,
            before_save_dir: None,
            auto: true,
            minutes: 20,
            auto_dir: None,
        }
    }
}

fn default_object_thumb() -> f32 {
    crate::object_thumbs::DEFAULT_SIZE
}

fn default_font_sample() -> f32 {
    crate::ui::font_list::DEFAULT_SAMPLE
}

fn default_super_nudge() -> f64 {
    0.2 * 25.4
}

fn default_micro_nudge() -> f64 {
    0.01 * 25.4
}

/// Document settings saved as the default for new drawings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DocumentDefaults {
    pub grid: tracedraw_core::document::GridSettings,
    pub rulers: tracedraw_core::document::RulerSettings,
    pub guides: tracedraw_core::document::GuideSettings,
    /// The fill and outline new objects get.
    #[serde(default)]
    pub objects: tracedraw_core::ObjectDefaults,
    /// The space between pages in the Multipage View, mm (`None`: the
    /// default 30 mm).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_spacing: Option<f64>,
}

impl DocumentDefaults {
    pub fn of(doc: &tracedraw_core::Document) -> Self {
        DocumentDefaults {
            grid: doc.metadata.grid,
            rulers: doc.metadata.rulers.clone(),
            guides: doc.metadata.guides,
            objects: doc.defaults.clone(),
            page_spacing: doc.metadata.page_spacing,
        }
    }

    pub fn apply(&self, doc: &mut tracedraw_core::Document) {
        doc.metadata.grid = self.grid;
        doc.metadata.rulers = self.rulers.clone();
        doc.metadata.guides = self.guides;
        doc.metadata.page_spacing = self.page_spacing;
        doc.defaults = defaults_for(&self.objects, doc.metadata.primary_color_mode);
    }
}

/// The stored object defaults as a drawing in `mode` takes them: a fill
/// or outline that is still the untouched default black is the mode's
/// black (new objects are black in the drawing's own colour mode),
/// whatever else the stored defaults hold (a paragraph style, an outline
/// width of one's own); a colour the user picked, a black included,
/// stays exactly as saved. Settings saved before the mark existed: a
/// black outline of graphics or black text fill, where the built-in
/// defaults put one, counts as untouched
/// ([`tracedraw_core::DefaultProperties::follows`]).
fn defaults_for(
    stored: &tracedraw_core::ObjectDefaults,
    mode: tracedraw_core::document::PrimaryColorMode,
) -> tracedraw_core::ObjectDefaults {
    stored.in_mode(mode)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SnapPrefs {
    pub grid: bool,
    pub guides: bool,
    pub objects: bool,
    pub page: bool,
    pub threshold_px: f64,
    /// The snapping modes in use (Options > Snapping).
    pub modes: crate::snap_points::SnapModes,
    /// Draw the mark of the point snapped to.
    pub show_marks: bool,
    /// Name the point snapped to next to its mark.
    pub screen_tips: bool,
    /// Snap to whole pixels, to the baseline grid; alignment and dynamic
    /// guides; all snapping off (Alt+Q).
    pub pixels: bool,
    pub baseline_grid: bool,
    pub alignment_guides: bool,
    pub dynamic_guides: bool,
    pub off: bool,
    /// Options > Alignment Guides and Options > Dynamic Guides.
    pub align: crate::live_guides::AlignOptions,
    pub dynamic: crate::live_guides::DynamicOptions,
}

impl Default for SnapPrefs {
    fn default() -> Self {
        let s = crate::snap::SnapSettings::default();
        SnapPrefs {
            grid: s.grid,
            guides: s.guides,
            objects: s.objects,
            page: s.page,
            threshold_px: 10.0,
            modes: crate::snap_points::SnapModes::default(),
            show_marks: true,
            screen_tips: true,
            pixels: s.pixels,
            baseline_grid: s.baseline_grid,
            alignment_guides: s.alignment_guides,
            dynamic_guides: s.dynamic_guides,
            off: s.off,
            align: Default::default(),
            dynamic: Default::default(),
        }
    }
}

/// The tool settings new objects take (Options > Tools and the property
/// bars), kept between sessions. The defaults are those of a first start.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ToolPrefs {
    pub polygon_points: u32,
    pub star_sharpness: f64,
    pub star_complex: bool,
    pub complex_points: u32,
    pub complex_sharpness: u32,
    pub rect_corners: tracedraw_core::Corners,
    pub corners_together: bool,
    pub ellipse_arc: Option<tracedraw_core::EllipseArc>,
    pub spiral_revolutions: u32,
    pub spiral_logarithmic: bool,
    pub table_rows: u32,
    pub table_cols: u32,
    pub graph_rows: u32,
    pub graph_cols: u32,
    pub blend_steps: u32,
    /// The Contour tool's and panel's values.
    pub contour: crate::contour_tool::ContourSettings,
    pub eraser_width: f64,
    pub eraser_square: bool,
    pub text_font: String,
    pub text_size_pt: f64,
    pub text_hyphenation: bool,
    /// The envelope editing mode and the mapping and Keep Lines new
    /// envelopes take (ids of `tracedraw_core::envelope`).
    pub envelope_mode: String,
    pub envelope_mapping: String,
    pub envelope_keep_lines: bool,
    /// The grid new mesh fills take.
    pub mesh_rows: u32,
    pub mesh_cols: u32,
    /// Smart Fill's fill and outline of the objects it makes (0 the
    /// default, 1 these, 2 none).
    pub smart_fill_mode: u8,
    pub smart_fill_color: tracedraw_core::Color,
    pub smart_outline_mode: u8,
    pub smart_outline_width: f64,
    pub smart_outline_color: tracedraw_core::Color,
    /// The shaping brushes: nib radius (mm), Roughen's spike size (mm) and
    /// the rest of their bars.
    pub brush_radius: f64,
    pub roughen_amount: f64,
    pub brush: crate::ui::brush_bars::BrushSettings,
    /// The style new dimensions take.
    pub dimension_style: crate::dimensions::DimensionStyle,
    /// The style new 3-point callouts take.
    pub callout_style: crate::callouts::CalloutStyle,
    /// The Distort tool's values.
    pub distort: crate::distort_tool::DistortSettings,
    /// The Extrude tool's and panel's values.
    pub extrude: crate::extrude_tool::ExtrudeSettings,
    /// Rounded connectors' corner radius, mm.
    pub connector_radius: f64,
    /// Shape Recognition's levels and Sketch's settings.
    pub recognize: crate::recognize::RecognizeSettings,
    pub sketch: crate::recognize::SketchSettings,
    /// The Knife's mode, gap and pieces.
    pub knife: crate::knife::KnifeSettings,
    /// Segment Delete's drag is a freehand stroke, not a box (#1150).
    pub segment_freehand: bool,
}

impl Default for ToolPrefs {
    fn default() -> Self {
        ToolPrefs {
            polygon_points: 5,
            star_sharpness: 0.5,
            star_complex: false,
            complex_points: 9,
            complex_sharpness: 2,
            rect_corners: tracedraw_core::Corners::tool_default(),
            corners_together: true,
            ellipse_arc: None,
            spiral_revolutions: 4,
            spiral_logarithmic: false,
            table_rows: 3,
            table_cols: 4,
            graph_rows: 4,
            graph_cols: 3,
            blend_steps: 20,
            contour: Default::default(),
            eraser_width: 6.35,
            eraser_square: false,
            text_font: "Arial".into(),
            text_size_pt: 24.0,
            text_hyphenation: false,
            envelope_mode: tracedraw_core::envelope::Mode::default().id().into(),
            envelope_mapping: tracedraw_core::envelope::Mapping::default().id().into(),
            envelope_keep_lines: false,
            mesh_rows: 2,
            mesh_cols: 2,
            smart_fill_mode: 0,
            smart_fill_color: tracedraw_core::Color::cmyk_pct(0.0, 0.0, 100.0, 0.0),
            smart_outline_mode: 0,
            smart_outline_width: 0.2,
            smart_outline_color: tracedraw_core::Color::BLACK,
            brush_radius: 10.0,
            roughen_amount: 2.0,
            brush: Default::default(),
            dimension_style: Default::default(),
            callout_style: Default::default(),
            distort: Default::default(),
            extrude: Default::default(),
            connector_radius: 3.0,
            recognize: Default::default(),
            sketch: Default::default(),
            knife: Default::default(),
            segment_freehand: false,
        }
    }
}

/// What the drawing window shows (the View menu, Options > Display, the
/// document grid page), kept between sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ViewPrefs {
    pub grid: bool,
    pub guides: bool,
    pub page_border: bool,
    /// The page's shadow, apart from its border (#1906).
    pub page_shadow: bool,
    pub bleed: bool,
    pub printable_area: bool,
    /// View > Mask Outside Page (#615).
    pub mask_outside_page: bool,
    pub pixel_grid: bool,
    pub baseline_grid: bool,
    pub non_printing: bool,
    pub proof_colors: bool,
    pub simulate_overprints: bool,
    pub rasterize_complex_effects: bool,
    /// Wireframe, normal, enhanced or pixels (`ViewMode::id`).
    pub mode: String,
    /// The grid page shows lines per unit instead of their spacing.
    pub grid_frequency: bool,
}

impl Default for ViewPrefs {
    fn default() -> Self {
        ViewPrefs {
            grid: false,
            guides: true,
            page_border: true,
            page_shadow: true,
            bleed: false,
            printable_area: false,
            mask_outside_page: false,
            pixel_grid: true,
            baseline_grid: false,
            non_printing: false,
            proof_colors: false,
            simulate_overprints: false,
            rasterize_complex_effects: true,
            mode: "enhanced".into(),
            grid_frequency: false,
        }
    }
}

/// Dockers' own choices kept between sessions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PanelPrefs {
    /// The Color docker's slider model (`Color::convert_to` names) and
    /// its viewer, sliders and palette views.
    pub color_model: String,
    pub color_views: [bool; 3],
    /// Align and Distribute: the target (`AlignTo::id`), Use Outline and
    /// where Distribute spreads (`DistributeTo::id`).
    pub align_to: String,
    pub align_use_outline: bool,
    pub distribute_to: String,
    /// The Shaping docker's operation (`Shaping::id`) and what Weld,
    /// Trim and Intersect leave: [source objects, target] each.
    pub shaping_op: String,
    pub shaping_leave: [[bool; 2]; 3],
}

impl Default for PanelPrefs {
    fn default() -> Self {
        PanelPrefs {
            color_model: "CMYK".into(),
            color_views: [true, true, false],
            align_to: "active".into(),
            align_use_outline: false,
            distribute_to: "selection".into(),
            shaping_op: "weld".into(),
            shaping_leave: [[false, false], [true, false], [true, true]],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorPrefs {
    pub rgb_profile: String,
    pub cmyk_profile: String,
    pub intent: String,
    pub black_point_compensation: bool,
    pub proof_profile: String,
    /// Paths of ICC files loaded by the user (empty = built-in model).
    #[serde(default)]
    pub rgb_profile_path: String,
    #[serde(default)]
    pub cmyk_profile_path: String,
    /// The default grayscale profile (name, and file when loaded).
    pub gray_profile: String,
    pub gray_profile_path: String,
    /// The colour preset the defaults came from (Default tab).
    pub preset: String,
    /// Presets the user saved.
    pub presets: Vec<ColorPreset>,
    /// New drawings' primary colour mode: "CMYK" or "RGB".
    pub primary_color_mode: String,
    /// The colour engine: the ICC engine, or the built-in formulas.
    pub builtin_engine: bool,
    /// Converting CMYK to CMYK keeps colours made of black ink only.
    pub preserve_pure_black: bool,
    /// Converting neutral greys to CMYK uses black ink only.
    pub map_gray_to_black: bool,
    /// Open policy per colour space: 0 use the drawing's embedded profile,
    /// 1 assign the default profile (values kept), 2 convert to the
    /// default profile.
    pub open_rgb: u8,
    pub open_cmyk: u8,
    pub open_gray: u8,
    pub warn_open_mismatch: bool,
    pub warn_open_missing: bool,
    /// Import and paste policy per colour space: 0 convert to the
    /// drawing's profile, 1 assign the drawing's profile (values kept).
    /// CMYK also supports 2: convert the destination to the incoming profile.
    pub import_rgb: u8,
    pub import_cmyk: u8,
    pub import_gray: u8,
    pub warn_import_mismatch: bool,
    pub warn_import_missing: bool,
    /// How spot colours are defined: 0 Lab values, 1 CMYK values, 2 RGB
    /// values (their process stand-in on screen and in conversions).
    pub spot_definition: u8,
}

/// A saved set of colour defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ColorPreset {
    pub name: String,
    pub rgb_profile: String,
    pub rgb_profile_path: String,
    pub cmyk_profile: String,
    pub cmyk_profile_path: String,
    pub gray_profile: String,
    pub gray_profile_path: String,
    pub primary_color_mode: String,
    pub intent: String,
}

impl Default for ColorPrefs {
    fn default() -> Self {
        ColorPrefs {
            rgb_profile: "sRGB IEC61966-2.1".into(),
            cmyk_profile: "Generic CMYK (open)".into(),
            intent: "Relative colorimetric".into(),
            black_point_compensation: true,
            proof_profile: "Generic CMYK (open)".into(),
            rgb_profile_path: String::new(),
            cmyk_profile_path: String::new(),
            gray_profile: "Gray Gamma 2.2".into(),
            gray_profile_path: String::new(),
            preset: String::new(),
            presets: Vec::new(),
            primary_color_mode: "CMYK".into(),
            builtin_engine: false,
            preserve_pure_black: false,
            map_gray_to_black: true,
            open_rgb: 0,
            open_cmyk: 0,
            open_gray: 0,
            warn_open_mismatch: false,
            warn_open_missing: false,
            import_rgb: 0,
            import_cmyk: 0,
            import_gray: 0,
            warn_import_mismatch: false,
            warn_import_missing: false,
            spot_definition: 0,
        }
    }
}

/// Options > Text > Spelling: what the spell checker skips, whether its
/// replacements go to the Autocorrect table, and which dictionaries it
/// uses.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct SpellingPrefs {
    /// Skip words with digits ("B2B").
    pub ignore_digits: bool,
    /// Skip words in capitals ("NASA").
    pub ignore_capitals: bool,
    /// A word the spell checker replaces is added to the Autocorrect
    /// replacements, so typing it wrong again corrects it.
    pub add_to_autocorrect: bool,
    /// A folder of Hunspell dictionaries besides Craft Corel's own.
    pub dictionary_folder: Option<PathBuf>,
    /// The dictionaries switched off, by name (`pt_BR`).
    pub disabled_dictionaries: Vec<String>,
    /// The dictionaries downloaded (Options > Text > Spelling), by their
    /// code in the collection (`pt`, `en-GB`): a browser fetches them
    /// again at each visit, as it has no folder to keep them in (#1405).
    pub downloaded_dictionaries: Vec<String>,
    /// Options > Text > Paragraph Text: the hyphenation pattern file used
    /// (`hyph_pt_BR`, found where the dictionaries are), empty for the
    /// built-in patterns.
    pub hyphenation_patterns: String,
}

impl SpellingPrefs {
    /// The dictionaries these options name, for the spell checker.
    pub fn config(&self) -> crate::spell::SpellConfig {
        crate::spell::SpellConfig {
            folder: self.dictionary_folder.clone(),
            disabled: self.disabled_dictionaries.clone(),
        }
    }
}

/// The fonts used last that the font lists show, by default.
pub fn default_recent_font_count() -> usize {
    5
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: String::new(),
            recent_files: Vec::new(),
            workspace: "default".into(),
            show_welcome_on_start: true,
            welcome_tab_closed: false,
            crop_picture_pixels: false,
            table: Default::default(),
            units: "mm".into(),
            nudge_mm: 2.54,
            super_nudge_mm: default_super_nudge(),
            micro_nudge_mm: default_micro_nudge(),
            // In place: the next Duplicate repeats what is done to the
            // copy (#1494).
            duplicate_offset_mm: [0.0, 0.0],
            default_dpi: 300.0,
            snap: SnapPrefs::default(),
            shortcuts: Vec::new(),
            bitmap_editor: String::new(),
            last_dir: None,
            folder_bookmarks: Vec::new(),
            color: ColorPrefs::default(),
            show_outline_flyout: false,
            wheel_zooms: true,
            open_view: OpenView::default(),
            toolbox: Vec::new(),
            custom_arrowheads: Vec::new(),
            custom_dashes: Vec::new(),
            page_sizes: Vec::new(),
            outline_units: default_outline_units(),
            text_units: default_outline_units(),
            document_defaults: DocumentDefaults::default(),
            startup: Startup::default(),
            undo_levels: default_undo_levels(),
            show_tooltips: true,
            font_manager: Default::default(),
            tracecut: Default::default(),
            hide_bbox_curve_tools: false,
            node_tracking: false,
            preview_page_border: true,
            constrain_angle: default_constrain_angle(),
            symmetry_show_lines: true,
            symmetry_full_preview: false,
            corners_preview: true,
            precision: default_precision(),
            nodes: NodePrefs::default(),
            clip_frame: ClipFramePrefs::default(),
            backup: BackupPrefs::default(),
            text_increment_pt: default_text_increment(),
            default_tab_mm: default_tab_spacing(),
            treat_all_filled: true,
            focus_visibility: default_focus_visibility(),
            focus_double_click: true,
            comment_author: String::new(),
            follow_text_size: true,
            follow_high_contrast: true,
            marquee_touching: false,
            upgrades: UPGRADES,
            crosshair_cursor: false,
            screen_ppi: default_screen_ppi(),
            zoom_right_click_out: true,
            zoom_relative_1to1: true,
            wheel_zoom_centers: false,
            pan_right_click_out: false,
            palette: PalettePrefs::default(),
            palette_bars: Vec::new(),
            main_palette: String::new(),
            palette_names: Default::default(),
            user_palettes: Vec::new(),
            tools: ToolPrefs::default(),
            view: ViewPrefs::default(),
            panels: PanelPrefs::default(),
            autocorrect: crate::autocorrect::AutocorrectPrefs::default(),
            spelling: SpellingPrefs::default(),
            thesaurus_file: None,
            show_new_document_dialog: true,
            show_scrollbars: true,
            new_document: crate::new_document::NewDocSettings::default(),
            desktop_rgb: default_desktop(),
            desktop_custom: false,
            status_info: crate::ui::status::StatusInfo::default(),
            toolbox_hidden: Vec::new(),
            button_size: ButtonSize::Small,
            fit_page_margin: 0.0,
            fit_page_preview: true,
            toolbox_button_size: None,
            toolbox_captions: Captions::ImageOnly,
            ui_size: 0.0,
            show_splash: true,
            hardware_acceleration: true,
            gpu_adapter: String::new(),
            working_renderer: String::new(),
            center_dialogs: true,
            ui_shadows: true,
            colored_icons: false,
            theme: String::new(),
            bars: crate::ui::bars::BarLayout::default(),
            bar_items: Default::default(),
            window_maximized: true,
            window_size: [0.0, 0.0],
            time_tracker: TimeTrackerPrefs::default(),
            preflight: crate::preflight::PreflightPrefs::default(),
            publish_pdf: crate::ui::publish_pdf::PublishPrefs::default(),
            defaults: crate::defaults::DefaultsPrefs::default(),
            shown_bars: None,
            dockers: None,
            recent_fonts: Vec::new(),
            show_recent_fonts: true,
            recent_font_count: default_recent_font_count(),
            properties_closed: Vec::new(),
            text_sections_closed: Vec::new(),
            text_docker_from: 0,
            text_docker_checks: false,
            text_docker_ticked: Vec::new(),
            recent_glyphs: Vec::new(),
            custom_hyphens: Vec::new(),
            hyphens_when_typing: true,
            hyphens_when_pasting: true,
            object_thumb_size: default_object_thumb(),
            master_in_front: [false; 3],
            objects_all_pages: true,
            font_sample_px: default_font_sample(),
            workspace_parts: crate::workspace_file::WorkspaceParts::default(),
            printing: Default::default(),
            formats: Default::default(),
            ai: Default::default(),
            ai_enabled: true,
            ai_image: None,
            mcp_window: false,
            check_updates: false,
            update_check_asked: true,
            last_update_check: 0,
            skipped_update: String::new(),
            pending_update: String::new(),
            script_filters: Vec::new(),
            github: Default::default(),
            tracing: Default::default(),
            warnings: Default::default(),
            curves: Default::default(),
            show_text_frames: true,
            path_tick_snap: false,
            path_tick_mm: default_path_tick(),
            screen_margins: crate::ui::screen_margins::ScreenMargins::default(),
        }
    }
}

/// The folder of a portable copy: a folder called `tracedraw-settings`
/// next to the program keeps its preferences there (a USB stick, a copy
/// that leaves no trace on the computer).
pub const PORTABLE_DIR: &str = "craft-corel-settings";

/// The portable settings folder next to the program `exe`, when there is
/// one.
pub fn portable_dir(exe: &std::path::Path) -> Option<PathBuf> {
    let dir = exe.parent()?.join(PORTABLE_DIR);
    dir.is_dir().then_some(dir)
}

pub fn config_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("TRACEDRAW_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(dir) = std::env::current_exe().ok().and_then(|e| portable_dir(&e)) {
        return Some(dir);
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Craft Corel"))
    }
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join("Library/Application Support/Craft Corel"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")))
            .map(|p| p.join("craft-corel"))
    }
}

/// Local storage key of the settings in the browser build.
#[cfg(target_arch = "wasm32")]
const WEB_KEY: &str = "tracedraw.settings";

impl Settings {
    /// The preferences as Craft Corel comes, keeping what is history rather
    /// than choice: the recent drawings, the last folder, the bookmarked
    /// folders and the window's size.
    pub fn reset_keeping_history(&self) -> Settings {
        Settings {
            recent_files: self.recent_files.clone(),
            last_dir: self.last_dir.clone(),
            folder_bookmarks: self.folder_bookmarks.clone(),
            window_maximized: self.window_maximized,
            window_size: self.window_size,
            ..Settings::default()
        }
    }

    pub fn path() -> Option<PathBuf> {
        config_dir().map(|d| d.join("settings.json"))
    }

    /// Bring preferences saved by an older version up to date, once:
    /// 1. objects are treated as filled (a click inside an unfilled one
    ///    selects it), which older versions saved switched off;
    /// 2. new rectangles keep their corners; 3. the alignment and dynamic
    ///    guides start off; 4. the last export's settings are remembered;
    /// 5. Duplicate's old default offset (6.35 mm) becomes none.
    pub fn upgrade(mut self) -> Settings {
        self.autocorrect.upgrade();
        if self.upgrades < 1 {
            self.treat_all_filled = true;
        }
        // Older versions had no way to save corners that keep their size
        // as the tool's default (scaling corners were the only default and
        // were not written down): new rectangles now keep their corners.
        if self.upgrades < 2 {
            self.tools.rect_corners.fixed = true;
        }
        // Versions before the alignment and dynamic guides existed kept
        // both switches on, with nothing behind them; once the guides
        // came they lit up for everyone with an older file. Off once,
        // as for a first start; turned on afterwards they stay on.
        if self.upgrades < 3 {
            self.snap.alignment_guides = false;
            self.snap.dynamic_guides = false;
        }
        // Remember the settings of the last export came on for new
        // preferences only (#617); files saved before kept the old off,
        // so for everyone who had used an earlier version File > Export
        // still started at SVG in each new session and the feature could
        // not be found (#691). On once; switched off afterwards it stays.
        if self.upgrades < 4 {
            self.formats.remember_last = true;
        }
        // Duplicate placed its copy 6.35 mm off by default; the copy now
        // stays in place to be moved, scaled or turned, and the next
        // Duplicate repeats that (#1494). An offset of one's own stays.
        if self.upgrades < 5 && self.duplicate_offset_mm == [6.35, 6.35] {
            self.duplicate_offset_mm = [0.0, 0.0];
        }
        self.upgrades = self.upgrades.max(UPGRADES);
        self.migrate_ai();
        migrate_command_keys(&mut self.shortcuts);
        self
    }

    /// Fold the separate image service of older versions into the one AI
    /// service. Safe to run again.
    pub fn migrate_ai(&mut self) {
        if let Some(image) = self.ai_image.take() {
            let (ai, kept) = migrate_ai(self.ai.clone(), image);
            self.ai = ai;
            self.ai_image = kept;
        }
    }

    pub fn load() -> Settings {
        // Unit tests start from the defaults, whatever an earlier run of
        // the program left in the folder they share (see `save`).
        if cfg!(test) {
            return Settings::default();
        }
        #[cfg(target_arch = "wasm32")]
        {
            // The browser build keeps settings in local storage.
            return crate::web::storage_get(WEB_KEY)
                .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
                .map(Settings::upgrade)
                .unwrap_or_default();
        }
        #[allow(unreachable_code)]
        let Some(p) = Self::path() else {
            return Settings::default();
        };
        match std::fs::read_to_string(&p) {
            Ok(s) => {
                let mut settings: Settings = serde_json::from_str(&s).unwrap_or_else(|e| {
                    log::warn!("settings unreadable ({e}); using defaults");
                    Settings::default()
                });
                // Files written before the start-up choice existed.
                if !s.contains("\"startup\"") && !settings.show_welcome_on_start {
                    settings.startup = Startup::NewDocument;
                }
                settings.upgrade()
            }
            Err(_) => Settings::default(),
        }
    }

    pub fn save(&self) {
        #[cfg(target_arch = "wasm32")]
        {
            if let Ok(s) = serde_json::to_string(self) {
                // A refusal is told to the user; the settings stay in
                // effect in this tab (#1414).
                if let Err(full) = crate::web::storage_set(WEB_KEY, &s) {
                    crate::storage_full::report(crate::storage_full::Kept::Settings, full);
                }
            }
            return;
        }
        #[allow(unreachable_code)]
        let Some(p) = Self::path() else {
            return;
        };
        match serde_json::to_string_pretty(self) {
            Ok(s) => {
                // Unit tests share one settings folder (`App::headless`):
                // preferences a test changes (shortcuts, bars shown, an
                // imported workspace) must not reach the tests that start
                // after it, so they never write the file.
                if cfg!(test) {
                    return;
                }
                if let Some(dir) = p.parent() {
                    let _ = std::fs::create_dir_all(dir);
                }
                if let Err(e) = std::fs::write(&p, s) {
                    log::warn!("could not save settings: {e}");
                }
            }
            Err(e) => log::warn!("could not serialise settings: {e}"),
        }
    }

    /// Move or insert `path` at the front of the recent list (max 10).
    pub fn touch_recent(&mut self, path: &std::path::Path) {
        // Browser uploads and downloads have no path to reopen.
        if crate::files::WEB {
            return;
        }
        self.recent_files.retain(|p| p != path);
        self.recent_files.insert(0, path.to_path_buf());
        self.recent_files.truncate(10);
    }
}

/// Command keys that earlier versions wrote into a shortcut list under
/// another name, and the key they go by now. A key is renamed only when a
/// name had to go; the menus keep their keys otherwise, so a saved
/// shortcut keeps working whatever its command is called or where it sits.
pub const RENAMED_COMMAND_KEYS: [(&str, &str); 1] = [("menu.text.grammatik", "menu.text.grammar")];

/// Bring the command keys of a shortcut list (the preferences' or a
/// workspace file's) up to date. Safe to run again.
pub fn migrate_command_keys(shortcuts: &mut [(String, String)]) {
    for (key, _) in shortcuts.iter_mut() {
        if let Some((_, now)) = RENAMED_COMMAND_KEYS.iter().find(|(old, _)| *old == key) {
            *key = (*now).to_string();
        }
    }
}

/// The one AI service from the drawing service (`ai`) and the separate
/// image service (`image`) older versions saved, and the image service
/// to keep in the file when its remembered key could not be taken (see
/// `Settings::ai_image`).
///
/// The drawing service stays when it was set up; it takes the image
/// model (and the remembered key, when it had none) of an image service
/// of the same kind. When only the image service was set up, it becomes
/// the one service: Gemini as the Gemini preset, OpenAI as the OpenAI
/// preset or, at another address, an OpenAI-compatible address with the
/// image model kept.
pub fn migrate_ai(
    ai: crate::ai::AiPrefs,
    image: crate::ai_image::ImagePrefs,
) -> (crate::ai::AiPrefs, Option<crate::ai_image::ImagePrefs>) {
    use crate::ai::{AiPrefs, ImageSupport, Provider};
    use crate::ai_image::{ImagePrefs, ImageProvider};
    if image == ImagePrefs::default() {
        return (ai, None);
    }
    let image_key = if image.remember_key {
        image.key.trim().to_string()
    } else {
        String::new()
    };
    let defaults = AiPrefs {
        show_bar: ai.show_bar,
        ..AiPrefs::default()
    };
    if ai != defaults {
        // The drawing service was set up: it stays.
        let mut ai = ai;
        let same_kind = match (ai.image_support(), image.provider) {
            (ImageSupport::Gemini, ImageProvider::Gemini) => image.endpoint.trim().is_empty(),
            (ImageSupport::OpenAi, ImageProvider::OpenAi) => {
                crate::ai::api_base(&image.endpoint()) == crate::ai::OPENAI_API_BASE
            }
            (ImageSupport::Custom { base }, ImageProvider::OpenAi) => {
                crate::ai::api_base(&image.endpoint()) == base
            }
            _ => false,
        };
        if !same_kind {
            // Its remembered key stays in the file for the user to find.
            return (ai, (!image_key.is_empty()).then_some(image));
        }
        if ai.image_model.trim().is_empty() {
            ai.image_model = match ai.image_support() {
                // An address of the user's makes pictures only with a
                // model named.
                ImageSupport::Custom { .. } => image.model(),
                _ => image.model.trim().to_string(),
            };
        }
        let drawing_key = crate::ai::remembered_key(&ai);
        if drawing_key.trim().is_empty() && !image_key.is_empty() {
            ai.key = image_key;
            ai.remember_key = true;
            return (ai, None);
        }
        let lost = !image_key.is_empty() && image_key != drawing_key;
        return (ai, lost.then_some(image));
    }
    // Only the image service was set up: it becomes the one service.
    let mut out = AiPrefs {
        provider: Provider::OpenAiCompatible,
        key: image_key.clone(),
        remember_key: image.remember_key && !image_key.is_empty(),
        show_bar: ai.show_bar,
        ..AiPrefs::default()
    };
    let preset_address = |key: &str| {
        crate::ai::PRESETS
            .iter()
            .find(|(k, _, _)| *k == key)
            .map(|(_, _, e)| e.to_string())
            .unwrap_or_default()
    };
    match image.provider {
        ImageProvider::Gemini => {
            out.endpoint = preset_address("aipreset.gemini");
            out.image_model = image.model.trim().to_string();
        }
        ImageProvider::OpenAi => {
            let base = crate::ai::api_base(&image.endpoint());
            if base == crate::ai::OPENAI_API_BASE {
                out.endpoint = preset_address("aipreset.openai");
                out.image_model = image.model.trim().to_string();
            } else {
                out.endpoint = format!("{base}/chat/completions");
                // The same server under another name when the address is
                // that of a chat-only preset, so that it stays one that
                // can make pictures.
                if !out.can_make_pictures() {
                    out.endpoint = if out.endpoint.contains("//localhost") {
                        out.endpoint.replacen("//localhost", "//127.0.0.1", 1)
                    } else {
                        out.endpoint.replacen("//127.0.0.1", "//localhost", 1)
                    };
                }
                out.image_model = image.model();
            }
        }
    }
    (out, None)
}

fn ai_enabled_default() -> bool {
    true
}

fn default_focus_visibility() -> u8 {
    20
}

#[cfg(test)]
mod tests {
    /// Stored defaults from an earlier version (RGB black text, and any
    /// other field of their time) give a CMYK drawing CMYK black text and
    /// an RGB drawing RGB black text; a colour of the user's own stays.
    #[test]
    fn stored_black_text_defaults_follow_the_drawings_colour_mode() {
        use tracedraw_core::document::{ObjectDefaults, PrimaryColorMode};
        use tracedraw_core::{Color, Fill};
        let mut old = ObjectDefaults::for_mode(PrimaryColorMode::Rgb);
        // Something an earlier version kept differently: not "untouched".
        old.paragraph.hyphenate = !old.paragraph.hyphenate;
        let cmyk = super::defaults_for(&old, PrimaryColorMode::Cmyk);
        assert_eq!(
            cmyk.artistic_text.fill,
            Fill::Solid(PrimaryColorMode::Cmyk.black())
        );
        assert_eq!(
            cmyk.paragraph_text.fill,
            Fill::Solid(PrimaryColorMode::Cmyk.black())
        );
        assert_eq!(cmyk.paragraph, old.paragraph, "the rest is kept");
        let rgb = super::defaults_for(&old, PrimaryColorMode::Rgb);
        assert_eq!(rgb.artistic_text.fill, Fill::Solid(Color::BLACK));
        let mut red = old.clone();
        red.artistic_text.fill = Fill::Solid(Color::rgb8(200, 0, 0));
        let kept = super::defaults_for(&red, PrimaryColorMode::Cmyk);
        assert_eq!(kept.artistic_text.fill, Fill::Solid(Color::rgb8(200, 0, 0)));
        // Through the document defaults as a drawing opens.
        let mut doc = tracedraw_core::Document::default();
        doc.metadata.primary_color_mode = PrimaryColorMode::Cmyk;
        let prefs = super::DocumentDefaults {
            objects: old,
            ..Default::default()
        };
        prefs.apply(&mut doc);
        assert_eq!(
            doc.defaults.artistic_text.fill,
            Fill::Solid(PrimaryColorMode::Cmyk.black())
        );
        // The CMYK black outline stored for graphics turns RGB black in
        // an RGB drawing, a width of one's own stays (#1665).
        let mut stored = ObjectDefaults::default();
        if let Some(s) = stored.graphic.stroke.as_mut() {
            s.width = 0.5;
        }
        let rgb = super::defaults_for(&stored, PrimaryColorMode::Rgb);
        assert_eq!(
            rgb.graphic.stroke.as_ref().map(|s| (s.color, s.width)),
            Some((Color::BLACK, 0.5))
        );
    }

    /// Saved defaults: the untouched default black follows the new
    /// drawing's mode, a black the user picked stays exactly as saved,
    /// and so does any other colour (#1665).
    #[test]
    fn saved_defaults_keep_chosen_blacks() {
        use crate::defaults::DefaultChange;
        use tracedraw_core::document::PrimaryColorMode;
        use tracedraw_core::{Color, Fill};
        let k100 = PrimaryColorMode::Cmyk.black();
        let outline =
            |d: &tracedraw_core::ObjectDefaults| d.graphic.stroke.as_ref().map(|s| s.color);
        let open_in = |prefs: &super::DocumentDefaults, mode| {
            let mut doc = tracedraw_core::Document::default();
            doc.metadata.primary_color_mode = mode;
            prefs.apply(&mut doc);
            doc.defaults
        };
        // Untouched defaults saved from a CMYK drawing follow both modes.
        let mut app = crate::app::App::headless();
        app.settings.new_document.values.rgb = false;
        app.new_document();
        let untouched = super::DocumentDefaults::of(app.doc());
        let rgb = open_in(&untouched, PrimaryColorMode::Rgb);
        assert_eq!(outline(&rgb), Some(Color::BLACK));
        assert_eq!(rgb.artistic_text.fill, Fill::Solid(Color::BLACK));
        let cmyk = open_in(&untouched, PrimaryColorMode::Cmyk);
        assert_eq!(outline(&cmyk), Some(k100));
        // The user picks CMYK black for graphics' outline: it stays CMYK
        // in an RGB drawing, the untouched text black still follows.
        app.change_defaults(
            &DefaultChange::OutlineColor(Some(k100)),
            [true, false, false],
        );
        let chosen = super::DocumentDefaults::of(app.doc());
        let json = serde_json::to_string(&chosen).unwrap();
        let chosen: super::DocumentDefaults = serde_json::from_str(&json).unwrap();
        let rgb = open_in(&chosen, PrimaryColorMode::Rgb);
        assert_eq!(outline(&rgb), Some(k100));
        assert_eq!(rgb.artistic_text.fill, Fill::Solid(Color::BLACK));
        // A saved colour that is not black stays.
        let blue = Color::rgb8(0, 0, 200);
        app.change_defaults(
            &DefaultChange::Fill(Fill::Solid(blue)),
            [false, true, false],
        );
        let rgb = open_in(
            &super::DocumentDefaults::of(app.doc()),
            PrimaryColorMode::Rgb,
        );
        assert_eq!(rgb.artistic_text.fill, Fill::Solid(blue));
        // Reset in the Object Styles panel makes the outline follow again.
        app.set_default_properties(tracedraw_core::DefaultKind::Graphic, |p| {
            *p = tracedraw_core::ObjectDefaults::for_mode(PrimaryColorMode::Cmyk).graphic
        });
        let rgb = open_in(
            &super::DocumentDefaults::of(app.doc()),
            PrimaryColorMode::Rgb,
        );
        assert_eq!(outline(&rgb), Some(Color::BLACK));
    }

    /// Settings saved before the mark existed: the stock CMYK black
    /// outline and text black (the old built-in defaults) follow the
    /// drawing's mode; a black elsewhere and other colours stay.
    #[test]
    fn old_settings_with_the_stock_blacks_follow() {
        use tracedraw_core::document::PrimaryColorMode;
        use tracedraw_core::{Color, Fill};
        let k100 = PrimaryColorMode::Cmyk.black();
        let mut stored = super::DocumentDefaults::default();
        stored.objects.graphic.fill = Fill::Solid(k100);
        stored.objects.paragraph_text.fill = Fill::Solid(Color::rgb8(0, 120, 0));
        let mut json = serde_json::to_value(&stored).unwrap();
        for kind in ["graphic", "artistic_text", "paragraph_text"] {
            json["objects"][kind]
                .as_object_mut()
                .unwrap()
                .remove("follow");
        }
        let old: super::DocumentDefaults = serde_json::from_value(json).unwrap();
        assert!(old.objects.graphic.follow.is_none());
        let mut doc = tracedraw_core::Document::default();
        doc.metadata.primary_color_mode = PrimaryColorMode::Rgb;
        old.apply(&mut doc);
        let d = &doc.defaults;
        assert_eq!(
            d.graphic.stroke.as_ref().map(|s| s.color),
            Some(Color::BLACK)
        );
        assert_eq!(d.artistic_text.fill, Fill::Solid(Color::BLACK));
        assert_eq!(d.graphic.fill, Fill::Solid(k100), "a black of one's own");
        assert_eq!(d.paragraph_text.fill, Fill::Solid(Color::rgb8(0, 120, 0)));
    }

    #[test]
    fn the_page_spacing_is_saved_as_a_default_for_new_drawings() {
        let mut doc = tracedraw_core::Document::default();
        // Settings saved before the field existed keep the default.
        let old: super::DocumentDefaults = serde_json::from_str("{}").unwrap();
        old.apply(&mut doc);
        assert_eq!(doc.metadata.page_spacing(), 30.0);
        doc.metadata.page_spacing = Some(10.0);
        let saved = super::DocumentDefaults::of(&doc);
        let json = serde_json::to_string(&saved).unwrap();
        let back: super::DocumentDefaults = serde_json::from_str(&json).unwrap();
        let mut fresh = tracedraw_core::Document::default();
        back.apply(&mut fresh);
        assert_eq!(fresh.metadata.page_spacing(), 10.0);
    }

    #[test]
    fn the_desktop_text_size_and_high_contrast_are_followed_unless_turned_off() {
        use crate::system_prefs::{ContrastColors, SystemPrefs};
        let mut s = Settings::default();
        s.ui_size = 1.0;
        let big = SystemPrefs {
            text_scale: 1.5,
            high_contrast: Some(ContrastColors::BLACK),
            text_scale_known: true,
            contrast_known: true,
        };
        assert!((effective_ui_size_with(&s, big) - 1.5).abs() < 1e-6);
        assert_eq!(effective_theme_with(&s, big), "high_contrast");
        s.follow_text_size = false;
        s.follow_high_contrast = false;
        assert!((effective_ui_size_with(&s, big) - 1.0).abs() < 1e-6);
        assert_eq!(effective_theme_with(&s, big), s.theme);
        // Never past the largest interface size.
        s.follow_text_size = true;
        s.ui_size = 3.0;
        assert!(effective_ui_size_with(&s, big) <= 4.0);
        assert_eq!(effective_theme_with(&s, SystemPrefs::default()), s.theme);
    }

    #[test]
    fn older_preferences_get_objects_treated_as_filled_once() {
        // Saved by an older version: switched off, no upgrades recorded.
        let old: Settings =
            serde_json::from_str(r#"{"treat_all_filled": false}"#).expect("old preferences");
        assert_eq!(old.upgrades, 0);
        let s = old.upgrade();
        assert!(s.treat_all_filled);
        assert_eq!(s.upgrades, UPGRADES);
        // Switched off again after the upgrade, it stays off.
        let mut s = s;
        s.treat_all_filled = false;
        let saved = serde_json::to_string(&s).expect("saved");
        let again: Settings = serde_json::from_str(&saved).expect("read back");
        assert!(!again.upgrade().treat_all_filled);
        // New preferences start up to date.
        assert_eq!(Settings::default().upgrades, UPGRADES);
    }

    /// Duplicate's old default offset becomes none, once; an offset of
    /// one's own stays, and none set again later stays (#1494).
    #[test]
    fn the_old_default_duplicate_offset_becomes_none_once() {
        let old: Settings =
            serde_json::from_str(r#"{"upgrades": 4, "duplicate_offset_mm": [6.35, 6.35]}"#)
                .expect("old preferences");
        let s = old.upgrade();
        assert_eq!(s.duplicate_offset_mm, [0.0, 0.0]);
        assert_eq!(s.upgrades, UPGRADES);
        let own: Settings =
            serde_json::from_str(r#"{"upgrades": 4, "duplicate_offset_mm": [5.0, 2.0]}"#)
                .expect("old preferences");
        assert_eq!(own.upgrade().duplicate_offset_mm, [5.0, 2.0]);
        let mut s = Settings::default();
        assert_eq!(s.duplicate_offset_mm, [0.0, 0.0]);
        s.duplicate_offset_mm = [6.35, 6.35];
        let saved = serde_json::to_string(&s).expect("saved");
        let again: Settings = serde_json::from_str(&saved).expect("read back");
        assert_eq!(again.upgrade().duplicate_offset_mm, [6.35, 6.35]);
    }

    /// Preferences saved before the last export's settings were kept
    /// between sessions stored the old off (the option existed with that
    /// default), so File > Export went on starting at SVG every session
    /// for anyone upgrading, and the feature of #617 seemed missing
    /// (#691). The upgrade turns it on once; off again afterwards, it
    /// stays off.
    #[test]
    fn older_preferences_get_the_last_export_remembered_once() {
        let old: Settings = serde_json::from_str(
            r#"{"upgrades": 3, "formats": {"remember_last": false, "export_format": 3}}"#,
        )
        .expect("old preferences");
        assert!(!old.formats.remember_last);
        let s = old.upgrade();
        assert!(s.formats.remember_last);
        assert_eq!(s.formats.export_format, 3, "the other choices stay");
        assert_eq!(s.upgrades, UPGRADES);
        let mut s = s;
        s.formats.remember_last = false;
        let saved = serde_json::to_string(&s).expect("saved");
        let again: Settings = serde_json::from_str(&saved).expect("read back");
        assert!(!again.upgrade().formats.remember_last);
        assert!(Settings::default().formats.remember_last);
    }

    #[test]
    fn rectangle_corners_keep_their_size_by_default_and_after_upgrading() {
        // A first start: new rectangles keep their corners when resized.
        assert!(Settings::default().tools.rect_corners.fixed);
        // Saved by an older version, which never wrote the value down.
        let old: Settings = serde_json::from_str(
            r#"{"upgrades": 1, "tools": {"rect_corners": {"radii": [2.0, 2.0, 2.0, 2.0], "kind": "round"}}}"#,
        )
        .expect("old preferences");
        assert!(!old.tools.rect_corners.fixed);
        let s = old.upgrade();
        assert!(s.tools.rect_corners.fixed);
        assert_eq!(s.tools.rect_corners.radii, [2.0; 4]);
        // Relative scaling turned on afterwards stays on.
        let mut s = s;
        s.tools.rect_corners.fixed = false;
        let saved = serde_json::to_string(&s).expect("saved");
        let again: Settings = serde_json::from_str(&saved).expect("read back");
        assert!(!again.upgrade().tools.rect_corners.fixed);
        // A file with no tool settings at all starts from the defaults.
        let bare: Settings = serde_json::from_str(r#"{"language":"en"}"#).expect("bare");
        assert!(bare.upgrade().tools.rect_corners.fixed);
    }

    use super::*;

    /// Each button size draws a larger icon in a larger button than the
    /// one before, and the padding around the toolbox's icon grows no
    /// faster than the icon (#1701 follow-up: the icons stayed one
    /// size). The property bar's does too up to Medium; at Large and
    /// Extra Large its icons keep a wider margin on purpose
    /// (`large_bar_icons_keep_a_margin`).
    #[test]
    fn each_button_size_has_a_larger_icon_and_proportional_padding() {
        let m: Vec<ButtonMetrics> = ButtonSize::ALL.iter().map(|s| s.metrics()).collect();
        assert_eq!(
            m.iter().map(|m| m.icon).collect::<Vec<_>>(),
            vec![16.0, 20.0, 24.0, 32.0, 40.0]
        );
        for (i, w) in m.windows(2).enumerate() {
            assert!(w[1].icon > w[0].icon, "{w:?}");
            assert!(w[1].button > w[0].button, "{w:?}");
            assert!(w[1].prop_icon > w[0].prop_icon, "{w:?}");
            assert!(w[1].prop_button > w[0].prop_button, "{w:?}");
            // The padding around the icon grows by less than the icon.
            let pad = |m: &ButtonMetrics| m.button - m.icon;
            let prop_pad = |m: &ButtonMetrics| m.prop_button - m.prop_icon;
            assert!(pad(&w[1]) >= pad(&w[0]), "{w:?}");
            assert!(pad(&w[1]) - pad(&w[0]) < w[1].icon - w[0].icon, "{w:?}");
            assert!(prop_pad(&w[1]) >= prop_pad(&w[0]), "{w:?}");
            if i < 2 {
                assert!(
                    prop_pad(&w[1]) - prop_pad(&w[0]) < w[1].prop_icon - w[0].prop_icon,
                    "{w:?}"
                );
            }
        }
        for m in &m {
            // Whole, even pixel sizes: the icon sits centred on whole
            // pixels in its button.
            for v in [m.icon, m.button, m.bar_icon, m.prop_icon, m.prop_button] {
                assert_eq!(v % 2.0, 0.0, "{m:?}");
            }
            assert!(m.icon < m.button && m.prop_icon < m.prop_button);
            assert!(m.bar_icon < m.button);
        }
        // The bars' icons grow at every step too.
        for w in m.windows(2) {
            assert!(w[1].bar_icon > w[0].bar_icon, "{w:?}");
        }
        // Medium keeps the 32 px buttons it had.
        assert_eq!(ButtonSize::Medium.metrics().button, 32.0);
        assert_eq!(ButtonSize::Medium.metrics().prop_button, 28.0);
    }

    /// At Large and Extra Large the standard and property bars draw their
    /// icons at about 5/6 of the toolbox's box, leaving 7 and 9 px of
    /// margin each side; the smaller sizes and the toolbox are as before.
    #[test]
    fn large_bar_icons_keep_a_margin() {
        let row = |s: ButtonSize| {
            let m = s.metrics();
            (m.icon, m.button, m.bar_icon, m.prop_icon, m.prop_button)
        };
        assert_eq!(row(ButtonSize::ExtraSmall), (16.0, 22.0, 16.0, 14.0, 20.0));
        assert_eq!(row(ButtonSize::Small), (20.0, 26.0, 20.0, 18.0, 24.0));
        assert_eq!(row(ButtonSize::Medium), (24.0, 32.0, 24.0, 22.0, 28.0));
        assert_eq!(row(ButtonSize::Large), (32.0, 42.0, 28.0, 24.0, 38.0));
        assert_eq!(row(ButtonSize::ExtraLarge), (40.0, 52.0, 34.0, 28.0, 46.0));
        for (size, margin) in [(ButtonSize::Large, 7.0), (ButtonSize::ExtraLarge, 9.0)] {
            let m = size.metrics();
            assert_eq!((m.button - m.bar_icon) / 2.0, margin, "{size:?}");
            assert_eq!((m.prop_button - m.prop_icon) / 2.0, margin, "{size:?}");
            // About 5/6 of the toolbox's icon box.
            let share = m.bar_icon / m.icon;
            assert!((0.8..=0.9).contains(&share), "{size:?} {share}");
        }
        for size in [
            ButtonSize::ExtraSmall,
            ButtonSize::Small,
            ButtonSize::Medium,
        ] {
            let m = size.metrics();
            assert_eq!(m.bar_icon, m.icon, "{size:?}");
        }
    }

    /// Settings saved with three sizes read as they did; the new sizes
    /// round-trip.
    #[test]
    fn saved_button_sizes_keep_their_meaning() {
        for (json, size) in [
            ("\"small\"", ButtonSize::Small),
            ("\"medium\"", ButtonSize::Medium),
            ("\"large\"", ButtonSize::Large),
            ("\"Medium\"", ButtonSize::Medium),
            ("\"extra_small\"", ButtonSize::ExtraSmall),
            ("\"extra_large\"", ButtonSize::ExtraLarge),
        ] {
            let read: ButtonSize = serde_json::from_str(json).expect(json);
            assert_eq!(read, size, "{json}");
        }
        for size in ButtonSize::ALL {
            let json = serde_json::to_string(&size).expect("write");
            assert_eq!(serde_json::from_str::<ButtonSize>(&json).ok(), Some(size));
        }
        let s: Settings =
            serde_json::from_str(r#"{"button_size": "medium", "toolbox_button_size": "large"}"#)
                .expect("settings");
        assert_eq!(s.button_size, ButtonSize::Medium);
        assert_eq!(s.toolbox_button_size, Some(ButtonSize::Large));
    }

    /// Ctrl+wheel steps the button sizes and the swatch size within
    /// their ranges; preferences written before the swatch size existed
    /// keep the 20 px swatches.
    #[test]
    fn the_wheel_steps_the_button_and_swatch_sizes_within_their_ranges() {
        assert_eq!(ButtonSize::Small.stepped(1), ButtonSize::Medium);
        assert_eq!(ButtonSize::Small.stepped(2), ButtonSize::Large);
        assert_eq!(ButtonSize::Small.stepped(3), ButtonSize::ExtraLarge);
        assert_eq!(ButtonSize::Small.stepped(5), ButtonSize::ExtraLarge);
        assert_eq!(ButtonSize::Large.stepped(-1), ButtonSize::Medium);
        assert_eq!(ButtonSize::Small.stepped(-1), ButtonSize::ExtraSmall);
        assert_eq!(ButtonSize::Small.stepped(-5), ButtonSize::ExtraSmall);
        assert_eq!(ButtonSize::Medium.stepped(0), ButtonSize::Medium);
        // One notch at a time walks through all five sizes and holds at
        // the ends.
        let mut size = ButtonSize::ExtraSmall;
        let mut seen = vec![size];
        for _ in 0..6 {
            size = size.stepped(1);
            seen.push(size);
        }
        assert_eq!(&seen[..5], &ButtonSize::ALL[..]);
        assert_eq!(seen[5], ButtonSize::ExtraLarge);
        assert_eq!(seen[6], ButtonSize::ExtraLarge);
        for _ in 0..6 {
            size = size.stepped(-1);
        }
        assert_eq!(size, ButtonSize::ExtraSmall);
        assert_eq!(swatch_resized(20, 1), 22);
        assert_eq!(swatch_resized(20, -1), 18);
        assert_eq!(swatch_resized(20, 100), SWATCH_MAX);
        assert_eq!(swatch_resized(20, -100), SWATCH_MIN);
        assert_eq!(swatch_resized(47, 1), SWATCH_MAX);
        assert_eq!(swatch_resized(0, 0), SWATCH_MIN);
        assert_eq!(swatch_resized(20, i32::MAX), SWATCH_MAX);
        let old: Settings = serde_json::from_str(r#"{"palette":{"rows":2}}"#).expect("old");
        assert_eq!(old.palette.swatch_size, SWATCH_DEFAULT);
        assert_eq!(old.palette.swatch_size(), 20.0);
        assert_eq!(rows_of(old.palette.rows), Some(2));
        let odd = PalettePrefs {
            swatch_size: 200,
            ..PalettePrefs::default()
        };
        assert_eq!(odd.swatch_size(), f32::from(SWATCH_MAX));
    }

    #[test]
    fn recent_list_dedups_and_caps() {
        let mut s = Settings::default();
        for i in 0..12 {
            s.touch_recent(std::path::Path::new(&format!("/f{i}.tdraw")));
        }
        s.touch_recent(std::path::Path::new("/f11.tdraw"));
        assert_eq!(s.recent_files.len(), 10);
        assert_eq!(s.recent_files[0], PathBuf::from("/f11.tdraw"));
    }

    /// A settings file of an older version keeps its language, whatever
    /// the installer or the system say; a new one has none picked.
    #[test]
    fn old_settings_files_keep_their_language() {
        for code in ["en", "de", "pt-BR"] {
            let old: Settings =
                serde_json::from_str(&format!("{{\"language\":\"{code}\"}}")).expect("old file");
            assert_eq!(
                crate::i18n::startup_language(&old.language, Some("ja"), &["fr-FR"]),
                code
            );
        }
        let fresh: Settings = serde_json::from_str("{}").expect("empty file");
        assert_eq!(fresh.language, "");
        assert_eq!(Settings::default().language, "");
        assert_eq!(
            crate::i18n::startup_language(&fresh.language, Some("ja"), &["fr-FR"]),
            "ja"
        );
        assert_eq!(
            crate::i18n::startup_language(&fresh.language, None, &["fr-FR"]),
            "fr"
        );
        // System language picked in Options is saved as such.
        let mut s = Settings::default();
        s.language = crate::i18n::SYSTEM.into();
        let back: Settings =
            serde_json::from_str(&serde_json::to_string(&s).expect("json")).expect("back");
        assert_eq!(back.language, crate::i18n::SYSTEM);
    }

    #[test]
    fn round_trip_json() {
        let s = Settings::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&j).unwrap();
        assert_eq!(back.nudge_mm, s.nudge_mm);
        let partial: Settings = serde_json::from_str("{\"language\":\"de\"}").unwrap();
        assert_eq!(partial.language, "de");
        assert!(
            partial.wheel_zooms,
            "old settings files keep the zoom default"
        );
        assert!(partial.zoom_relative_1to1 && !partial.wheel_zoom_centers);
        assert!(!partial.pan_right_click_out);
        assert!(partial.snap.guides);
    }

    #[test]
    fn a_reset_keeps_history_and_nothing_else() {
        let mut s = Settings::default();
        s.recent_files = vec!["/a.tdraw".into()];
        s.last_dir = Some("/drawings".into());
        s.window_maximized = false;
        s.window_size = [1200.0, 800.0];
        s.shortcuts = vec![("menu.file.export".into(), "Ctrl+Shift+X".into())];
        s.ui_size = 1.5;
        s.language = "de".into();
        s.shown_bars = Some(ShownBars {
            status_bar: false,
            ..ShownBars::default()
        });
        let r = s.reset_keeping_history();
        assert_eq!(r.recent_files, s.recent_files);
        assert_eq!(r.last_dir, s.last_dir);
        assert_eq!(
            (r.window_maximized, r.window_size),
            (false, [1200.0, 800.0])
        );
        assert!(r.shortcuts.is_empty() && r.shown_bars.is_none());
        assert_eq!((r.ui_size, r.language.as_str()), (0.0, ""));
    }

    #[test]
    fn a_settings_folder_next_to_the_program_makes_it_portable() {
        let base = std::env::temp_dir().join(format!("tracedraw-portable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).expect("dir");
        let exe = base.join("tracedraw.exe");
        assert_eq!(portable_dir(&exe), None);
        std::fs::create_dir_all(base.join(PORTABLE_DIR)).expect("dir");
        assert_eq!(portable_dir(&exe), Some(base.join(PORTABLE_DIR)));
        let _ = std::fs::remove_dir_all(&base);
    }

    /// The portable zip ships the settings folder (see the Package step
    /// of the build workflow), under the name the program looks for.
    #[test]
    fn the_portable_zip_ships_the_settings_folder() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let folder = root.join("packaging/windows/portable").join(PORTABLE_DIR);
        assert!(folder.join("README.txt").is_file(), "{folder:?}");
        let workflow =
            std::fs::read_to_string(root.join(".github/workflows/build.yml")).expect("workflow");
        assert!(workflow.contains(&format!("packaging\\windows\\portable\\{PORTABLE_DIR}")));
        assert!(workflow.contains(&format!("\\{PORTABLE_DIR}\",")));
    }

    /// A file from before the guides existed had both switches on with
    /// nothing behind them: off once at upgrade, and left alone after.
    #[test]
    fn inherited_guide_switches_go_off_once() {
        let old = migrated(
            r#"{"upgrades": 2, "snap": {"alignment_guides": true, "dynamic_guides": true}}"#,
        );
        assert!(!old.snap.alignment_guides && !old.snap.dynamic_guides);
        assert_eq!(old.upgrades, UPGRADES);
        // Turned on by the user since: kept.
        let kept = migrated(
            r#"{"upgrades": 3, "snap": {"alignment_guides": true, "dynamic_guides": false}}"#,
        );
        assert!(kept.snap.alignment_guides && !kept.snap.dynamic_guides);
        assert!(!Settings::default().snap.alignment_guides);
    }

    /// Preferences as an older version saved them, migrated as at start.
    /// A first start snaps to guidelines and the page but not to objects,
    /// within 10 screen pixels (#1814); a choice saved before is kept,
    /// either way, and the app takes it.
    #[test]
    fn new_installs_do_not_snap_to_objects_and_saved_choices_stay() {
        let s = Settings::default();
        assert!(!s.snap.objects, "snapping to objects starts off");
        assert!(s.snap.guides && s.snap.page && !s.snap.off);
        assert_eq!(s.snap.threshold_px, 10.0);
        let on = migrated(r#"{"upgrades": 5, "snap": {"objects": true, "page": true}}"#);
        assert!(on.snap.objects);
        let old = migrated(r#"{"snap": {"objects": true, "page": false}}"#);
        assert!(old.snap.objects && !old.snap.page);
        let mut app = crate::app::App::headless();
        app.apply_settings(Settings::default());
        assert!(!app.snap.objects && app.snap.page && app.snap.guides);
        app.apply_settings(on);
        assert!(app.snap.objects);
    }

    fn migrated(json: &str) -> Settings {
        let s: Settings = serde_json::from_str(json).expect("old preferences");
        s.upgrade()
    }

    /// A shortcut saved for the grammar checker under its old key goes
    /// by the new key; the other shortcuts, and one already under the new
    /// key, stay as they are.
    #[test]
    fn a_shortcut_saved_under_the_old_grammar_key_moves_to_the_new_one() {
        let s = migrated(
            r#"{"shortcuts": [["menu.text.grammatik", "Ctrl+Shift+G"],
                ["menu.file.open", "Ctrl+Shift+O"], ["menu.text.grammar", "F7"]]}"#,
        );
        assert_eq!(
            s.shortcuts,
            [
                ("menu.text.grammar".to_string(), "Ctrl+Shift+G".to_string()),
                ("menu.file.open".into(), "Ctrl+Shift+O".into()),
                ("menu.text.grammar".into(), "F7".into()),
            ]
        );
        // An empty shortcut (a built-in one taken away) moves too.
        let s = migrated(r#"{"shortcuts": [["menu.text.grammatik", ""]]}"#);
        assert_eq!(
            s.shortcuts,
            [("menu.text.grammar".to_string(), String::new())]
        );
        // Running the migration again changes nothing.
        let mut list = s.shortcuts.clone();
        migrate_command_keys(&mut list);
        assert_eq!(list, s.shortcuts);
        // Every renamed key names a command that exists now, not one that
        // was itself renamed.
        for (old, now) in RENAMED_COMMAND_KEYS {
            assert_ne!(old, now);
            assert!(!RENAMED_COMMAND_KEYS.iter().any(|(o, _)| *o == now));
            assert!(
                crate::i18n::english_keys().iter().any(|k| k == now),
                "{now}"
            );
            assert!(
                !crate::i18n::english_keys().iter().any(|k| k == old),
                "{old}"
            );
        }
    }

    #[test]
    fn an_older_image_service_alone_becomes_the_one_service() {
        use crate::ai::{ImageSupport, Provider};
        use crate::ai_image::ImageProvider;
        // Gemini: the Gemini preset with that key.
        let s = migrated(
            r#"{"ai_image": {"provider": "gemini", "model": "gemini-3-pro-image-preview",
                "key": "g-key", "remember_key": true}}"#,
        );
        assert!(s.ai_image.is_none(), "folded in");
        assert_eq!(s.ai.provider, Provider::OpenAiCompatible);
        assert_eq!(crate::ai::preset_of(&s.ai), Some(2));
        assert_eq!(s.ai.image_support(), ImageSupport::Gemini);
        assert_eq!(crate::ai::remembered_key(&s.ai), "g-key");
        let p = s.ai.image_prefs().expect("pictures");
        assert_eq!(
            (p.provider, p.model()),
            (ImageProvider::Gemini, "gemini-3-pro-image-preview".into())
        );
        // OpenAI at its usual address: the OpenAI preset.
        let s = migrated(
            r#"{"ai_image": {"provider": "open_ai", "key": "o-key", "remember_key": true}}"#,
        );
        assert_eq!(crate::ai::preset_of(&s.ai), Some(1));
        assert_eq!(crate::ai::remembered_key(&s.ai), "o-key");
        assert_eq!(
            s.ai.image_prefs().map(|p| p.model()),
            Some("gpt-image-1".into())
        );
        // OpenAI-compatible at the user's address: a custom address with
        // the image model kept.
        let s = migrated(
            r#"{"ai_image": {"provider": "open_ai", "endpoint": "http://box:7000/v1/",
                "model": "sd-xl"}}"#,
        );
        assert_eq!(s.ai.endpoint, "http://box:7000/v1/chat/completions");
        assert_eq!(crate::ai::preset_of(&s.ai), None);
        let p = s.ai.image_prefs().expect("pictures");
        assert_eq!(
            (p.endpoint(), p.model()),
            ("http://box:7000/v1".into(), "sd-xl".into())
        );
        assert!(!s.ai.remember_key && s.ai.key.is_empty());
        // The usual local image server shares its address with a chat-only
        // preset: it stays a custom address that makes pictures.
        let s = migrated(
            r#"{"ai_image": {"provider": "open_ai", "endpoint": "http://localhost:8080/v1"}}"#,
        );
        assert_eq!(s.ai.endpoint, "http://127.0.0.1:8080/v1/chat/completions");
        assert!(s.ai.can_make_pictures());
        assert_eq!(
            s.ai.image_prefs().map(|p| p.model()),
            Some("gpt-image-1".into())
        );
        // The prompt bar switch is kept.
        let s = migrated(r#"{"ai": {"show_bar": false}, "ai_image": {"provider": "open_ai"}}"#);
        assert!(!s.ai.show_bar);
        // Saved again, only the one service is written; read back it stays.
        let json = serde_json::to_string(&s).expect("saved");
        assert!(!json.contains("\"ai_image\""));
        let again = migrated(&json);
        assert_eq!(again.ai, s.ai);
    }

    #[test]
    fn an_older_drawing_service_stays_and_no_remembered_key_is_lost() {
        use crate::ai::Provider;
        // Both set up, of the same kind: the drawing service stays and
        // takes the image model; its own key wins.
        let s = migrated(
            r#"{"ai": {"provider": "open_ai_compatible", "model": "gpt-5", "key": "d-key",
                "remember_key": true},
                "ai_image": {"provider": "open_ai", "model": "dall-e-3", "key": "d-key",
                "remember_key": true}}"#,
        );
        assert_eq!(
            (s.ai.model.as_str(), s.ai.image_model.as_str()),
            ("gpt-5", "dall-e-3")
        );
        assert_eq!(crate::ai::remembered_key(&s.ai), "d-key");
        assert!(s.ai_image.is_none());
        // The drawing service without a remembered key takes the image
        // service's key of the same kind.
        let s = migrated(
            r#"{"ai": {"provider": "open_ai_compatible",
                "endpoint": "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions",
                "model": "gemini-2.5-flash"},
                "ai_image": {"provider": "gemini", "key": "g-key", "remember_key": true}}"#,
        );
        assert_eq!(crate::ai::remembered_key(&s.ai), "g-key");
        assert_eq!(s.ai.model, "gemini-2.5-flash");
        assert!(s.ai_image.is_none());
        // Different kinds (the Messages API and Gemini pictures): the
        // drawing service stays, and the image key stays in the file.
        let s = migrated(
            r#"{"ai": {"key": "a-key", "remember_key": true},
                "ai_image": {"provider": "gemini", "key": "g-key", "remember_key": true}}"#,
        );
        assert_eq!(s.ai.provider, Provider::Anthropic);
        assert_eq!(crate::ai::remembered_key(&s.ai), "a-key");
        assert!(!s.ai.can_make_pictures());
        let json = serde_json::to_string(&s).expect("saved");
        assert!(json.contains("a-key") && json.contains("g-key"));
        let again = migrated(&json);
        assert_eq!(again.ai, s.ai);
        assert!(again.ai_image.is_some(), "still kept");
        // Two different remembered keys of the same kind: both kept.
        let s = migrated(
            r#"{"ai": {"provider": "open_ai_compatible", "key": "one", "remember_key": true},
                "ai_image": {"provider": "open_ai", "key": "two", "remember_key": true}}"#,
        );
        assert_eq!(crate::ai::remembered_key(&s.ai), "one");
        assert_eq!(s.ai_image.as_ref().map(|i| i.key.as_str()), Some("two"));
        // An image service left as it came: nothing to fold in.
        let s = migrated(r#"{"ai_image": {}}"#);
        assert_eq!(s.ai, crate::ai::AiPrefs::default());
        assert!(s.ai_image.is_none());
    }
}
