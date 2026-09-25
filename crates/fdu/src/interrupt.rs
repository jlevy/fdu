//! Ctrl-C while the indicator draws.
//!
//! A handler is installed only for a run that draws, so every other run keeps the
//! default signal behavior exactly. When Ctrl-C arrives the handler takes the line,
//! marks it stopped so no later frame is drawn, erases the frame if one is showing,
//! writes `fdu: interrupted`, and then ends the process the way the default action
//! would. After the ticker has stopped, while the report is written, it skips the
//! message and takes the same default action.
//!
//! What the default action is differs by platform, and so does the crate. On Unix it
//! is death by `SIGINT`: `signal-hook`'s `Signals` hands the signal to a thread of our
//! own, which cleans the line and calls `emulate_default_handler`, restoring the
//! default disposition and raising the signal again, so the shell reports 130 and a
//! calling script stops, which an exit status of 130 alone would not make it do. On
//! Windows it is `ExitProcess(STATUS_CONTROL_C_EXIT)`, what the console does to a
//! process with no handler of its own: `ctrlc`'s console handler runs the closure on
//! a thread of its own, and the closure cleans the line and exits with that status.
//! Both crates register through safe functions, so this module adds no `unsafe`.

use std::io::Write;
use std::sync::Arc;

use crate::cli::{STYLE_ERROR, paint};
use crate::progress_ticker::Line;

/// What a person sees under the erased line when they interrupt a drawing run.
pub(crate) const INTERRUPTED: &str = "fdu: interrupted";

/// What the handler does when Ctrl-C arrives: stop, erase, say so, then `terminate`.
///
/// The line's lock is held for the first three, so the ticker cannot draw a frame
/// between the erase and the message, and released before `terminate`, which in
/// production never returns. The message is skipped when the line was already
/// stopped: the ticker had finished, and what the terminal shows is the report being
/// written, not a frame. Write errors are ignored, as everywhere on this line; this
/// is the last thing the process does.
pub(crate) fn on_interrupt(line: &Line, color: bool, terminate: impl FnOnce()) {
    {
        let mut state = line.lock();
        let was_drawing = !state.stopped;
        state.stop();
        if was_drawing {
            let _ = writeln!(state.out, "{}", paint(INTERRUPTED, STYLE_ERROR, color))
                .and_then(|()| state.out.flush());
        }
    }
    terminate();
}

/// Install the handler over `line`, for a run that draws.
///
/// A registration that fails leaves today's behavior, the process dying on Ctrl-C with
/// the line uncleared, rather than refusing to run: the indicator is a courtesy and the
/// report is the job.
#[cfg(unix)]
pub(crate) fn install(line: Arc<Line>, color: bool) {
    use signal_hook::consts::SIGINT;
    use signal_hook::iterator::Signals;
    use signal_hook::low_level::emulate_default_handler;

    let Ok(mut signals) = Signals::new([SIGINT]) else {
        return;
    };
    // Never joined: it blocks until the first SIGINT, and the process ends inside the
    // handler, so there is no second one and nothing to wait for on the way out.
    let _ = std::thread::Builder::new().name("fdu-interrupt".to_string()).spawn(move || {
        if signals.forever().next().is_some() {
            on_interrupt(&line, color, || {
                let _ = emulate_default_handler(SIGINT);
            });
        }
    });
}

/// Install the handler over `line`, for a run that draws.
///
/// A registration that fails leaves today's behavior, the process dying on Ctrl-C with
/// the line uncleared, rather than refusing to run: the indicator is a courtesy and the
/// report is the job.
#[cfg(windows)]
pub(crate) fn install(line: Arc<Line>, color: bool) {
    /// The status the console's own default Ctrl-C handling ends a process with.
    const STATUS_CONTROL_C_EXIT: i32 = i32::from_ne_bytes(0xC000_013A_u32.to_ne_bytes());

    let _ = ctrlc::set_handler(move || {
        on_interrupt(&line, color, || std::process::exit(STATUS_CONTROL_C_EXIT));
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    use crate::progress_ticker::{ERASE_LINE, SharedBuffer};

    #[test]
    fn an_interrupt_stops_erases_says_so_and_only_then_terminates() {
        let out = SharedBuffer::default();
        let line = Line::new(Box::new(out.clone()));
        line.lock().draw("⠋ ~  Scanning");
        assert!(line.lock().frame_on_screen);

        let at_terminate = Mutex::new(None);
        on_interrupt(&line, false, || {
            let state = line.lock();
            *at_terminate.lock().expect("no poison") =
                Some((state.stopped, state.frame_on_screen, out.text()));
        });
        let (stopped, frame_on_screen, text) =
            at_terminate.into_inner().expect("no poison").expect("terminate was called");
        assert!(stopped, "the line is stopped before the process ends");
        assert!(!frame_on_screen);
        assert_eq!(text, format!("{ERASE_LINE}⠋ ~  Scanning{ERASE_LINE}{INTERRUPTED}\n"));
        assert_eq!(out.text(), text, "nothing is written after terminate");
    }

    #[test]
    fn the_message_takes_the_error_style_under_the_color_rule() {
        let out = SharedBuffer::default();
        let line = Line::new(Box::new(out.clone()));
        on_interrupt(&line, true, || {});
        assert_eq!(out.text(), "\u{1b}[1m\u{1b}[31mfdu: interrupted\u{1b}[0m\n");
        assert!(!out.text().contains(ERASE_LINE), "no frame was showing, so nothing to erase");
    }

    #[test]
    fn after_the_ticker_has_stopped_an_interrupt_is_silent_but_still_terminates() {
        let out = SharedBuffer::default();
        let line = Line::new(Box::new(out.clone()));
        line.lock().draw("⠋ ~  Scanning");
        line.lock().stop();
        let before = out.text();
        assert!(before.ends_with(ERASE_LINE));

        let mut terminated = false;
        on_interrupt(&line, false, || terminated = true);
        assert!(terminated);
        assert_eq!(out.text(), before, "no message once the report is being written");

        // And a second interrupt, after the first marked the line stopped, is silent too.
        on_interrupt(&line, false, || {});
        assert_eq!(out.text(), before);
    }

    #[test]
    fn no_frame_is_drawn_after_an_interrupt_stops_the_line() {
        use crate::progress_line::ProgressPlan;
        use crate::progress_ticker::{ProgressIo, Ticker, Timing, scanned};

        let root = tempfile::tempdir().expect("tempdir");
        let out = SharedBuffer::default();
        let io = ProgressIo {
            out: Box::new(out.clone()),
            timing: Timing { delay: Duration::ZERO, tick: Duration::from_millis(1) },
            width: || 100,
            interrupt: |_, _| {},
        };
        let plan = ProgressPlan {
            draw: true,
            root: "~".to_string(),
            color: false,
            size: fdu_core::query::SizeMetric::Allocated,
        };
        let mut ticker = Ticker::start(plan, scanned(root.path()), Instant::now(), io);
        let deadline = Instant::now() + Duration::from_secs(10);
        while out.contents().is_empty() {
            assert!(Instant::now() < deadline, "the ticker never drew");
            std::thread::sleep(Duration::from_millis(1));
        }

        on_interrupt(ticker.line(), false, || {});
        let after_interrupt = out.text();
        assert!(after_interrupt.ends_with(&format!("{ERASE_LINE}{INTERRUPTED}\n")));
        std::thread::sleep(Duration::from_millis(30));
        assert_eq!(out.text(), after_interrupt, "a frame was drawn after the interrupt");

        // The stop point the run reaches next finds the line already stopped and clean.
        ticker.stop();
        assert_eq!(out.text(), after_interrupt);
    }
}
