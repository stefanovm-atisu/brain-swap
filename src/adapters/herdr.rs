//! The herdr adapter (TECHSPEC 9): herdr runs through its public CLI only.

use std::time::Duration;

/// The timeout every herdr call asks for (T-15).
pub const CALL_TIMEOUT: Duration = Duration::from_secs(2);
