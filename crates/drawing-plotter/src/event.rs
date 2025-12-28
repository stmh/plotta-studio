//! Events and handles for background plotting

use crate::PlotterError;
use drawing_core::Point;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

/// Events emitted during plotting
#[derive(Debug, Clone)]
pub enum PlotEvent {
    /// Plotting started
    Started { total_strokes: usize },
    /// Starting a new stroke
    StrokeStart { index: usize, total: usize },
    /// Stroke completed
    StrokeComplete { index: usize, total: usize },
    /// Moving to a position
    MoveTo { position: Point, pen_down: bool },
    /// Plotting paused
    Paused,
    /// Plotting resumed
    Resumed,
    /// Plotting completed successfully
    Completed,
    /// Error occurred
    Error(String),
}

/// Shared state for pause control
#[derive(Clone)]
pub struct PauseControl {
    /// Flag indicating if plotting should be paused
    paused: Arc<AtomicBool>,
}

impl PauseControl {
    /// Create a new pause control
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Check if currently paused
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    /// Request pause
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    /// Request resume
    pub fn resume(&self) {
        self.paused.store(false, Ordering::SeqCst);
    }

    /// Toggle pause state, returns new state (true = paused)
    pub fn toggle(&self) -> bool {
        let was_paused = self.paused.load(Ordering::SeqCst);
        self.paused.store(!was_paused, Ordering::SeqCst);
        !was_paused
    }

    /// Wait while paused, checking periodically
    /// Returns true if was paused (and now resumed), false if wasn't paused
    pub fn wait_if_paused(&self) -> bool {
        if !self.is_paused() {
            return false;
        }

        // Wait until resumed
        while self.is_paused() {
            std::thread::sleep(Duration::from_millis(50));
        }
        true
    }
}

impl Default for PauseControl {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle to a background plotting job
pub struct PlotHandle {
    /// Channel to receive plot events
    pub receiver: mpsc::Receiver<PlotEvent>,
    /// Thread handle
    handle: JoinHandle<Result<(), PlotterError>>,
    /// Pause control
    pause_control: PauseControl,
}

impl PlotHandle {
    /// Create a new PlotHandle
    pub(crate) fn new(
        receiver: mpsc::Receiver<PlotEvent>,
        handle: JoinHandle<Result<(), PlotterError>>,
        pause_control: PauseControl,
    ) -> Self {
        Self {
            receiver,
            handle,
            pause_control,
        }
    }

    /// Wait for the plotting to complete
    pub fn join(self) -> Result<(), PlotterError> {
        self.handle
            .join()
            .map_err(|_| PlotterError::Device("Plotting thread panicked".to_string()))?
    }

    /// Check if plotting is still running
    pub fn is_running(&self) -> bool {
        !self.handle.is_finished()
    }

    /// Check if plotting is paused
    pub fn is_paused(&self) -> bool {
        self.pause_control.is_paused()
    }

    /// Pause plotting (will pause after current stroke completes)
    pub fn pause(&self) {
        self.pause_control.pause();
    }

    /// Resume plotting
    pub fn resume(&self) {
        self.pause_control.resume();
    }

    /// Toggle pause state, returns new state (true = paused)
    pub fn toggle_pause(&self) -> bool {
        self.pause_control.toggle()
    }

    /// Try to receive the next event without blocking
    pub fn try_recv(&self) -> Option<PlotEvent> {
        self.receiver.try_recv().ok()
    }

    /// Receive all pending events
    pub fn drain_events(&self) -> Vec<PlotEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.receiver.try_recv() {
            events.push(event);
        }
        events
    }
}
