use std::collections::HashMap;
use std::time::Duration;

use serde::Serialize;

pub struct Loki {
	url: String,
}

#[derive(Debug, Serialize)]
struct Event {
	stream: HashMap<String, String>,
	values: Vec<[String; 2]>,
}

#[derive(Debug, Serialize)]
struct LokiRequest {
	streams: Vec<Event>,
}

impl Loki {
	pub fn new(loki_server_url: &str) -> Self {
		Self {
			url: format!("{}/loki/api/v1/push", loki_server_url),
		}
	}

	pub async fn send_to_loki(&self, tags: HashMap<String, String>, time: Duration, value: &str) -> Result<(), String> {
		let client = reqwest::Client::new();

		let request = LokiRequest {
			streams: vec![Event {
				stream: tags,
				values: vec![[time.as_nanos().to_string(), value.to_owned()]],
			}],
		};
		match client.post(self.url.clone()).json(&request).send().await {
			Ok(_) => Ok(()),
			Err(e) => Err(format!("Error send to loki {:?}", e)),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::time::Duration;

	use httpmock::MockServer;

	use crate::loki::Loki;

	#[tokio::test]
	pub async fn should_send_event() {
		let server = MockServer::start();
		let http_server_mock = server.mock(|when, _then| {
			when.method(httpmock::Method::POST).path("/loki/api/v1/push");
		});
		let loki_layer = Loki::new(server.base_url().as_str());
		loki_layer
			.send_to_loki(Default::default(), Duration::from_secs(0), "hello")
			.await
			.unwrap();
		std::thread::sleep(Duration::from_secs(1));
		http_server_mock.assert();
	}
}
