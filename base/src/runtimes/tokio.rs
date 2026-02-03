use crate::runtimes::MAX_THREAD_NAME_LENGTH;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::runtime::{Builder, Runtime};
use tokio::task::JoinHandle;

static APP_RT: std::sync::LazyLock<Runtime> =
	std::sync::LazyLock::new(|| build_named_runtime("app-rt", Some(num_cpus::get() * 2)));

pub trait Spawnable: Future + Send + 'static {
	fn spawn(self) -> JoinHandle<Self::Output>;
}

impl<F> Spawnable for F
where
	F: Future + Send + 'static,
	F::Output: Send + 'static,
{
	fn spawn(self) -> JoinHandle<Self::Output> {
		APP_RT.spawn(self)
	}
}

pub trait SpawnTask {
	fn spawn_task(self);
}

impl<T> SpawnTask for T
where
	T: Spawnable<Output = ()> + Send + 'static,
{
	fn spawn_task(self) {
		self.spawn();
	}
}

pub struct Tokio;

impl Tokio {
	pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>
	where
		F: Future + Send + 'static,
		F::Output: Send + 'static,
	{
		APP_RT.spawn(future)
	}

	// pub fn spawn_sys_thread<F>(&self, future: F) -> std::thread::JoinHandle<()>
	// where
	// 	F: Future<Output = ()> + Send + 'static,
	// {
	// 	spawn_sys_thread(future)
	// }
}

/// Returns a tokio runtime with named threads.
/// This is useful for tracking threads when debugging.
pub fn build_named_runtime(thread_name: &str, num_worker_threads: Option<usize>) -> Runtime {
	build_named_runtime_with_start_hook(thread_name, num_worker_threads, || {})
}

pub fn build_named_runtime_with_start_hook<F>(
	thread_name: &str,
	num_worker_threads: Option<usize>,
	on_thread_start: F,
) -> Runtime
where
	F: Fn() + Send + Sync + 'static,
{
	const MAX_BLOCKING_THREADS: usize = 64;

	// Verify the given name has an appropriate length
	if thread_name.len() > MAX_THREAD_NAME_LENGTH {
		panic!(
			"The given runtime thread name is too long! Max length: {}, given name: {}",
			MAX_THREAD_NAME_LENGTH, thread_name
		);
	}

	// Create the runtime builder
	let atomic_id = AtomicUsize::new(0);
	let thread_name_clone = thread_name.to_string();
	let mut builder = Builder::new_multi_thread();
	builder
		.thread_name_fn(move || {
			let id = atomic_id.fetch_add(1, Ordering::SeqCst);
			format!("{thread_name_clone}-{id}")
		})
		.on_thread_start(on_thread_start)
		// .disable_lifo_slot()
		// Limit concurrent blocking tasks from spawn_blocking(), in case, for example, too many
		// Rest API calls overwhelm the node.
		.max_blocking_threads(MAX_BLOCKING_THREADS)
		.enable_all();
	if let Some(num_worker_threads) = num_worker_threads {
		builder.worker_threads(num_worker_threads);
	}

	// Spawn and return the runtime
	builder.build().unwrap_or_else(|error| {
		panic!("Failed to spawn named runtime! Name: {thread_name:?}, Error: {error:?}",)
	})
}

// fn spawn_sys_thread<F>(fut: F) -> std::thread::JoinHandle<()>
// where
// 	F: Future<Output = ()> + Send + 'static,
// {
// 	std::thread::spawn(|| {
// 		Builder::new_current_thread()
// 			.enable_all()
// 			.build()
// 			.expect("New runtime spawn to main thread error")
// 			.block_on(fut);
// 	})
// }

#[cfg(test)]
mod tests {
	use super::*;
	use crate::runtimes::Tokio;

	#[tokio::test]
	async fn it_works() {
		Tokio.spawn(async {
			println!("Hello, world!");
		});
	}

	#[tokio::test]
	async fn test_spawn_task() {
		let task = async {
			println!("Hello, task!");
		};
		task.spawn_task();
	}
}
