use std::{sync::LazyLock, time::Instant};

/// The monotonic time instant when the program started. Actually, it keeps the time when this constant is first accessed.
static EPOCH: LazyLock<Instant> = LazyLock::new(|| Instant::now());

/// Returns the current monotonic time (never goes backwards) in milliseconds since the program started.
///
/// It is used to calculate the expiration time of cache entries. Since it is monotonic, it is not affected
/// by system time changes.
#[inline]
pub fn current_monotime() -> u64 {
    EPOCH.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}
