use crate::runtimes::MAX_THREAD_NAME_LENGTH;
use rayon::{ThreadPool, ThreadPoolBuilder};

/// Returns a rayon threadpool with threads.
/// This is useful for tracking threads when debugging.
pub fn build_rayon_thread_pool(thread_name: &str, num_worker_threads: Option<usize>) -> ThreadPool {
	build_rayon_thread_pool_with_start_hook(thread_name, num_worker_threads, || {})
}

pub fn build_rayon_thread_pool_with_start_hook<F>(
	thread_name: &str,
	num_worker_threads: Option<usize>,
	on_thread_start: F,
) -> ThreadPool
where
	F: Fn() + Send + Sync + 'static,
{
	// Verify the given name has an appropriate length
	if thread_name.len() > MAX_THREAD_NAME_LENGTH {
		panic!(
			"The given runtime thread name is too long! Max length: {}, given name: {}",
			MAX_THREAD_NAME_LENGTH, thread_name
		);
	}

	let thread_name_clone = thread_name.to_string();
	let mut builder = ThreadPoolBuilder::new()
		.thread_name(move |index| format!("{}-{index}", thread_name_clone))
		.start_handler(move |_| on_thread_start());

	if let Some(num_worker_threads) = num_worker_threads {
		builder = builder.num_threads(num_worker_threads);
	}

	// Spawn and return the threadpool
	builder.build().unwrap_or_else(|error| {
		panic!(
			"Failed to spawn named rayon thread pool! Name: {:?}, Error: {:?}",
			thread_name, error
		)
	})
}
