use crate::grpc::userstore::{fetch_server::Fetch, GetIntReply, GetIntRequest};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct FetchService {}

#[tonic::async_trait]
impl Fetch for FetchService {
	async fn get_int(
		&self,
		request: Request<GetIntRequest>,
	) -> Result<Response<GetIntReply>, Status> {
		let reply = GetIntReply { value: 0 };
		Ok(Response::new(reply))
	}
}
