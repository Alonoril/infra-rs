use std::boxed::Box;
use std::fmt::{Debug, Display};
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tracing::{debug, error, warn};

const MAX_RETRY: usize = 10;

struct Delay {
    end_time: Instant,
}

impl Delay {
    fn new(delay_ms: u32) -> Self {
        Self {
            end_time: Instant::now() + Duration::from_millis(delay_ms as u64),
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if Instant::now() >= self.end_time {
            Poll::Ready(())
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

pub struct Retry<F, Out, Err, Fut> {
    operation: F,
    max_retries: usize,
    attempt_times: usize,
    delay_fut: Option<Pin<Box<Delay>>>,
    fn_future: Option<Pin<Box<Fut>>>,
    _mark: PhantomData<(Out, Err)>,
}

impl<F, Out, Err, Fut> Unpin for Retry<F, Out, Err, Fut> {}

impl<F, Out, Err, Fut> Retry<F, Out, Err, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Out, Err>>,
{
    /// Default max_retries is 3
    pub fn run(max_retries: Option<usize>, fn_fut: F) -> Self {
        Self {
            operation: fn_fut,
            max_retries: max_retries.unwrap_or(3),
            attempt_times: 0,
            delay_fut: None,
            fn_future: None,
            _mark: PhantomData,
        }
    }

    fn delay_ms(&self, attempt_times: usize) -> u32 {
        let delay = if attempt_times > MAX_RETRY {
            1 << (attempt_times % MAX_RETRY)
        } else {
            1 << attempt_times
        };
        delay * 500
    }
}

impl<F, Out, Err, Fut> Future for Retry<F, Out, Err, Fut>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Out, Err>>,
    Err: Debug + Display,
{
    type Output = Result<Out, Err>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.as_mut().get_mut();
        if let Some(delay) = this.delay_fut.as_mut() {
            match delay.as_mut().poll(cx) {
                Poll::Pending => {
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(()) => {
                    this.delay_fut = None;
                }
            }
        }

        if this.fn_future.is_none() {
            this.fn_future = Some(Box::pin((this.operation)()));
        }

        if let Some(mut op_fut) = this.fn_future.take() {
            return match op_fut.as_mut().poll(cx) {
                Poll::Ready(Ok(result)) => Poll::Ready(Ok(result)),
                Poll::Ready(Err(e)) if this.attempt_times < this.max_retries => {
                    debug!("Retrying... {e}");

                    this.attempt_times += 1;
                    let delay_ms = this.delay_ms(this.attempt_times) ;
                    warn!("Retry next times[{}], after {} ms", this.attempt_times, delay_ms);
                    this.delay_fut = Some(Box::pin(Delay::new(delay_ms)));

                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Poll::Ready(Err(e)) => {
                    error!("Failed after {} attempts: {}", this.attempt_times, e);
                    Poll::Ready(Err(e))
                },
                Poll::Pending => {
                    this.fn_future = Some(op_fut);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            };
        }

        Poll::Pending
    }
}
