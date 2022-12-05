use std::convert::AsRef;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tonic::Status;

use cheetah_common::commands::FieldType;
use cheetah_common::room::owner::GameObjectOwner;
use cheetah_common::room::RoomId;
use cheetah_microservice::tonic::{Request, Response};
use cheetah_microservice::trace::Trace;

use crate::debug::proto::admin;
use crate::debug::proto::shared;
use crate::debug::tracer::{TracedBothDirectionCommand, TracedCommand, TracerSessionCommand};
use crate::server::manager::RoomsServerManager;

pub struct CommandTracerGRPCService {
	pub manager: Arc<Mutex<RoomsServerManager>>,
}

impl CommandTracerGRPCService {
	#[must_use]
	pub fn new(relay_server: Arc<Mutex<RoomsServerManager>>) -> Self {
		Self { manager: relay_server }
	}

	///
	/// Выполнить задачу в relay сервере (в другом потоке), дождаться результата и преобразовать
	/// его в нужный для grpc формат
	///
	pub async fn execute_task<TaskResult, GrpcType>(
		&self,
		room_id: RoomId,
		task: TracerSessionCommand,
		receiver: std::sync::mpsc::Receiver<TaskResult>,
		converter: fn(TaskResult) -> Result<GrpcType, Status>,
	) -> Result<Response<GrpcType>, Status> {
		let manager = self.manager.lock().await;

		manager
			.execute_command_trace_sessions_task(room_id, task.clone())
			.trace_err(format!("Schedule tracer command {room_id} {task:?}"))
			.map_err(Status::internal)?;

		let result = receiver
			.recv_timeout(Duration::from_millis(100))
			.trace_err(format!("Wait tracer command {room_id} {task:?}"))
			.map_err(Status::internal)?;

		converter(result).map(Response::new)
	}
}

#[tonic::async_trait]
impl admin::command_tracer_server::CommandTracer for CommandTracerGRPCService {
	async fn create_session(&self, request: Request<admin::CreateSessionRequest>) -> Result<Response<admin::CreateSessionResponse>, Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let task = TracerSessionCommand::CreateSession(sender);
		self.execute_task(request.get_ref().room, task, receiver, |session_id| {
			Ok(admin::CreateSessionResponse { id: u32::from(session_id) })
		})
		.await
	}

	async fn set_filter(&self, request: Request<admin::SetFilterRequest>) -> Result<Response<admin::SetFilterResponse>, Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let request = request.get_ref();
		let task = TracerSessionCommand::SetFilter(
			request
				.session
				.try_into()
				.map_err(|e| Status::invalid_argument(format!("session is too large: {e}")))?,
			request.filter.clone(),
			sender,
		);
		self.execute_task(request.room, task, receiver, |_| Ok(admin::SetFilterResponse {})).await
	}

	async fn get_commands(&self, request: Request<admin::GetCommandsRequest>) -> Result<Response<admin::GetCommandsResponse>, Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let request = request.get_ref();
		let task = TracerSessionCommand::GetCommands(
			request
				.session
				.try_into()
				.map_err(|e| Status::invalid_argument(format!("session is too large: {e}")))?,
			sender,
		);
		self.execute_task(request.room, task, receiver, |result| {
			result
				.trace_err("Get commands for trace")
				.map_err(Status::internal)
				.map(|commands| admin::GetCommandsResponse {
					commands: commands.into_iter().map(admin::Command::from).collect(),
				})
		})
		.await
	}

	async fn close_session(&self, request: Request<admin::CloseSessionRequest>) -> Result<Response<admin::CloseSessionResponse>, Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let request = request.get_ref();
		let task = TracerSessionCommand::CloseSession(
			request
				.session
				.try_into()
				.map_err(|e| Status::invalid_argument(format!("session is too large: {e}")))?,
			sender,
		);
		self.execute_task(request.room, task, receiver, |result| {
			result
				.trace_err("Close tracer session")
				.map_err(Status::internal)
				.map(|_| admin::CloseSessionResponse {})
		})
		.await
	}
}

impl From<TracedCommand> for admin::Command {
	fn from(command: TracedCommand) -> Self {
		let direction = match command.network_command {
			TracedBothDirectionCommand::C2S(_) => "c2s",
			TracedBothDirectionCommand::S2C(_) => "s2c",
		};

		let object_id = match command.network_command.get_object_id() {
			None => "none".to_owned(),
			Some(id) => match &id.owner {
				GameObjectOwner::Room => {
					format!("root({})", id.id)
				}
				GameObjectOwner::Member(member_id) => {
					format!("member({member_id},{})", id.id)
				}
			},
		};
		let template = command.template.map(u32::from);
		let command_name: String = match &command.network_command {
			TracedBothDirectionCommand::C2S(command) => command.as_ref().to_owned(),
			TracedBothDirectionCommand::S2C(command) => command.as_ref().to_owned(),
		};
		let field_id = command.network_command.get_field_id().map(u32::from);
		let field_type = command
			.network_command
			.get_field_type()
			.map(|field_type| match field_type {
				FieldType::Long => shared::FieldType::Long,
				FieldType::Double => shared::FieldType::Double,
				FieldType::Structure => shared::FieldType::Structure,
				FieldType::Event => shared::FieldType::Event,
			})
			.map(|field_type| field_type as i32);
		let value = get_string_value(&command);

		Self {
			time: command.time,
			direction: direction.to_owned(),
			command: command_name,
			object_id,
			user_id: u32::from(command.member),
			template,
			value,
			field_id,
			field_type,
		}
	}
}

fn get_string_value(command: &TracedCommand) -> String {
	match &command.network_command {
		TracedBothDirectionCommand::C2S(command) => command.get_trace_string(),
		TracedBothDirectionCommand::S2C(command) => command.get_trace_string(),
	}
}

#[cfg(test)]
pub mod test {
	use cheetah_common::commands::binary_value::BinaryValue;
	use cheetah_common::commands::c2s::C2SCommand;
	use cheetah_common::commands::types::event::EventCommand;
	use cheetah_common::room::object::GameObjectId;
	use cheetah_common::room::owner::GameObjectOwner;

	use crate::debug::proto::admin;
	use crate::debug::proto::shared;
	use crate::debug::tracer::{TracedBothDirectionCommand, TracedCommand};

	#[test]
	pub fn should_convert() {
		let command = TracedCommand {
			time: 1.1,
			template: Some(155),
			member: 255,
			network_command: TracedBothDirectionCommand::C2S(C2SCommand::Event(EventCommand {
				object_id: GameObjectId::new(100, GameObjectOwner::Room),
				field_id: 555,
				event: BinaryValue::from(vec![10, 20, 30].as_slice()),
			})),
		};

		let grpc_command = admin::Command::from(command);
		assert_eq!(
			grpc_command,
			admin::Command {
				time: 1.1,
				direction: "c2s".to_owned(),
				command: "Event".to_owned(),
				object_id: "root(100)".to_owned(),
				user_id: 255,
				template: Some(155),
				value: "[10, 20, 30]".to_owned(),
				field_id: Some(555),
				field_type: Some(shared::FieldType::Event as i32),
			}
		);
	}

	#[test]
	pub fn should_convert_with_none_template_and_none_field() {
		let command = TracedCommand {
			time: 1.1,
			template: None,
			member: 255,
			network_command: TracedBothDirectionCommand::C2S(C2SCommand::AttachToRoom),
		};

		let grpc_command = admin::Command::from(command);
		assert_eq!(
			grpc_command,
			admin::Command {
				time: 1.1,
				direction: "c2s".to_owned(),
				command: "AttachToRoom".to_owned(),
				object_id: "none".to_owned(),
				user_id: 255,
				template: None,
				value: String::new(),
				field_id: None,
				field_type: None,
			}
		);
	}
}
