mod error;
mod trace;

pub use error::*;
pub use trace::*;

use base_infra::utils::uuid::UID;
use http::Request;
use tracing::{Span, info, info_span};

pub fn make_span<B>(_request: &Request<B>) -> Span {
	// let headers = request.headers();
	let trace_id = UID.v4_simple_str();
	info_span!("api", tid = trace_id.to_string())
}

pub fn accept_trace<B>(request: Request<B>) -> Request<B> {
	// Current context, if no or invalid data is received.
	// let parent_context = global::get_text_map_propagator(|propagator| {
	//     propagator.extract(&HeaderExtractor(request.headers()))
	// });
	// Span::current().set_parent(parent_context);

	request
}

pub fn record_trace_id<B>(request: Request<B>) -> Request<B> {
	// let span = Span::current();
	let uri = request.uri();

	// let trace_id = span.context().span().span_context().trace_id();
	// let trace_id = UID.v4_simple_str();
	info!(?uri);
	// span.record("tid", trace_id.to_string());

	request
}
