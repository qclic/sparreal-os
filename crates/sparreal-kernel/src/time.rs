use core::{future::Future, time::Duration};

use crate::platform::PlatformImpl;

pub fn since_boot() -> Duration {
    let current_tick = unsafe { PlatformImpl::current_ticks() };
    let freq = unsafe { PlatformImpl::tick_hz() };
    Duration::from_nanos(current_tick * 1_000_000_000 / freq)
}

pub fn delay(duration: Duration) -> impl Future<Output = ()> {
    unsafe {
        let current_tick = PlatformImpl::current_ticks();
        let freq = PlatformImpl::tick_hz();
        let ticks = duration.as_nanos() * freq as u128 / 1_000_000_000;
        let until = current_tick + ticks as u64;
        FutureDelay { until }
    }
}

pub struct FutureDelay {
    until: u64,
}

impl Future for FutureDelay {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let current_tick = unsafe { PlatformImpl::current_ticks() };
        if current_tick >= self.until {
            core::task::Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    }
}
