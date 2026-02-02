use monotonic::{Clock, Instant, StdClock};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug)]
pub struct MockClock {
    local_epoch: Instant,
    elapsed: Mutex<Duration>,
}

impl Default for MockClock {
    #[inline]
    fn default() -> Self {
        #[cfg(debug_assertions)]
        let local_epoch = StdClock::new_mock_epoch();

        #[cfg(not(debug_assertions))]
        let local_epoch = StdClock.now();

        Self {
            local_epoch,
            elapsed: Mutex::new(Duration::ZERO),
        }
    }
}

impl MockClock {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn new_shared() -> Arc<MockClock> {
        Arc::new(MockClock::new())
    }

    #[inline]
    pub fn advance(&self, duration: Duration) {
        *self.elapsed.lock().unwrap() += duration;
    }
}

impl Clock for MockClock {
    #[inline]
    fn now(&self) -> Instant {
        self.local_epoch + *self.elapsed.lock().unwrap()
    }
}

#[allow(unused)]
#[cfg(test)]
mod tests {
    use monotonic::ClockExt;

    use super::*;

    fn use_ref<C>(clock: &C)
    where
        C: Clock,
    {
        let _ = clock.now();
        println!("Hi");
    }

    struct UsesClock<C = StdClock>
    where
        C: Clock,
    {
        start: Instant,
        clock: C,
    }

    impl Default for UsesClock<StdClock> {
        fn default() -> Self {
            Self::with_clock(StdClock)
        }
    }

    impl UsesClock<StdClock> {
        pub fn new() -> Self {
            Self::default()
        }
    }

    impl Default for UsesClock<Arc<MockClock>> {
        fn default() -> Self {
            Self::with_clock(MockClock::new_shared())
        }
    }

    impl UsesClock<Arc<MockClock>> {
        pub fn with_mock_clock(clock: Arc<MockClock>) -> Self {
            Self::with_clock(clock)
        }
    }

    impl<C> UsesClock<C>
    where
        C: Clock,
    {
        pub fn with_clock(clock: C) -> Self {
            Self {
                start: clock.now(),
                clock,
            }
        }

        pub fn use_clock(&self) -> Duration {
            self.clock.elapsed_since(self.start)
        }
    }

    #[test]
    fn test() {
        let real = StdClock;
        real.now();

        use_ref(&real);

        let mock = MockClock::new();
        mock.now();

        use_ref(&mock);

        let mut arc_mock = MockClock::new_shared(); //Arc::new(mock);
        arc_mock.now();

        use_ref(&arc_mock);

        let arc_mock_clone = Arc::clone(&arc_mock);

        let (tx_ready, rx_ready) = std::sync::mpsc::sync_channel(1);
        let (tx_update, rx_update) = std::sync::mpsc::sync_channel(1);

        let t1 = std::thread::spawn(move || {
            let true_start = std::time::Instant::now();
            let start = arc_mock_clone.now();
            loop {
                println!("True elapsed: {:?}", true_start.elapsed());
                println!("Elapsed: {:?}", arc_mock_clone.elapsed_since(start));
                tx_ready.send(()).unwrap();
                let _ = rx_update.recv().unwrap();
            }
        });

        let t2 = std::thread::spawn(move || {
            let mut count = 0;
            loop {
                let _ = rx_ready.recv().unwrap();
                count = match count {
                    0..3 => count + 1,
                    3 => {
                        arc_mock.advance(Duration::from_secs(1));
                        0
                    }
                    _ => panic!("!"),
                };
                tx_update.send(()).unwrap();
            }
        });

        let clk = MockClock::new_shared();

        let uses = UsesClock::with_mock_clock(Arc::clone(&clk));

        loop {
            println!("{:?}", uses.use_clock());
            clk.advance(Duration::from_secs(1));
        }

        t1.join().unwrap();
        t2.join().unwrap();
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn panic_with_debug_assertions() {
        let c1 = MockClock::new();

        let c2 = MockClock::new();

        c2.elapsed_since(c1.now());
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn no_panic_without_debug_assertions() {
        let c1 = MockClock::new();

        let c2 = MockClock::new();

        c2.elapsed_since(c1.now());
    }
}
