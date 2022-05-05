use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tonic::{Request, Response, Status};

use crate::proto;
use crate::proto::{VersionExpirationRequest, VersionExpirationResponse};

pub struct Service {
	versions: HashMap<String, DateTime<Utc>>,
}

impl Service {
	pub fn new(versions: HashMap<String, DateTime<Utc>>) -> Self {
		Self { versions }
	}
}

#[tonic::async_trait]
impl proto::client_version_server::ClientVersion for Service {
	async fn get_version_expiration(
		&self,
		request: Request<VersionExpirationRequest>,
	) -> Result<Response<VersionExpirationResponse>, Status> {
		let request_version = request.into_inner().version;
		let expired = match self.versions.get(&request_version) {
			None => i64::MIN,
			Some(expired) => expired.timestamp_millis() - Utc::now().timestamp_millis(),
		};
		Ok(Response::new(VersionExpirationResponse {
			expired_time_in_ms: expired,
		}))
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;

	use chrono::{Duration, Utc};
	use tonic::Request;

	use crate::proto::client_version_server::ClientVersion;
	use crate::proto::VersionExpirationRequest;
	use crate::service::Service;

	#[tokio::test]
	async fn should_expired() {
		let service = Service::new(Default::default());
		let response = service
			.get_version_expiration(Request::new(VersionExpirationRequest {
				version: "0.0.1".to_owned(),
			}))
			.await;
		assert!(response.unwrap().into_inner().expired_time_in_ms < 0);
	}

	#[tokio::test]
	async fn should_not_expired() {
		let service = Service::new(
			vec![("0.0.1".to_owned(), Utc::now().to_owned().add(Duration::days(1)))]
				.into_iter()
				.collect(),
		);

		let response = service
			.get_version_expiration(Request::new(VersionExpirationRequest {
				version: "0.0.1".to_owned(),
			}))
			.await;
		assert!(response.unwrap().into_inner().expired_time_in_ms > Duration::hours(23).num_milliseconds());
	}
}
