use std::sync::{Arc, Mutex};

use crossbeam::queue::ArrayQueue;
use thiserror::Error;
use tokio::task::JoinHandle;
use tonic::transport::{Channel, Error};

use crate::proto::matches::realtime::internal::EmptyRequest;
use crate::proto::matches::realtime::internal::realtime_client::RealtimeClient;
use crate::registry::RoomId;

///
/// Класс для чтения событий о создании новых комнат, фактически обертка над асинхронным кодом
/// чтения данных из gRPC stream. Не блокирует текущий поток, создает свой, также создает
/// tokio::runtime
///
pub struct CreateRoomEventReader {
    handler: Option<JoinHandle<()>>,
    runtime: Option<tokio::runtime::Runtime>,
    reader_result: Arc<Mutex<Option<Result<(), CreatedRoomEventReaderError>>>>,
    created_rooms: Arc<ArrayQueue<RoomId>>,
}


#[derive(Debug, Clone, Error)]
pub enum CreatedRoomEventReaderError {
    #[error("ArrayQueue overflow")]
    ArrayQueueOverflow,
    #[error("gRPC error {0}")]
    GrpcError(String),
}


impl Default for CreateRoomEventReader {
    fn default() -> Self {
        Self {
            handler: Default::default(),
            runtime: Some(tokio::runtime::Builder::new_multi_thread().build().unwrap()),
            reader_result: Arc::new(Mutex::new(None)),
            created_rooms: Arc::new(ArrayQueue::new(100)),
        }
    }
}

impl CreateRoomEventReader {
    pub fn from_address(grpc_server_address: String) -> Result<CreateRoomEventReader, Error> {
        let mut reader = Self::default();
        let handler = reader.runtime.as_ref().unwrap().block_on(async move {
            Channel::from_static(grpc_server_address.as_str()).connect().await
        });
        let channel = handler?;
        Ok(run(reader, channel))
    }

    pub fn from_channel(server_channel: Channel) -> Self {
        let mut result = Self::default();
        run(result, server_channel)
    }

    pub fn run(mut self, server_channel: Channel) -> Self {
        let created_rooms_cloned = self.created_rooms.clone();
        let reader_result_cloned = self.reader_result.clone();

        let handler = self.runtime.as_ref().unwrap().spawn(async move {
            let r = Self::read_loop(server_channel, created_rooms_cloned).await;
            let mut result_storage = reader_result_cloned.lock().unwrap();
            *result_storage = Some(r);
        });
        self.handler = Some(handler);
        self
    }

    pub fn get_created_rooms(&self) -> Result<Vec<RoomId>, CreatedRoomEventReaderError> {
        let mut result = Vec::default();
        self.check_read_thread()?;

        while let Some(room_id) = self.created_rooms.pop() {
            result.push(room_id);
        }

        Ok(result)
    }

    fn check_read_thread(&self) -> Result<(), CreatedRoomEventReaderError> {
        match self.reader_result.lock().unwrap().as_ref() {
            None => Ok(()),
            Some(e) => match e {
                Ok(_) => Ok(()),
                Err(e) => Err(e.clone()),
            },
        }
    }

    async fn read_loop(server_channel: Channel, created_rooms: Arc<ArrayQueue<RoomId>>) -> Result<(), CreatedRoomEventReaderError> {
        let mut client = RealtimeClient::new(server_channel);
        let mut response = client
            .watch_created_room_event(EmptyRequest::default())
            .await
            .map_err(|e| CreatedRoomEventReaderError::GrpcError(format!("{:?}", e)))?;

        let stream = response.get_mut();
        loop {
            match stream.message().await {
                Ok(message) => match message {
                    None => {}
                    Some(message) => {
                        if created_rooms.push(message.room_id).is_err() {
                            return Err(CreatedRoomEventReaderError::ArrayQueueOverflow);
                        }
                    }
                },
                Err(e) => return Err(CreatedRoomEventReaderError::GrpcError(format!("{:?}", e))),
            }
        }
    }
}

impl Drop fr
#[cfg(test)]
mod test {
    use std::time::Duration;

    use tokio::sync::mpsc::Sender;
    use tonic::Status;

    use crate::proto::matches::realtime::internal::RoomIdResponse;
    use crate::registry::created_room::CreateRoomEventReader;
    use crate::registry::stubs::create_stub_server;

    async fn setup_should_get_rooms(tx: Sender<Result<RoomIdResponse, Status>>) {
        tx.send(Ok(RoomIdResponse { room_id: 1 })).await.unwrap();
        tx.send(Ok(RoomIdResponse { room_id: 2 })).await.unwrap();
    }

    #[test]
    fn should_get_rooms() -> Result<(), Box<dyn std::error::Error>> {
        let (_runtime, _handler, channel) = create_stub_server(setup_should_get_rooms);
        let reader = CreateRoomEventReader::from_channel(channel);
        std::thread::sleep(Duration::from_secs(1));
        let rooms = reader.get_created_rooms().unwrap();
        assert_eq!(rooms, vec![1, 2]);
        let rooms = reader.get_created_rooms().unwrap();
        assert_eq!(rooms, vec![]);
        Ok(())
    }

    async fn setup_should_fail_when_server_halt(tx: Sender<Result<RoomIdResponse, Status>>) {
        tx.send(Err(Status::internal("error"))).await.unwrap();
    }

    // #[tokio::test]
    // async fn should_fail_when_server_halt() -> Result<(), Box<dyn std::error::Error>> {
    // 	let (server_handler, channel) = create_stub_server(setup_should_fail_when_server_halt).await;
    // 	server_handler.abort();
    // 	// wait then server finish
    // 	let _ = server_handler.await;
    // 	let reader = CreateRoomEventReader::from_channel(channel);
    // 	tokio::time::sleep(Duration::from_secs(1)).await;
    // 	assert!(reader.get_created_rooms().is_err());
    // 	Ok(())
    // }
}
