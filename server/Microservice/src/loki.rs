use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tracing::field::Field;
use tracing::Event;
use tracing_subscriber::field::Visit;
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;

pub(crate) struct LokiLayer {
	url: String,
	client: reqwest::Client,
	default_values: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
struct LokiStream {
	stream: HashMap<String, String>,
	values: Vec<[String; 2]>,
}

#[derive(Debug, Serialize)]
struct LokiRequest {
	streams: Vec<LokiStream>,
}

impl LokiLayer {
	pub(crate) fn new<S: AsRef<str>>(url: S, default_values: HashMap<String, String>) -> Self {
		Self {
			url: format!("{}/loki/api/v1/push", url.as_ref()),
			client: Default::default(),
			default_values,
		}
	}
}

#[derive(Default)]
pub struct ValuesVisitor {
	values: HashMap<String, String>,
}

impl<'a> Visit for ValuesVisitor {
	fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
		if field.name() != "message" {
			self.values
				.insert(field.name().to_string().replace(".", "_"), format!("{:?}", value));
		}
	}
}

#[derive(Default)]
pub struct ValueVisitor {
	name: String,
	result: Option<String>,
}

impl ValueVisitor {
	pub fn new<S: AsRef<str>>(name: S) -> Self {
		Self {
			name: name.as_ref().to_string(),
			result: None,
		}
	}
}
impl<'a> Visit for ValueVisitor {
	fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
		if field.name() == self.name {
			self.result = Some(format!("{:?}", value));
		}
	}
}

impl<S: tracing::Subscriber> Layer<S> for LokiLayer {
	fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
		let mut values_visitor = ValuesVisitor::default();
		event.record(&mut values_visitor);
		let mut message_visitor = ValueVisitor::new("message");
		event.record(&mut message_visitor);
		let mut values = values_visitor.values;
		values.insert("level".to_string(), event.metadata().level().to_string());
		for (k, v) in self.default_values.iter() {
			values.insert(k.to_owned(), v.to_owned());
		}

		if let Some(file) = event.metadata().file() {
			values.insert("file".to_owned(), file.to_owned());
		}

		if let Some(line) = event.metadata().line() {
			values.insert("line".to_owned(), line.to_string());
		}

		if let Some(module) = event.metadata().module_path() {
			values.insert("module".to_owned(), module.to_string());
		}

		let start = SystemTime::now();
		let since_start = start.duration_since(UNIX_EPOCH).expect("cant get duration since");
		let time_ns = since_start.as_nanos().to_string();

		let request = LokiRequest {
			streams: vec![LokiStream {
				stream: values,
				values: vec![[time_ns, message_visitor.result.unwrap_or_else(|| "".to_string())]],
			}],
		};

		let client = self.client.clone();
		let url = self.url.clone();

		// не логируем события из http клиента, а то может получиться рекурсия
		if event.metadata().name().to_string().contains("hyper") {
			return;
		}

		tokio::spawn(async move {
			match client.post(url).json(&request).send().await {
				Ok(_) => {}
				Err(e) => {
					eprintln!("{:?}", e);
				}
			}
		});
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use httpmock::MockServer;
	use tokio::time::sleep;
	use tracing_subscriber::layer::SubscriberExt;
	use tracing_subscriber::Registry;

	use crate::LokiLayer;

	#[tokio::test]
	pub async fn should_send_request() {
		let server = MockServer::start();
		let http_server_mock = server.mock(|when, _then| {
			when.method(httpmock::Method::POST).path("/loki/api/v1/push");
		});
		let loki_layer = LokiLayer::new(server.base_url(), Default::default());
		let subscriber = Registry::default().with(loki_layer);
		tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

		tracing::info!("test");
		sleep(Duration::from_secs(1)).await;
		http_server_mock.assert_async().await;
	}
}
