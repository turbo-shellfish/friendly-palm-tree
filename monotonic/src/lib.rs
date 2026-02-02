use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::sync::Arc;
use std::time::Duration;

#[cfg(debug_assertions)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum ClockSource {
    Std,
    Mock(u64),
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant {
    inner: std::time::Instant,
    #[cfg(debug_assertions)]
    source: ClockSource,
}

impl Instant {
    #[inline]
    pub fn duration_since(&self, earlier: Instant) -> Duration {
        #[cfg(debug_assertions)]
        assert!(self.source == earlier.source);

        self.inner.duration_since(earlier.inner)
    }

    #[inline]
    pub fn checked_duration_since(&self, earlier: Instant) -> Option<Duration> {
        #[cfg(debug_assertions)]
        assert!(self.source == earlier.source);

        self.inner.checked_duration_since(earlier.inner)
    }

    #[inline]
    pub fn saturating_duration_since(&self, earlier: Instant) -> Duration {
        #[cfg(debug_assertions)]
        assert!(self.source == earlier.source);

        self.inner.saturating_duration_since(earlier.inner)
    }

    #[inline]
    pub fn checked_add(&self, duration: Duration) -> Option<Instant> {
        self.inner.checked_add(duration).map(|inner| Self {
            inner,
            #[cfg(debug_assertions)]
            source: self.source,
        })
    }

    #[inline]
    pub fn checked_sub(&self, duration: Duration) -> Option<Instant> {
        self.inner.checked_sub(duration).map(|inner| Self {
            inner,
            #[cfg(debug_assertions)]
            source: self.source,
        })
    }
}

impl Add<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn add(self, rhs: Duration) -> Self::Output {
        Self {
            inner: self.inner + rhs,
            #[cfg(debug_assertions)]
            source: self.source,
        }
    }
}

impl AddAssign<Duration> for Instant {
    #[inline]
    fn add_assign(&mut self, rhs: Duration) {
        *self = *self + rhs;
    }
}

impl Sub<Duration> for Instant {
    type Output = Instant;

    #[inline]
    fn sub(self, rhs: Duration) -> Self::Output {
        Self {
            inner: self.inner - rhs,
            #[cfg(debug_assertions)]
            source: self.source,
        }
    }
}

impl Sub for Instant {
    type Output = Duration;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        #[cfg(debug_assertions)]
        assert!(self.source == rhs.source);

        self.inner - rhs.inner
    }
}

impl SubAssign<Duration> for Instant {
    #[inline]
    fn sub_assign(&mut self, rhs: Duration) {
        *self = *self - rhs;
    }
}

pub trait Clock {
    fn now(&self) -> Instant;
}

mod private {
    pub trait Sealed {}
    impl<C> Sealed for C where C: super::Clock {}
}

pub trait ClockExt: Clock + private::Sealed {
    fn elapsed_since(&self, instant: Instant) -> Duration;
}

impl<C> ClockExt for C
where
    C: Clock,
{
    #[inline]
    fn elapsed_since(&self, instant: Instant) -> Duration {
        #[cfg(not(debug_assertions))]
        {
            self.now() - instant
        }

        #[cfg(debug_assertions)]
        {
            let now = self.now();
            debug_assert!(now.source == instant.source);
            now - instant
        }
    }
}

impl<T> Clock for Arc<T>
where
    T: Clock,
{
    #[inline]
    fn now(&self) -> Instant {
        (**self).now()
    }
}

#[cfg(debug_assertions)]
static NEXT_MOCK_CLOCK_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub struct StdClock;

#[cfg(debug_assertions)]
impl StdClock {
    #[inline]
    pub fn new_mock_epoch() -> Instant {
        let clock_id = NEXT_MOCK_CLOCK_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Instant {
            inner: std::time::Instant::now(),
            source: ClockSource::Mock(clock_id),
        }
    }
}

impl Clock for StdClock {
    #[inline]
    fn now(&self) -> Instant {
        Instant {
            inner: std::time::Instant::now(),
            #[cfg(debug_assertions)]
            source: ClockSource::Std,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Clock, StdClock};
    use std::time::Duration;

    #[test]
    fn test() {
        let clock = StdClock;

        let start = clock.now();
        let actual_start = std::time::Instant::now();

        assert!(actual_start - start.inner < Duration::from_millis(1));
    }
}
