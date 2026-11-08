//! A quit asked of the whole program by the system rather than by the
//! window (#1669): on macOS the Quit item of the application menu and its
//! Command+Q, the Dock icon's Quit, a log-out or shut-down. AppKit would
//! carry it out at once, without the window hearing of it, and unsaved
//! drawings were lost. The patched winit asks the program first
//! (`winit::platform::macos::set_quit_handler`, `vendor/README.md`); the
//! answer comes from here, on the main thread and without waiting for a
//! frame: with unsaved drawings the quit is taken over (the terminate is
//! cancelled) and the next frame asks about each one, as File > Exit and
//! the close button do (`App::request_exit`); with none it goes ahead.
//! See `docs/behavior/recovery.md`.

use crate::app::App;
use std::sync::atomic::{AtomicBool, Ordering};

/// What the system's question needs between frames.
pub struct SystemQuit {
    /// A drawing had unsaved changes at the last frame.
    unsaved: AtomicBool,
    /// The system asked to quit and the program took it over.
    asked: AtomicBool,
}

impl SystemQuit {
    pub const fn new() -> Self {
        Self {
            unsaved: AtomicBool::new(false),
            asked: AtomicBool::new(false),
        }
    }

    /// The system asks to quit: true when the program takes it over (a
    /// drawing has unsaved changes; the window is woken to ask about
    /// them), false when the quit may go ahead.
    pub fn system_asks(&self, wake: Option<&egui::Context>) -> bool {
        if !self.unsaved.load(Ordering::SeqCst) {
            return false;
        }
        self.asked.store(true, Ordering::SeqCst);
        if let Some(ctx) = wake {
            ctx.request_repaint();
        }
        true
    }

    /// Each frame: a quit the system asked for goes the way of File >
    /// Exit, and the unsaved state is kept for its next question.
    pub fn frame(&self, app: &mut App) {
        if self.asked.swap(false, Ordering::SeqCst) && !app.quit_now {
            app.request_exit();
        }
        self.unsaved.store(app.has_unsaved(), Ordering::SeqCst);
    }
}

/// The program's one instance, read by the handler the window sets.
#[cfg(target_os = "macos")]
pub static SYSTEM_QUIT: SystemQuit = SystemQuit::new();

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::dialogs::Dialog;
    use tracedraw_core::document::ShapeKind;
    use tracedraw_core::geometry::{Rect, Size};
    use tracedraw_core::Document;

    fn dirty_app() -> App {
        let mut app = App::headless();
        app.open_document(Document::new("Dirty", Size::new(50.0, 50.0)), None);
        app.new_shape(ShapeKind::Rect {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            radius: 0.0,
            corners: None,
        })
        .expect("a layer");
        app
    }

    #[test]
    fn a_quit_with_no_unsaved_drawing_goes_ahead() {
        let q = SystemQuit::new();
        let mut app = App::headless();
        q.frame(&mut app);
        assert!(!app.has_unsaved());
        assert!(!q.system_asks(None));
        q.frame(&mut app);
        assert!(!app.quit_now);
        assert!(matches!(app.dialog, Dialog::None));
    }

    #[test]
    fn a_quit_with_an_unsaved_drawing_is_taken_over_and_asks_like_file_exit() {
        let q = SystemQuit::new();
        let mut app = dirty_app();
        assert!(app.has_unsaved());
        // Before the first frame nothing is known: the quit goes ahead
        // (the backup on leaving keeps the drawing, #1395).
        assert!(!q.system_asks(None));
        q.frame(&mut app);
        assert!(q.system_asks(None));
        assert!(!app.quit_now);
        // The next frame asks Save / Don't Save / Cancel.
        q.frame(&mut app);
        assert!(matches!(app.dialog, Dialog::ConfirmClose));
        assert!(!app.quit_now);
        // Cancel: the program stays, and the request was used up.
        app.dialog = Dialog::None;
        app.after_close_question(false);
        q.frame(&mut app);
        assert!(matches!(app.dialog, Dialog::None));
        assert!(!app.quit_now && app.has_document());
        // Asked again, Don't Save quits.
        assert!(q.system_asks(None));
        q.frame(&mut app);
        assert!(matches!(app.dialog, Dialog::ConfirmClose));
        app.dialog = Dialog::None;
        app.close_active_document();
        app.after_close_question(true);
        assert!(app.quit_now);
    }

    #[test]
    fn a_drawing_saved_before_the_frame_quits_without_asking() {
        let q = SystemQuit::new();
        let mut app = dirty_app();
        q.frame(&mut app);
        assert!(q.system_asks(None));
        app.engine.mark_saved();
        q.frame(&mut app);
        assert!(app.quit_now);
        assert!(matches!(app.dialog, Dialog::None));
    }
}
