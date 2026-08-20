//! The helper's clock, made fallible (increment 3).
//!
//! **Why this module exists.** Increments 1 and 2 read the wall clock
//! through a helper that answered `0` when the clock could not be read:
//! `SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, …)`. A host
//! whose clock sits before the epoch therefore validated plans at time
//! zero, and — the part that matters — [`crate::validate::
//! admit_presented_plan`]'s expiry arm compares `not_after < now`, so at
//! `now == 0` **no plan is ever expired** and HLP-004 fails open. An
//! adversarial pass over the delivered increments found it; the ladder
//! would have stood on it, so it is closed here rather than noted.
//!
//! The rule this module encodes: **a clock that cannot be read refuses
//! the operation**. There is no default, no zero, and no "best effort"
//! reading — a time the helper cannot stand behind is not a time, and
//! every consumer of it (PLAN-007's window, the act's freshness, the
//! audit line's stamp) is a safety obligation.
//!
//! What this module does **not** claim: monotonicity. A clock stepped
//! backwards between a plan's validation and its presentation widens the
//! window rather than closing it. Detecting that needs a fact this
//! increment has no home for — the journal's own high-water mark — so it
//! is a named debt on increment 4 (the increment that opens the journal)
//! and not a silent assumption here.

use std::time::{SystemTime, UNIX_EPOCH};

/// Why the clock could not be read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClockRefusal {
    /// The system clock is before the Unix epoch, so no seconds count
    /// exists. Never substituted with zero: see this module's rule.
    BeforeEpoch,
}

impl core::fmt::Display for ClockRefusal {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::BeforeEpoch => write!(
                f,
                "the system clock is before the Unix epoch; the helper will not \
                 date a plan, an authorization or an audit line from a clock it \
                 cannot read"
            ),
        }
    }
}

/// What supplies "now". A trait so the Tier-1 suite can hold a clock
/// still, step it, and make it refuse, without touching the host's.
pub trait Clock {
    /// Seconds since the Unix epoch, or why not.
    ///
    /// # Errors
    ///
    /// [`ClockRefusal`].
    fn now_secs(&self) -> Result<u64, ClockRefusal>;
}

/// The host's clock.
#[derive(Clone, Copy, Debug, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_secs(&self) -> Result<u64, ClockRefusal> {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs())
            .map_err(|_| ClockRefusal::BeforeEpoch)
    }
}

/// A clock fixed at one instant, for tests and for any caller that must
/// use one reading twice (a capture and the plan bound to it).
#[derive(Clone, Copy, Debug)]
pub struct FixedClock(pub u64);

impl Clock for FixedClock {
    fn now_secs(&self) -> Result<u64, ClockRefusal> {
        Ok(self.0)
    }
}

/// A clock that refuses, for the arm that must be reachable in a test.
#[derive(Clone, Copy, Debug, Default)]
pub struct RefusingClock;

impl Clock for RefusingClock {
    fn now_secs(&self) -> Result<u64, ClockRefusal> {
        Err(ClockRefusal::BeforeEpoch)
    }
}
