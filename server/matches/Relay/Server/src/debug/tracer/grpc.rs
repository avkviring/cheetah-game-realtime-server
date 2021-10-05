use std::sync::{Arc, Mutex};
use std::time::Duration;

use cheetah_matches_relay_common::room::RoomId;
use cheetah_microservice::tonic::{Request, Response, Status};

use crate::debug::tracer::proto::admin;
use crate::debug::tracer::{CommandTracerSessionsError, CommandTracerSessionsTask, SessionId};
use crate::room::command::execute;
use crate::server::manager::{CommandTracerSessionTaskError, RelayManager};

pub struct CommandTracerGRPCServer {
	pub manager: Arc<Mutex<RelayManager>>,
}

impl CommandTracerGRPCServer {
	pub fn new(relay_server: Arc<Mutex<RelayManager>>) -> Self {
		Self { manager: relay_server }
	}

	pub fn execute_task<T, V>(
		&self,
		room_id: RoomId,
		task: CommandTracerSessionsTask,
		receiver: std::sync::mpsc::Receiver<T>,
		converter: fn(T) -> V,
	) -> Result<Response<V>, tonic::Status> {
		let manager = self.manager.lock().unwrap();
		match manager.execute_command_trace_sessions_task(room_id, task) {
			Ok(_) => match receiver.recv_timeout(Duration::from_millis(100)) {
				Ok(result) => Result::Ok(Response::new(converter(result))),
				Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e).to_string())),
			},
			Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e).to_string())),
		}
	}
}

#[tonic::async_trait]
impl admin::command_tracer_server::CommandTracer for CommandTracerGRPCServer {
	async fn get_rooms(&self, request: Request<admin::GetRoomsRequest>) -> Result<Response<admin::GetRoomsResponse>, tonic::Status> {
		let manager = self.manager.lock().unwrap();
		match manager.get_rooms() {
			Ok(rooms) => Result::Ok(Response::new(admin::GetRoomsResponse { rooms })),
			Err(e) => Result::Err(tonic::Status::internal(e)),
		}
	}

	async fn create_session(&self, request: Request<admin::CreateSessionRequest>) -> Result<Response<admin::CreateSessionResponse>, tonic::Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let task = CommandTracerSessionsTask::CreateSession(sender);
		self.execute_task(request.get_ref().room as RoomId, task, receiver, |session_id| {
			admin::CreateSessionResponse { id: session_id as u32 }
		})
	}

	async fn set_filter(&self, request: Request<admin::SetFilterRequest>) -> Result<Response<admin::SetFilterResponse>, tonic::Status> {
		// let (sender, receiver) = std::sync::mpsc::channel();
		// let request = request.get_ref();
		// let task = CommandTracerSessionsTask::SetFilter(request.session as SessionId, request.filter.clone(), sender);
		// // self.execute_task(request.room as RoomId, task, receiver, |result| match result {
		// // 	Ok(_) => {}
		// // 	Err(_) => {}
		// // })
		todo!()
	}

	async fn get_commands(&self, request: Request<admin::GetCommandsRequest>) -> Result<Response<admin::GetCommandsResponse>, tonic::Status> {
		todo!()
	}

	async fn close_session(&self, request: Request<admin::CloseSessionRequest>) -> Result<Response<admin::CloseSessionResponse>, tonic::Status> {
		todo!()
	}
}
