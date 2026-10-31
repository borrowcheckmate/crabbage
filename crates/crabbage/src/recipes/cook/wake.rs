//! The screen wake lock of the cook page: held while the page is open, dropped by the browser when the
//! page is hidden, and asked for again when it is shown. Pure and tested here; the browser binding
//! is in `wake_client.rs`.

pub(crate) trait WakeApi {
    fn supported(&self) -> bool;
    fn request(&mut self);
    fn release(&mut self);
}

#[derive(Debug)]
pub(crate) struct WakeLock<A> {
    api: A,
    wanted: bool,
    held: bool,
}

impl<A: WakeApi> WakeLock<A> {
    pub(crate) const fn new(api: A) -> Self {
        Self {
            api,
            wanted: false,
            held: false,
        }
    }

    pub(crate) fn enter(&mut self) {
        self.wanted = true;
        self.acquire();
    }

    pub(crate) fn leave(&mut self) {
        self.wanted = false;
        if self.held {
            self.api.release();
            self.held = false;
        }
    }

    pub(crate) fn visibility_changed(&mut self, visible: bool) {
        if visible {
            self.acquire();
        } else {
            self.lock_lost();
        }
    }

    pub(crate) const fn lock_lost(&mut self) {
        self.held = false;
    }

    fn acquire(&mut self) {
        if self.wanted && !self.held && self.api.supported() {
            self.api.request();
            self.held = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{WakeApi, WakeLock};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Call {
        Request,
        Release,
    }

    #[derive(Debug)]
    struct Stub {
        supported: bool,
        calls: Vec<Call>,
    }

    impl WakeApi for Stub {
        fn supported(&self) -> bool {
            self.supported
        }

        fn request(&mut self) {
            self.calls.push(Call::Request);
        }

        fn release(&mut self) {
            self.calls.push(Call::Release);
        }
    }

    fn lock() -> WakeLock<Stub> {
        WakeLock::new(Stub {
            supported: true,
            calls: Vec::new(),
        })
    }

    fn calls(lock: &WakeLock<Stub>) -> &[Call] {
        &lock.api.calls
    }

    #[test]
    fn entering_requests_the_lock() {
        let mut lock = lock();
        lock.enter();
        assert_eq!(calls(&lock), [Call::Request]);
    }

    #[test]
    fn leaving_releases_a_held_lock() {
        let mut lock = lock();
        lock.enter();
        lock.leave();
        assert_eq!(calls(&lock), [Call::Request, Call::Release]);
    }

    #[test]
    fn leaving_without_a_lock_does_nothing() {
        let mut lock = lock();
        lock.leave();
        assert_eq!(calls(&lock), []);
    }

    #[test]
    fn entering_twice_requests_once() {
        let mut lock = lock();
        lock.enter();
        lock.enter();
        assert_eq!(calls(&lock), [Call::Request]);
    }

    #[test]
    fn showing_the_page_while_the_lock_is_held_does_nothing() {
        let mut lock = lock();
        lock.enter();
        lock.visibility_changed(true);
        assert_eq!(calls(&lock), [Call::Request]);
    }

    #[test]
    fn hiding_the_page_drops_the_lock_without_a_release_call() {
        let mut lock = lock();
        lock.enter();
        lock.visibility_changed(false);
        assert_eq!(calls(&lock), [Call::Request]);
    }

    #[test]
    fn showing_the_page_again_reacquires_the_lock() {
        let mut lock = lock();
        lock.enter();
        lock.visibility_changed(false);
        lock.visibility_changed(true);
        assert_eq!(calls(&lock), [Call::Request, Call::Request]);
    }

    #[test]
    fn showing_the_page_after_leaving_does_nothing() {
        let mut lock = lock();
        lock.enter();
        lock.leave();
        lock.visibility_changed(false);
        lock.visibility_changed(true);
        assert_eq!(calls(&lock), [Call::Request, Call::Release]);
    }

    #[test]
    fn a_lock_lost_while_visible_is_not_requested_again_until_the_page_is_shown() {
        let mut lock = lock();
        lock.enter();
        lock.lock_lost();
        assert_eq!(calls(&lock), [Call::Request]);
        lock.visibility_changed(true);
        assert_eq!(calls(&lock), [Call::Request, Call::Request]);
    }

    #[test]
    fn an_unsupported_browser_makes_no_calls() {
        let mut lock = WakeLock::new(Stub {
            supported: false,
            calls: Vec::new(),
        });
        lock.enter();
        lock.visibility_changed(false);
        lock.visibility_changed(true);
        lock.leave();
        assert_eq!(calls(&lock), []);
    }
}
