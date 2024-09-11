use crate::Platform;
use core::{future::Future, marker::PhantomData, time::Duration};

pub struct Time<P: Platform> {
    _marker: PhantomData<P>,
}

impl<P: Platform> Time<P> {
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn delay(duration: Duration) -> impl Future<Output = ()> {
        let current_tick = P::current_ticks();
        let freq = P::tick_hz();
        let ticks = duration.as_nanos() * freq as u128 / 1_000_000_000;
        let until = current_tick + ticks as u64;
        FutureDelay {
            until,
            _marker: PhantomData::<P>,
        }
    }

    pub fn since_boot() -> Duration {
        let current_tick = P::current_ticks();
        let freq = P::tick_hz();
        Duration::from_nanos(current_tick * 1_000_000_000 / freq)
    }
}

pub struct FutureDelay<P: Platform> {
    until: u64,
    _marker: PhantomData<P>,
}

impl<P: Platform> Future for FutureDelay<P> {
    type Output = ();

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        let current_tick = P::current_ticks();
        if current_tick >= self.until {
            core::task::Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            core::task::Poll::Pending
        }
    }
}
