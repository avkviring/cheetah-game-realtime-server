use std::convert::Infallible;
use std::net::SocketAddr;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use prometheus::{Encoder, TextEncoder};

pub mod counter;
pub mod gauge;

pub(crate) fn setup_prometheus() {
	tokio::spawn(async move {
		tracing::info!("starting prometheus exporter");
		let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
		let service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(metrics)) });
		let server = Server::bind(&addr).serve(service);
		if let Err(e) = server.await {
			tracing::error!("hyper server error: {}", e);
		}
	});
}

async fn metrics(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
	let metric_families = prometheus::gather();
	let encoder = TextEncoder::new();
	let mut buffer = Vec::new();
	encoder.encode(&metric_families, &mut buffer).unwrap();
	let output = String::from_utf8(buffer.clone()).unwrap();
	Ok(Response::new(output.into()))
}

#[cfg(test)]
mod test {}
