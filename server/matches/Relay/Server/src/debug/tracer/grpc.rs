use std::convert::AsRef;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use cheetah_matches_relay_common::commands::c2s::C2SCommand;
use cheetah_matches_relay_common::commands::s2c::S2CCommand;
use cheetah_matches_relay_common::commands::FieldType;
use cheetah_matches_relay_common::room::owner::GameObjectOwner;
use cheetah_matches_relay_common::room::RoomId;
use cheetah_microservice::tonic::{Request, Response};

use crate::debug::proto::admin;
use crate::debug::proto::shared;
use crate::debug::tracer::{CommandTracerSessionsTask, SessionId, TracedCommand, TracedBothDirectionCommand};
use crate::server::manager::ServerManager;

pub struct CommandTracerGRPCService {
	pub manager: Arc<Mutex<ServerManager>>,
}

impl CommandTracerGRPCService {
	pub fn new(relay_server: Arc<Mutex<ServerManager>>) -> Self {
		Self { manager: relay_server }
	}

	///
	/// Выполнить задачу в relay сервере (в другом потоке), дождаться результата и преобразовать
	/// его в нужный для grpc формат
	///
	pub fn execute_task<T, V>(
		&self,
		room_id: RoomId,
		task: CommandTracerSessionsTask,
		receiver: std::sync::mpsc::Receiver<T>,
		converter: fn(T) -> Result<Response<V>, tonic::Status>,
	) -> Result<Response<V>, tonic::Status> {
		let manager = self.manager.lock().unwrap();
		match manager.execute_command_trace_sessions_task(room_id, task) {
			Ok(_) => match receiver.recv_timeout(Duration::from_millis(100)) {
				Ok(result) => converter(result),
				Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e))),
			},
			Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e))),
		}
	}
}

#[tonic::async_trait]
impl admin::command_tracer_server::CommandTracer for CommandTracerGRPCService {
	async fn create_session(
		&self,
		request: Request<admin::CreateSessionRequest>,
	) -> Result<Response<admin::CreateSessionResponse>, tonic::Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let task = CommandTracerSessionsTask::CreateSession(sender);
		self.execute_task(request.get_ref().room as RoomId, task, receiver, |session_id| {
			Result::Ok(Response::new(admin::CreateSessionResponse { id: session_id as u32 }))
		})
	}

	async fn set_filter(
		&self,
		request: Request<admin::SetFilterRequest>,
	) -> Result<Response<admin::SetFilterResponse>, tonic::Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let request = request.get_ref();
		let task = CommandTracerSessionsTask::SetFilter(request.session as SessionId, request.filter.clone(), sender);
		self.execute_task(request.room as RoomId, task, receiver, |result| match result {
			Ok(_) => Result::Ok(Response::new(admin::SetFilterResponse {})),
			Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e))),
		})
	}

	async fn get_commands(
		&self,
		request: Request<admin::GetCommandsRequest>,
	) -> Result<Response<admin::GetCommandsResponse>, tonic::Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let request = request.get_ref();
		let task = CommandTracerSessionsTask::GetCommands(request.session as SessionId, sender);
		self.execute_task(request.room as RoomId, task, receiver, |result| match result {
			Ok(commands) => Result::Ok(Response::new(admin::GetCommandsResponse {
				commands: commands.into_iter().map(admin::Command::from).collect(),
			})),
			Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e))),
		})
	}

	async fn close_session(
		&self,
		request: Request<admin::CloseSessionRequest>,
	) -> Result<Response<admin::CloseSessionResponse>, tonic::Status> {
		let (sender, receiver) = std::sync::mpsc::channel();
		let request = request.get_ref();
		let task = CommandTracerSessionsTask::CloseSession(request.session as SessionId, sender);
		self.execute_task(request.room as RoomId, task, receiver, |result| match result {
			Ok(_) => Result::Ok(Response::new(admin::CloseSessionResponse {})),
			Err(e) => Result::Err(tonic::Status::internal(format!("{:?}", e))),
		})
	}
}

impl From<TracedCommand> for admin::Command {
	fn from(command: TracedCommand) -> Self {
		let direction = match command.network_command {
			TracedBothDirectionCommand::C2S(_) => "c2s",
			TracedBothDirectionCommand::S2C(_) => "s2c",
		};

		let object_id = match command.network_command.get_object_id() {
			None => "none".to_string(),
			Some(id) => match &id.owner {
				GameObjectOwner::Room => {
					format!("root({})", id.id)
				}
				GameObjectOwner::Member(user) => {
					format!("user({},{})", user, id.id)
				}
			},
		};
		let template = command.template.map(|id| id as u32);
		let command_name: String = match &command.network_command {
			TracedBothDirectionCommand::C2S(command) => command.as_ref().to_string(),
			TracedBothDirectionCommand::S2C(command) => command.as_ref().to_string(),
		};
		let field_id = command.network_command.get_field_id().map(|field_id| field_id as u32);
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
			direction: direction.to_string(),
			command: command_name,
			object_id,
			user_id: command.user as u32,
			template,
			value,
			field_id,
			field_type,
		}
	}
}

fn get_string_value(command: &TracedCommand) -> String {
	match &command.network_command {
		TracedBothDirectionCommand::C2S(command) => match command {
			C2SCommand::Create(command) => {
				format!("access({:?}), template({:?}) ", command.access_groups.0, command.template)
			}
			C2SCommand::Created(_) => "".to_string(),
			C2SCommand::SetLong(command) => {
				format!("{:?}", command.value)
			}
			C2SCommand::IncrementLongValue(command) => {
				format!("{:?}", command.increment)
			}
			C2SCommand::CompareAndSetLong(command) => {
				format!(
					"new = {:?}, current = {:?}, reset = {:?}",
					command.new, command.current, command.reset
				)
			}
			C2SCommand::SetDouble(command) => {
				format!("{:?}", command.value)
			}
			C2SCommand::IncrementDouble(command) => {
				format!("{:?}", command.increment)
			}
			C2SCommand::SetStructure(command) => {
				format!("{:?}", command.structure)
			}
			C2SCommand::Event(command) => {
				format!("{:?}", command.event)
			}
			C2SCommand::TargetEvent(command) => {
				format!("target_user = {:?}, value = {:?}", command.target, command.event.event)
			}
			C2SCommand::Delete(_) => "".to_string(),
			C2SCommand::AttachToRoom => "".to_string(),
			C2SCommand::DetachFromRoom => "".to_string(),
			C2SCommand::DeleteField(command) => {
				format!("field_type = {:?}", command.field_type)
			}
		},
		TracedBothDirectionCommand::S2C(command) => match command {
			S2CCommand::Create(command) => format!("access({:?}), template({:?}) ", command.access_groups.0, command.template),
			S2CCommand::Created(_) => "".to_string(),
			S2CCommand::SetLong(command) => format!("{:?}", command.value),
			S2CCommand::SetDouble(command) => format!("{:?}", command.value),
			S2CCommand::SetStructure(command) => format!("{:?}", command.structure),
			S2CCommand::Event(command) => format!("{:?}", command.event),
			S2CCommand::Delete(_) => "".to_string(),
			S2CCommand::DeleteField(_) => "".to_string(),
		},
	}
}

#[cfg(test)]
pub mod test {
	use cheetah_matches_relay_common::commands::c2s::C2SCommand;
	use cheetah_matches_relay_common::commands::types::event::EventCommand;
	use cheetah_matches_relay_common::commands::CommandBuffer;
	use cheetah_matches_relay_common::room::object::GameObjectId;
	use cheetah_matches_relay_common::room::owner::GameObjectOwner;

	use crate::debug::proto::admin;
	use crate::debug::proto::shared;
	use crate::debug::tracer::{TracedCommand, TracedBothDirectionCommand};

	#[test]
	pub fn should_convert() {
		let command = TracedCommand {
			time: 1.1,
			template: Option::Some(155),
			user: 255,
			network_command: TracedBothDirectionCommand::C2S(C2SCommand::Event(EventCommand {
				object_id: GameObjectId::new(100, GameObjectOwner::Room),
				field_id: 555,
				event: CommandBuffer::from_slice(vec![10, 20, 30].as_slice()).unwrap(),
			})),
		};

		let grpc_command = admin::Command::from(command);
		assert_eq!(
			grpc_command,
			admin::Command {
				time: 1.1,
				direction: "c2s".to_string(),
				command: "Event".to_string(),
				object_id: "root(100)".to_string(),
				user_id: 255,
				template: Option::Some(155),
				value: "[10, 20, 30]".to_string(),
				field_id: Option::Some(555),
				field_type: Option::Some(shared::FieldType::Event as i32)
			}
		)
	}

	#[test]
	pub fn should_convert_with_none_template_and_none_field() {
		let command = TracedCommand {
			time: 1.1,
			template: None,
			user: 255,
			network_command: TracedBothDirectionCommand::C2S(C2SCommand::AttachToRoom),
		};

		let grpc_command = admin::Command::from(command);
		assert_eq!(
			grpc_command,
			admin::Command {
				time: 1.1,
				direction: "c2s".to_string(),
				command: "AttachToRoom".to_string(),
				object_id: "none".to_string(),
				user_id: 255,
				template: Option::None,
				value: "".to_string(),
				field_id: Option::None,
				field_type: Option::None
			}
		)
	}
}
