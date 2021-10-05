use cheetah_microservice::tonic::{Request, Response, Status};

use crate::debug::tracer::proto::admin;

pub struct CommandTracerGRPCServer {}

#[tonic::async_trait]
impl admin::command_tracer_server::CommandTracer for CommandTracerGRPCServer {
	async fn get_rooms(&self, request: Request<admin::GetRoomsRequest>) -> Result<Response<admin::GetRoomsResponse>, tonic::Status> {
		todo!()
	}

	async fn create_session(&self, request: Request<admin::CreateSessionRequest>) -> Result<Response<admin::CreateSessionResponse>, tonic::Status> {
		todo!()
	}

	async fn set_filter(&self, request: Request<admin::SetFilterRequest>) -> Result<Response<admin::SetFilterResponse>, tonic::Status> {
		todo!()
	}

	async fn get_commands(&self, request: Request<admin::GetCommandsRequest>) -> Result<Response<admin::GetCommandsResponse>, tonic::Status> {
		todo!()
	}

	async fn close_session(&self, request: Request<admin::CloseSessionRequest>) -> Result<Response<admin::CloseSessionResponse>, tonic::Status> {
		todo!()
	}
}
