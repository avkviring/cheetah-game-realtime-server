use crate::loki::Loki;
use crate::proto;
use crate::proto::{EventRequest, EventResponse, LogRequest, LogResponse};
use std::time::Duration;
use tonic::{Code, Request, Response, Status};

pub struct EventReceiverService {
	loki: Loki,
}

impl EventReceiverService {
	pub fn new(loki_url: &str) -> Self {
		Self {
			loki: Loki::new(loki_url),
		}
	}
}

#[tonic::async_trait]
impl proto::event_receiver_server::EventReceiver for EventReceiverService {
	async fn send_event(&self, request: Request<EventRequest>) -> Result<Response<EventResponse>, Status> {
		let request = request.into_inner();
		let mut labels = request.labels.clone();
		labels.insert("type".to_owned(), "event".to_owned());
		labels.insert("create_time".to_owned(), request.time.to_string());
		self.loki
			.send_to_loki(request.labels, Duration::from_millis(request.time), request.value.as_str())
			.await
			.map(|_| Response::new(EventResponse {}))
			.map_err(|e| Status::new(Code::Internal, e))
	}

	async fn send_log(&self, request: Request<LogRequest>) -> Result<Response<LogResponse>, Status> {
		let request = request.into_inner();
		let mut labels = request.labels.clone();
		labels.insert("type".to_owned(), "log".to_owned());
		labels.insert("create_time".to_owned(), request.time.to_string());
		labels.insert(
			"level".to_owned(),
			match request.level {
				0 => "debug",
				1 => "info",
				2 => "warning",
				3 => "error",
				_ => "unknown",
			}
			.to_owned(),
		);
		self.loki
			.send_to_loki(labels, Duration::from_millis(request.time), request.value.as_str())
			.await
			.map(|_| Response::new(LogResponse {}))
			.map_err(|e| Status::new(Code::Internal, e))
	}
}
