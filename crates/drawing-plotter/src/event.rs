//! Events and handles for background plotting

use crate::PlotterError;
use drawing_core::Point;
use std::sync::mpsc;
use std::thread::JoinHandle;

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
    /// Plotting completed successfully
    Completed,
    /// Error occurred
    Error(String),
}

/// Handle to a background plotting job
pub struct PlotHandle {
    /// Channel to receive plot events
    pub receiver: mpsc::Receiver<PlotEvent>,
    /// Thread handle
    handle: JoinHandle<Result<(), PlotterError>>,
}

impl PlotHandle {
    /// Create a new PlotHandle
    pub(crate) fn new(
        receiver: mpsc::Receiver<PlotEvent>,
        handle: JoinHandle<Result<(), PlotterError>>,
    ) -> Self {
        Self { receiver, handle }
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
