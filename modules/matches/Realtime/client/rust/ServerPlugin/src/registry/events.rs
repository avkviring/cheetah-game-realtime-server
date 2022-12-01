use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossbeam::queue::ArrayQueue;
use thiserror::Error;
use tokio::task::JoinHandle;
use tonic::transport::{Channel, Error};

use crate::proto::matches::realtime::internal::realtime_client::RealtimeClient;
use crate::proto::matches::realtime::internal::room_lifecycle_response::RoomLifecycleType;
use crate::proto::matches::realtime::internal::{EmptyRequest, RoomLifecycleResponse};
use crate::registry::RoomId;

///
/// Класс для чтения событий о создании новых комнат, фактически обертка над асинхронным кодом
/// чтения данных из gRPC stream. Не блокирует текущий поток, создает свой, также создает
/// tokio::runtime
///
pub struct RoomLifecycleEventReader {
	handler: Option<JoinHandle<()>>,
	runtime: Option<tokio::runtime::Runtime>,
	reader_result: Arc<Mutex<Option<Result<(), RoomLifecycleEventReaderError>>>>,
	created_rooms: Arc<ArrayQueue<RoomId>>,
	deleted_rooms: Arc<ArrayQueue<RoomId>>,
}

#[derive(Debug, Clone, Error)]
pub enum RoomLifecycleEventReaderError {
	#[error("CreatedRoomQueueOverflow overflow")]
	CreatedRoomQueueOverflow,
	#[error("DeletedRoomQueueOverflow overflow")]
	DeletedRoomQueueOverflow,
	#[error("gRPC error {0}")]
	GrpcError(String),
	#[error("UnknownRoomLifecycleType")]
	UnknownRoomLifecycleType,
}

impl Default for RoomLifecycleEventReader {
	fn default() -> Self {
		Self {
			handler: Default::default(),
			runtime: Some(tokio::runtime::Builder::new_multi_thread().build().unwrap()),
			reader_result: Arc::new(Mutex::new(None)),
			created_rooms: Arc::new(ArrayQueue::new(100)),
			deleted_rooms: Arc::new(ArrayQueue::new(100)),
		}
	}
}

impl RoomLifecycleEventReader {
	pub fn from_address(grpc_server_address: String) -> Result<RoomLifecycleEventReader, Error> {
		let reader = Self::default();
		let handler = reader
			.runtime
			.as_ref()
			.unwrap()
			.block_on(async move { Channel::from_shared(grpc_server_address).unwrap().connect().await });
		let channel = handler?;
		Ok(reader.run(channel))
	}

	pub fn from_channel(server_channel: Channel) -> Self {
		let reader = Self::default();
		reader.run(server_channel)
	}

	pub fn run(mut self, server_channel: Channel) -> Self {
		let created_rooms = self.created_rooms.clone();
		let deleted_rooms = self.deleted_rooms.clone();
		let reader_result = self.reader_result.clone();

		let handler = self.runtime.as_ref().unwrap().spawn(async move {
			let r = Self::reader_loop(server_channel, created_rooms, deleted_rooms).await;
			let mut result_storage = reader_result.lock().unwrap();
			*result_storage = Some(r);
		});
		self.handler = Some(handler);
		self
	}

	pub fn pop_create_room(&self) -> Result<Option<RoomId>, RoomLifecycleEventReaderError> {
		self.assert_reader_thread_alive()?;
		Ok(self.created_rooms.pop())
	}

	pub fn pop_deleted_rooms(&self) -> Result<Option<RoomId>, RoomLifecycleEventReaderError> {
		self.assert_reader_thread_alive()?;
		Ok(self.deleted_rooms.pop())
	}

	fn assert_reader_thread_alive(&self) -> Result<(), RoomLifecycleEventReaderError> {
		match self.reader_result.lock().unwrap().as_ref() {
			None => Ok(()),
			Some(e) => match e {
				Ok(_) => Ok(()),
				Err(e) => Err(e.clone()),
			},
		}
	}

	async fn reader_loop(
		server_channel: Channel,
		created_rooms: Arc<ArrayQueue<RoomId>>,
		deleted_rooms: Arc<ArrayQueue<RoomId>>,
	) -> Result<(), RoomLifecycleEventReaderError> {
		let mut client = RealtimeClient::new(server_channel);
		let mut response = client
			.watch_room_lifecycle_event(EmptyRequest::default())
			.await
			.map_err(|e| RoomLifecycleEventReaderError::GrpcError(format!("{:?}", e)))?;
		let stream = response.get_mut();
		loop {
			let message = stream
				.message()
				.await
				.map_err(|e| RoomLifecycleEventReaderError::GrpcError(format!("{:?}", e)))?;
			Self::process_message(created_rooms.clone(), deleted_rooms.clone(), message)?;
		}
	}

	fn process_message(
		created_rooms: Arc<ArrayQueue<RoomId>>,
		deleted_rooms: Arc<ArrayQueue<RoomId>>,
		message: Option<RoomLifecycleResponse>,
	) -> Result<(), RoomLifecycleEventReaderError> {
		if let Some(message) = message {
			match RoomLifecycleType::from_i32(message.r#type).ok_or_else(|| RoomLifecycleEventReaderError::UnknownRoomLifecycleType)? {
				RoomLifecycleType::Created => created_rooms
					.push(message.room_id)
					.map_err(|_| RoomLifecycleEventReaderError::CreatedRoomQueueOverflow)?,
				RoomLifecycleType::Deleted => deleted_rooms
					.push(message.room_id)
					.map_err(|_| RoomLifecycleEventReaderError::DeletedRoomQueueOverflow)?,
			}
		}
		Ok(())
	}
}

impl Drop for RoomLifecycleEventReader {
	fn drop(&mut self) {
		let runtime = self.runtime.take().unwrap();
		runtime.shutdown_timeout(Duration::from_millis(100))
	}
}

#[cfg(test)]
mod test {
	use std::thread;
	use std::time::Duration;

	use tokio::sync::mpsc::Sender;
	use tonic::Status;

	use crate::proto::matches::realtime::internal::room_lifecycle_response::RoomLifecycleType;
	use crate::proto::matches::realtime::internal::RoomLifecycleResponse;
	use crate::registry::events::RoomLifecycleEventReader;
	use crate::registry::stubs::create_stub_server;

	async fn setup_should_get_rooms(tx: Sender<Result<RoomLifecycleResponse, Status>>) {
		tx.send(Ok(RoomLifecycleResponse {
			room_id: 1,
			r#type: RoomLifecycleType::Created as i32,
		}))
		.await
		.unwrap();
		tx.send(Ok(RoomLifecycleResponse {
			room_id: 2,
			r#type: RoomLifecycleType::Created as i32,
		}))
		.await
		.unwrap();
		tx.send(Ok(RoomLifecycleResponse {
			room_id: 3,
			r#type: RoomLifecycleType::Deleted as i32,
		}))
		.await
		.unwrap();
	}

	#[test]
	fn should_get_rooms() {
		let (_runtime, _handler, channel) = create_stub_server(setup_should_get_rooms);
		let reader = RoomLifecycleEventReader::from_channel(channel);
		std::thread::sleep(Duration::from_secs(1));

		let room_id = reader.pop_create_room().unwrap();
		assert_eq!(room_id.unwrap(), 1);

		let room_id = reader.pop_create_room().unwrap();
		assert_eq!(room_id.unwrap(), 2);

		let room_id = reader.pop_create_room().unwrap();
		assert!(room_id.is_none());

		let room_id = reader.pop_deleted_rooms().unwrap();
		assert_eq!(room_id.unwrap(), 3);

		let room_id = reader.pop_deleted_rooms().unwrap();
		assert!(room_id.is_none());
	}

	async fn setup_should_fail_when_server_halt(tx: Sender<Result<RoomLifecycleResponse, Status>>) {
		tx.send(Err(Status::internal("error"))).await.unwrap();
	}

	#[test]
	fn should_fail_when_server_halt() {
		let (_runtime, handler, channel) = create_stub_server(setup_should_fail_when_server_halt);
		handler.abort();
		let reader = RoomLifecycleEventReader::from_channel(channel);
		thread::sleep(Duration::from_secs(1));
		assert!(reader.pop_create_room().is_err());
		assert!(reader.pop_deleted_rooms().is_err());
	}
}
