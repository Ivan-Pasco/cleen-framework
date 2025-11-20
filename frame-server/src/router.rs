use axum::{Router, routing::get};

pub struct FrameRouter;

impl FrameRouter {
	pub fn new() -> Router {
		Router::new()
			.route("/", get(|| async { "Frame Server" }))
			.route("/health", get(|| async { "OK" }))
	}
}

impl Default for FrameRouter {
	fn default() -> Self {
		Self
	}
}
