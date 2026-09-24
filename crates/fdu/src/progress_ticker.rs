//! The thread that draws the progress line, and the one owner of that line.
//!
//! `progress_line` decides whether a run may draw and what a frame says; this module
//! draws it. A [`Ticker`] polls the engine's [`Progress`] handle on its own thread and
//! redraws one line on stderr until it is stopped, and a [`Line`] is the state that
//! says whether a frame is on screen. Exactly one owner holds the line at a time: the
//! ticker takes its lock to draw, the stop point takes it to erase, and the interrupt
//! handler takes it to erase and say why, so nothing is written over a frame and no
//! frame is drawn over anything else.
//!
//! The ticker waits for its first frame with a timed receive on its stop channel rather
//! than a sleep, so a run that finishes inside the delay stops it at once, shows no
//! indicator, and pays nothing on exit; after that it redraws on every tick. A frame is
//! `\r\x1b[2K` plus what `render_frame` says; the cursor is never hidden. Write errors
//! on the line are ignored and never change the exit status, and after the first failed
//! write the ticker stops drawing.

use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use fdu_core::{Progress, ProgressPhase, ProgressSnapshot};

use crate::progress_line::{FrameFacts, Phase, ProgressPlan, render_frame};

/// Move to the start of the line and erase it.
///
/// Every frame starts with it, and it is the last thing the ticker writes. Erasing the
/// whole line rather than a frame's byte count is what leaves no residue when a
/// terminal was narrower than the frame or was resized under it.
pub(crate) const ERASE_LINE: &str = "\r\x1b[2K";

/// How long a run may take before the first frame appears.
const FIRST_FRAME_DELAY: Duration = Duration::from_millis(500);

/// How often the line is redrawn once it is showing, one spinner cell per redraw.
const REDRAW_INTERVAL: Duration = Duration::from_millis(80);

/// The width a frame must fit when stderr's own width is unknown.
const FALLBACK_WIDTH: usize = 80;

/// When the first frame appears and how often the line is redrawn after it.
///
/// The shipped values are the plan's 500 ms and 80 ms; a test passes its own so it can
/// see a frame at once, or none at all.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Timing {
    /// How long a run may take before the first frame appears.
    pub delay: Duration,
    /// How often the line is redrawn once it is showing.
    pub tick: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self { delay: FIRST_FRAME_DELAY, tick: REDRAW_INTERVAL }
    }
}

/// The process resources the indicator draws with, passed in so a test can substitute
/// its own.
///
/// A run reads these once, in `run_process`, the way it reads its terminal facts: the
/// shipped values are the process's stderr and the plan's timings, and a test passes a
/// buffer it can read back with a delay it can wait out.
pub(crate) struct ProgressIo {
    /// Where frames go, and where the interruption message goes.
    ///
    /// The process's stderr, or a test's buffer that also receives the run's
    /// diagnostics, so the test sees the order the terminal would.
    pub out: Box<dyn Write + Send>,
    /// When the first frame appears and how often the line is redrawn.
    pub timing: Timing,
    /// The width a frame must fit, read again for every frame so a resize takes effect
    /// at once.
    pub width: fn() -> usize,
    /// Installs the Ctrl-C handler over the line once the ticker owns it, with whether
    /// the interruption message is colored.
    ///
    /// The process's real handler ends the process, so a test passes one that does
    /// nothing and drives the handler's logic directly.
    pub interrupt: fn(Arc<Line>, bool),
}

impl ProgressIo {
    /// The process's own stderr, the shipped timings, and the real Ctrl-C handler.
    pub(crate) fn for_process() -> Self {
        Self {
            out: Box::new(io::stderr()),
            timing: Timing::default(),
            width: stderr_width,
            interrupt: crate::interrupt::install,
        }
    }

    /// Resources for a test that is not about the indicator: a line nobody reads, with
    /// a delay no test outlives, so a run under injected interactive facts behaves as
    /// one that finished inside the delay, and no handler.
    #[cfg(test)]
    pub(crate) fn inert() -> Self {
        let never = Duration::from_secs(60 * 60);
        Self {
            out: Box::new(io::sink()),
            timing: Timing { delay: never, tick: never },
            width: || 80,
            interrupt: |_, _| {},
        }
    }
}

/// Stderr's own terminal width, or [`FALLBACK_WIDTH`] when it has none.
///
/// Stderr's, not stdout's: the line is drawn on stderr, and a run whose stdout is a
/// pipe still has a terminal to fit.
pub(crate) fn stderr_width() -> usize {
    terminal_size::terminal_size_of(io::stderr())
        .map_or(FALLBACK_WIDTH, |(terminal_size::Width(width), _)| usize::from(width))
}

/// What is on the line right now, and where it is drawn.
pub(crate) struct LineState {
    /// No further frame may be drawn.
    ///
    /// Set by the stop point, by the interrupt handler, and by a failed write. Once set
    /// it is never cleared: a line is for one run.
    pub stopped: bool,
    /// A frame is on screen, so the line must be erased before anything else reaches
    /// the terminal.
    pub frame_on_screen: bool,
    /// Where the line is drawn.
    pub out: Box<dyn Write + Send>,
}

impl LineState {
    /// Draw `frame` over whatever the line shows.
    ///
    /// A write that fails stops the ticker: the line is a courtesy, and a stderr that
    /// refuses it is not one to keep writing to. Whether a frame is on screen is left
    /// as it was, because a frame drawn earlier may still be showing and one attempt
    /// to erase it at the stop point costs nothing.
    pub(crate) fn draw(&mut self, frame: &str) {
        let written = self
            .out
            .write_all(ERASE_LINE.as_bytes())
            .and_then(|()| self.out.write_all(frame.as_bytes()))
            .and_then(|()| self.out.flush());
        match written {
            Ok(()) => self.frame_on_screen = true,
            Err(_) => self.stopped = true,
        }
    }

    /// Stop drawing, and erase the frame if one is showing.
    ///
    /// Idempotent, so the stop point, the guard's drop, and the interrupt handler can
    /// each call it without knowing who went first.
    pub(crate) fn stop(&mut self) {
        self.stopped = true;
        if self.frame_on_screen {
            self.frame_on_screen = false;
            let _ = self.out.write_all(ERASE_LINE.as_bytes()).and_then(|()| self.out.flush());
        }
    }
}

/// The progress line, shared by the ticker thread, the stop point, and the interrupt
/// handler.
pub(crate) struct Line {
    state: Mutex<LineState>,
}

impl Line {
    /// A line with nothing on it, drawn to `out`.
    pub(crate) fn new(out: Box<dyn Write + Send>) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(LineState { stopped: false, frame_on_screen: false, out }),
        })
    }

    /// Take the line.
    ///
    /// A poisoned lock is taken anyway: the thread that panicked while holding it was
    /// drawing, and the state it left is exactly what the stop point and the interrupt
    /// handler need in order to clean up after it.
    pub(crate) fn lock(&self) -> MutexGuard<'_, LineState> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The engine's snapshot as the renderer takes it, or `None` while no route has begun.
///
/// `Starting` draws nothing: the engine has not named a phase yet, so the ticker keeps
/// waiting rather than show one it invented. The other phases map one to one, and the
/// counters pass through unchanged.
pub(crate) fn frame_facts(snapshot: &ProgressSnapshot) -> Option<FrameFacts> {
    let phase = match snapshot.phase {
        ProgressPhase::Starting => return None,
        ProgressPhase::Loading => Phase::Loading,
        ProgressPhase::Scanning => Phase::Scanning,
        ProgressPhase::Revalidating => Phase::Revalidating,
        ProgressPhase::Analyzing => Phase::Analyzing,
        ProgressPhase::Saving => Phase::Saving,
        ProgressPhase::Indexing => Phase::Indexing,
    };
    Some(FrameFacts {
        phase,
        directories: snapshot.directories,
        files: snapshot.files,
        bytes: snapshot.bytes,
        analysis: snapshot.analysis,
    })
}

/// The thread that redraws the line, and the guard that stops it.
///
/// Started only for a run that draws. [`Ticker::stop`] stops the thread, joins it, and
/// erases the line; dropping the ticker does the same, which is what clears the line
/// on unwind. The caller stops it explicitly at one place, right after the engine
/// returns and before anything is written to either stream, so the guard is the
/// fallback rather than the rule.
pub(crate) struct Ticker {
    line: Arc<Line>,
    /// Dropped to stop: the thread's timed receive returns at once when the sender is
    /// gone, so stopping never waits out a delay or a tick.
    stop: Option<mpsc::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Ticker {
    /// Start drawing `progress` for `plan`, with elapsed time measured from `started`.
    ///
    /// The line is owned from here: nothing else may write to `io.out` until
    /// [`Ticker::stop`] returns. A thread that cannot be spawned draws nothing, and the
    /// run proceeds as if it were not drawing. The Ctrl-C handler is installed here,
    /// over the same line, and only here: a run that does not draw never reaches this.
    pub(crate) fn start(
        plan: ProgressPlan,
        progress: Progress,
        started: Instant,
        io: ProgressIo,
    ) -> Self {
        let line = Line::new(io.out);
        let color = plan.color;
        let (stop, stopped) = mpsc::channel();
        let thread = thread::Builder::new()
            .name("fdu-progress".to_string())
            .spawn({
                let line = Arc::clone(&line);
                move || {
                    redraw_until_stopped(
                        &line, &plan, &progress, started, &stopped, io.timing, io.width,
                    );
                }
            })
            .ok();
        (io.interrupt)(Arc::clone(&line), color);
        Self { line, stop: Some(stop), thread }
    }

    /// The line this ticker draws, for a test to inspect or interrupt.
    #[cfg(test)]
    pub(crate) fn line(&self) -> &Arc<Line> {
        &self.line
    }

    /// Stop the thread, join it, and erase the line if a frame is showing.
    ///
    /// Returns at once when the thread is still waiting for its first frame. Idempotent:
    /// a second call, including the one from drop, finds nothing to stop and nothing on
    /// the line.
    pub(crate) fn stop(&mut self) {
        drop(self.stop.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        self.line.lock().stop();
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The ticker thread: wait out the delay, then redraw on every tick until stopped.
fn redraw_until_stopped(
    line: &Line,
    plan: &ProgressPlan,
    progress: &Progress,
    started: Instant,
    stopped: &Receiver<()>,
    timing: Timing,
    width: fn() -> usize,
) {
    let mut wait = timing.delay;
    let mut spinner_step = 0;
    loop {
        match stopped.recv_timeout(wait) {
            Err(RecvTimeoutError::Timeout) => {}
            Ok(()) | Err(RecvTimeoutError::Disconnected) => return,
        }
        wait = timing.tick;
        let Some(facts) = frame_facts(&progress.snapshot()) else {
            continue;
        };
        let frame =
            render_frame(&plan.root, &facts, started.elapsed(), spinner_step, width(), plan.color);
        let mut state = line.lock();
        if state.stopped {
            return;
        }
        state.draw(&frame);
        if state.stopped {
            return;
        }
        spinner_step = spinner_step.wrapping_add(1);
    }
}

/// A buffer several writers share, for tests that need the ticker's bytes and a run's
/// diagnostics in one stream, in the order a terminal would see them.
#[cfg(test)]
#[derive(Clone, Default)]
pub(crate) struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

#[cfg(test)]
impl SharedBuffer {
    pub(crate) fn contents(&self) -> Vec<u8> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    pub(crate) fn text(&self) -> String {
        String::from_utf8(self.contents()).expect("the buffer holds UTF-8")
    }
}

#[cfg(test)]
impl Write for SharedBuffer {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner).extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A handle a route has finished with, for tests: its final phase persists after the
/// route returns, so a ticker started over it draws at once.
///
/// The engine keeps phase changes to itself, so this is the one way a test in this
/// crate gets a handle past `Starting`: run a real report over `root` with the cache
/// off, which leaves the handle in `Indexing` with the tree's counts.
#[cfg(test)]
pub(crate) fn scanned(root: &std::path::Path) -> Progress {
    use std::time::SystemTime;

    use fdu_core::content::AnalysisSet;
    use fdu_core::query::{Basis, Delivery, Query, Request};
    use fdu_core::{CachePolicy, ScanConfig, prepare_report_with_progress};

    let progress = Progress::new();
    let request = Request::new(
        Basis {
            root: root.to_path_buf(),
            scope: ScanConfig::default().into(),
            content: AnalysisSet::NONE,
        },
        Query::default(),
        SystemTime::now(),
    );
    let delivery = Delivery::new(CachePolicy::Off, None);
    let (_, pending, _) =
        prepare_report_with_progress(&request, &delivery, &progress).expect("a report");
    pending.join().expect("no save to fail");
    assert_eq!(progress.snapshot().phase, ProgressPhase::Indexing);
    progress
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOT: &str = "~/wrk/github";

    fn plan() -> ProgressPlan {
        ProgressPlan { draw: true, root: ROOT.to_string(), color: false }
    }

    fn io(out: &SharedBuffer, timing: Timing) -> ProgressIo {
        ProgressIo { out: Box::new(out.clone()), timing, width: || 100, interrupt: |_, _| {} }
    }

    /// How many erase sequences `bytes` holds: one per frame drawn, plus the final one.
    fn erases(bytes: &[u8]) -> usize {
        bytes.windows(ERASE_LINE.len()).filter(|window| *window == ERASE_LINE.as_bytes()).count()
    }

    fn wait_for(out: &SharedBuffer, condition: impl Fn(&[u8]) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !condition(&out.contents()) {
            assert!(Instant::now() < deadline, "the ticker never got there");
            thread::sleep(Duration::from_millis(1));
        }
    }

    #[test]
    fn a_fresh_handle_gives_no_frame_and_every_phase_maps_across() {
        assert_eq!(frame_facts(&Progress::new().snapshot()), None);
        let snapshot = ProgressSnapshot {
            phase: ProgressPhase::Analyzing,
            directories: 3,
            files: 40,
            bytes: 4_096,
            analysis: Some((2, 5)),
        };
        assert_eq!(
            frame_facts(&snapshot),
            Some(FrameFacts {
                phase: Phase::Analyzing,
                directories: 3,
                files: 40,
                bytes: 4_096,
                analysis: Some((2, 5)),
            })
        );
        for (engine, frame) in [
            (ProgressPhase::Loading, Phase::Loading),
            (ProgressPhase::Scanning, Phase::Scanning),
            (ProgressPhase::Revalidating, Phase::Revalidating),
            (ProgressPhase::Analyzing, Phase::Analyzing),
            (ProgressPhase::Saving, Phase::Saving),
            (ProgressPhase::Indexing, Phase::Indexing),
        ] {
            let snapshot = ProgressSnapshot { phase: engine, ..snapshot };
            assert_eq!(frame_facts(&snapshot).map(|facts| facts.phase), Some(frame));
        }
    }

    #[test]
    fn a_run_that_finishes_inside_the_delay_writes_nothing_and_stops_at_once() {
        let root = tempfile::tempdir().expect("tempdir");
        let out = SharedBuffer::default();
        let timing = Timing { delay: Duration::from_secs(60), tick: Duration::from_secs(60) };
        let started = Instant::now();
        let mut ticker = Ticker::start(plan(), scanned(root.path()), started, io(&out, timing));
        ticker.stop();
        assert!(started.elapsed() < Duration::from_secs(10), "stopping waited out the delay");
        assert!(out.contents().is_empty(), "{:?}", out.text());
        assert!(ticker.line().lock().stopped);
        assert!(!ticker.line().lock().frame_on_screen);
    }

    #[test]
    fn the_first_bytes_are_an_erase_and_a_frame_and_the_last_are_an_erase() {
        let root = tempfile::tempdir().expect("tempdir");
        let out = SharedBuffer::default();
        let timing = Timing { delay: Duration::ZERO, tick: Duration::from_millis(1) };
        let mut ticker =
            Ticker::start(plan(), scanned(root.path()), Instant::now(), io(&out, timing));
        wait_for(&out, |bytes| erases(bytes) >= 3);
        ticker.stop();
        assert!(ticker.line().lock().stopped);
        assert!(!ticker.line().lock().frame_on_screen);

        let text = out.text();
        let frames: Vec<&str> = text.split(ERASE_LINE).collect();
        assert_eq!(frames[0], "", "the first bytes are the erase sequence");
        assert!(
            frames[1].starts_with(&format!("⠋ {ROOT}  Indexing      0 files · 1 dirs · 0 B  ")),
            "{:?}",
            frames[1]
        );
        assert!(frames[2].starts_with('⠙'), "the spinner advances one cell per redraw");
        assert!(frames[3].starts_with('⠹'));
        assert_eq!(frames[frames.len() - 1], "", "the last bytes are the erase sequence");
        assert!(text.ends_with(ERASE_LINE));
        assert!(!text.contains("\x1b[?25l"), "the cursor is never hidden");

        // Nothing else, ever: a second stop finds nothing to do.
        let before = out.contents();
        ticker.stop();
        drop(ticker);
        assert_eq!(out.contents(), before);
    }

    #[test]
    fn a_stopped_ticker_can_be_stopped_again_and_is_stopped_by_drop() {
        let root = tempfile::tempdir().expect("tempdir");
        let out = SharedBuffer::default();
        let timing = Timing { delay: Duration::ZERO, tick: Duration::from_millis(1) };
        let ticker = Ticker::start(plan(), scanned(root.path()), Instant::now(), io(&out, timing));
        wait_for(&out, |bytes| !bytes.is_empty());
        let line = Arc::clone(ticker.line());
        drop(ticker);
        assert!(line.lock().stopped);
        assert!(!line.lock().frame_on_screen);
        assert!(out.text().ends_with(ERASE_LINE));
        let before = out.contents();
        thread::sleep(Duration::from_millis(20));
        assert_eq!(out.contents(), before, "a dropped ticker draws nothing more");
    }

    /// A writer that accepts one frame and refuses everything after it.
    struct FailsAfterOneFrame {
        accepted: SharedBuffer,
        writes: usize,
    }

    impl Write for FailsAfterOneFrame {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            // A frame is two writes: the erase sequence and the frame.
            if self.writes >= 2 {
                return Err(io::Error::other("stderr went away"));
            }
            self.writes += 1;
            self.accepted.write(buffer)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn after_a_failed_write_nothing_more_is_written() {
        let root = tempfile::tempdir().expect("tempdir");
        let accepted = SharedBuffer::default();
        let out = FailsAfterOneFrame { accepted: accepted.clone(), writes: 0 };
        let timing = Timing { delay: Duration::ZERO, tick: Duration::from_millis(1) };
        let io = ProgressIo { out: Box::new(out), timing, width: || 100, interrupt: |_, _| {} };
        let mut ticker = Ticker::start(plan(), scanned(root.path()), Instant::now(), io);
        wait_for(&accepted, |bytes| !bytes.is_empty());
        // The second frame fails and stops the ticker on its own.
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ticker.line().lock().stopped {
            assert!(Instant::now() < deadline, "the failed write never stopped the ticker");
            thread::sleep(Duration::from_millis(1));
        }
        let first_frame = accepted.text();
        assert!(
            first_frame.starts_with(&format!("{ERASE_LINE}⠋ {ROOT}  Indexing")),
            "{first_frame:?}"
        );
        assert_eq!(first_frame.matches(ERASE_LINE).count(), 1, "exactly one frame got through");

        ticker.stop();
        assert_eq!(accepted.text(), first_frame, "the erase at the stop point was refused too");
        assert!(!ticker.line().lock().frame_on_screen);
    }

    #[test]
    fn the_shipped_timing_is_the_plan_s_and_the_width_falls_back_to_eighty() {
        let timing = Timing::default();
        assert_eq!(timing.delay, Duration::from_millis(500));
        assert_eq!(timing.tick, Duration::from_millis(80));
        let width = stderr_width();
        assert!(width >= 1, "a width of {width} fits nothing");
        if !io::IsTerminal::is_terminal(&io::stderr()) {
            assert_eq!(width, FALLBACK_WIDTH);
        }
    }

    #[test]
    fn a_new_line_shows_nothing_and_stop_without_a_frame_writes_nothing() {
        let out = SharedBuffer::default();
        let line = Line::new(Box::new(out.clone()));
        {
            let mut state = line.lock();
            assert!(!state.stopped);
            assert!(!state.frame_on_screen);
            state.stop();
            assert!(state.stopped);
        }
        assert!(out.contents().is_empty());

        let line = Line::new(Box::new(out.clone()));
        line.lock().draw("⠋ a frame");
        assert!(line.lock().frame_on_screen);
        line.lock().stop();
        assert_eq!(out.text(), format!("{ERASE_LINE}⠋ a frame{ERASE_LINE}"));
        line.lock().stop();
        assert_eq!(out.text(), format!("{ERASE_LINE}⠋ a frame{ERASE_LINE}"), "idempotent");
    }
}
