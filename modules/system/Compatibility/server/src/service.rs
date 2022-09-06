use std::collections::HashMap;

use chrono::{DateTime, Utc};
use tonic::{Request, Response, Status};

use crate::proto;
use crate::proto::{CheckCompatibilityRequest, CheckCompatibilityResponse};

pub struct Service {
	versions: HashMap<String, DateTime<Utc>>,
}

impl Service {
	pub fn new(versions: HashMap<String, DateTime<Utc>>) -> Self {
		Self { versions }
	}
}

#[tonic::async_trait]
impl proto::compatibility_checker_server::CompatibilityChecker for Service {
	async fn check_compatibility(&self, request: Request<CheckCompatibilityRequest>) -> Result<Response<CheckCompatibilityResponse>, Status> {
		let request_version = request.into_inner().version;
		let status = match self.versions.get(&request_version) {
			None => proto::check_compatibility_response::Status::Unsupported,
			Some(expired) => {
				let remaining = (expired.timestamp_millis() - Utc::now().timestamp_millis()) / (1000 * 60 * 60);
				match remaining {
					0..=4 => proto::check_compatibility_response::Status::PlanningUnsupportedSoon,
					5..=24 => proto::check_compatibility_response::Status::PlanningUnsupportedAfterSomeHours,
					_ => proto::check_compatibility_response::Status::Supported,
				}
			}
		};
		Ok(Response::new(CheckCompatibilityResponse { status: status as i32 }))
	}
}

#[cfg(test)]
mod tests {
	use std::ops::Add;

	use chrono::{Duration, Utc};
	use tonic::Request;

	use crate::proto;
	use crate::proto::compatibility_checker_server::CompatibilityChecker;
	use crate::proto::CheckCompatibilityRequest;
	use crate::service::Service;

	#[tokio::test]
	async fn should_unsupported() {
		let service = Service::new(Default::default());
		let response = service
			.check_compatibility(Request::new(CheckCompatibilityRequest { version: "0.0.1".to_owned() }))
			.await;
		assert_eq!(
			response.unwrap().into_inner().status,
			proto::check_compatibility_response::Status::Unsupported as i32
		);
	}

	#[tokio::test]
	async fn should_planning_unsupported_soon() {
		let service = Service::new(vec![("0.0.1".to_owned(), Utc::now().add(Duration::hours(2)))].into_iter().collect());

		let response = service
			.check_compatibility(Request::new(CheckCompatibilityRequest { version: "0.0.1".to_owned() }))
			.await;
		assert_eq!(
			response.unwrap().into_inner().status,
			proto::check_compatibility_response::Status::PlanningUnsupportedSoon as i32
		);
	}

	#[tokio::test]
	async fn should_planning_unsupported_after_some_hours() {
		let service = Service::new(vec![("0.0.1".to_owned(), Utc::now().add(Duration::hours(6)))].into_iter().collect());

		let response = service
			.check_compatibility(Request::new(CheckCompatibilityRequest { version: "0.0.1".to_owned() }))
			.await;
		assert_eq!(
			response.unwrap().into_inner().status,
			proto::check_compatibility_response::Status::PlanningUnsupportedAfterSomeHours as i32
		);
	}

	#[tokio::test]
	async fn should_supported() {
		let service = Service::new(vec![("0.0.1".to_owned(), Utc::now().add(Duration::days(2)))].into_iter().collect());

		let response = service
			.check_compatibility(Request::new(CheckCompatibilityRequest { version: "0.0.1".to_owned() }))
			.await;
		assert_eq!(
			response.unwrap().into_inner().status,
			proto::check_compatibility_response::Status::Supported as i32
		);
	}
}
