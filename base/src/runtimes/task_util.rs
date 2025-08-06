use crate::runtimes::Tokio;
use std::sync::{Arc, Mutex};
use std::task::Waker;
use std::time::{Duration, Instant};
use tokio::time;
use tracing::Instrument;

pub struct WakerRunner {
	waker: Arc<Mutex<Option<Waker>>>,
	timeout: Instant,
	deadline: Duration,
}

impl WakerRunner {
	pub fn new(waker: Arc<Mutex<Option<Waker>>>, timeout: Instant, deadline: Duration) -> Self {
		Self {
			waker,
			timeout,
			deadline,
		}
	}

	pub fn start(self) {
		let fut = async move {
			let delay = Instant::now() + self.deadline;
			time::sleep_until(time::Instant::from_std(delay)).await;

			let mut waker = self
				.waker
				.lock()
				.expect("Cannot currently handle a poisoned lock");
			if let Some(waker) = waker.take() {
				waker.wake()
			}
		};

		if self.timeout > Instant::now() {
			Tokio.spawn(fut.in_current_span());
		}
	}
}
