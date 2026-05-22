//! [`SpiralismoPerceiver`] — [`ExternalPerceiver`] adapter over [`SpiralismoPress`].

use std::sync::{Arc, Mutex};

use super::spiralismo_press::{SpiralismoPress, SPIRALISMO_PERCEIVER_ID};
use super::traits::{ExternalPerceiver, PerceptionFrame, PerceptionOffer};

/// Shared hand-state for hosts that register an explicit trait-object perceptor.
#[derive(Clone)]
pub struct SpiralismoPerceiver {
    press: Arc<Mutex<SpiralismoPress>>,
}

impl SpiralismoPerceiver {
    /// New perceptor holding the given press snapshot (updates via [`Self::replace_press`]).
    #[must_use]
    pub fn new(press: SpiralismoPress) -> Self {
        Self {
            press: Arc::new(Mutex::new(press)),
        }
    }

    /// Replaces the entire press (mouse frame, click burst, etc.).
    pub fn replace_press(&self, press: SpiralismoPress) {
        if let Ok(mut g) = self.press.lock() {
            *g = press;
        }
    }

    /// Read-only copy for diagnostics.
    #[must_use]
    pub fn read_press(&self) -> SpiralismoPress {
        self.press
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }
}

impl ExternalPerceiver for SpiralismoPerceiver {
    fn id(&self) -> &'static str {
        SPIRALISMO_PERCEIVER_ID
    }

    fn perceive(&self, _frame: &PerceptionFrame) -> PerceptionOffer {
        self.press
            .lock()
            .map(|g| g.to_offer())
            .unwrap_or_else(|_| PerceptionOffer::silent())
    }
}
