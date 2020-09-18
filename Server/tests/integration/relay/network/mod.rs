use std::{io, thread};
use std::cmp::min;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, mpsc, Mutex};
use std::time::Duration;

use cheetah_relay_common::network::command::{CommandCode, Decoder, Encoder};
use cheetah_relay_common::network::command::load::LoadGameObjectCommand;
use cheetah_relay_common::network::command::meta::c2s::C2SMetaCommandInformation;
use cheetah_relay_common::network::command::meta::s2c::S2CMetaCommandInformation;
use cheetah_relay_common::network::hash::HashValue;
use cheetah_relay_common::network::niobuffer::NioBuffer;
use cheetah_relay_common::room::access::AccessGroups;
use cheetah_relay_common::room::fields::GameObjectFields;
use cheetah_relay_common::room::object::ClientGameObjectId;
use cheetah_relay_common::room::owner::ClientOwner;
use rand::Rng;
use stderrlog::Timestamp;

use cheetah_relay::room::objects::id::{ServerGameObjectId, ServerOwner};
use cheetah_relay::room::request::{ClientInfo, RoomRequest};
use cheetah_relay::rooms::Rooms;
use cheetah_relay::server::{Server, ServerBuilder};

pub mod autocreate;


#[test]
fn should_connect_client_to_room() {
	let addr = "127.0.0.1:5050";
	let server = setup(addr);
	let (room_hash, mut clients) = create_room(&server.rooms);
	let client_hash = clients.pop().unwrap();
	
	let mut buffer = NioBuffer::new();
	create_client_and_send_hashes(&mut buffer, &room_hash, &client_hash);
	
	let mut stream = TcpStream::connect(addr).unwrap();
	send(&mut stream, &mut buffer);
	
	let clients = get_clients(server.rooms.clone(), &room_hash);
	assert_eq!(clients.iter().any(|p| p.hash == client_hash), true)
}

#[test]
fn should_disconnect_client_from_room() {
	let addr = "127.0.0.1:5051";
	let server = setup(addr);
	let (room_hash, mut clients) = create_room(&server.rooms);
	let client_hash = clients.pop().unwrap();
	{
		let mut buffer = NioBuffer::new();
		create_client_and_send_hashes(&mut buffer, &room_hash, &client_hash);
		let mut stream = TcpStream::connect(addr).unwrap();
		send(&mut stream, &mut buffer);
	}
	thread::sleep(Duration::from_secs(1));
	let clients = get_clients(server.rooms.clone(), &room_hash);
	assert_eq!(clients.is_empty(), true)
}

#[test]
fn should_client_create_object() {
	let addr = "127.0.0.1:5052";
	let server = setup(addr);
	let (room_hash, mut clients) = create_room(&server.rooms);
	let client_hash = clients.pop().unwrap();
	let mut buffer = NioBuffer::new();
	create_client_and_send_hashes(&mut buffer, &room_hash, &client_hash);
	let object_id = ServerGameObjectId::new(100, ServerOwner::Root);
	create_object(&mut buffer, object_id.clone());
	
	let mut stream = TcpStream::connect(addr).unwrap();
	send(&mut stream, &mut buffer);
	
	let objects = get_objects(server.rooms.clone(), &room_hash);
	assert_eq!(
		objects
			.iter()
			.any(|id| *id == object_id),
		true
	);
}


///
/// Проверям что команды с клиента не возращаются ему же
#[test]
fn should_dont_send_upload_for_self_object() {
	let addr = "127.0.0.1:5053";
	let server = setup(addr);
	let (room_hash, mut clients) = create_room(&server.rooms);
	let client_hash = clients.pop().unwrap();
	let mut buffer = NioBuffer::new();
	
	create_client_and_send_hashes(&mut buffer, &room_hash, &client_hash);
	let object_id = ServerGameObjectId::new(100, ServerOwner::Root);
	create_object(&mut buffer, object_id);
	
	let mut stream = TcpStream::connect(addr).unwrap();
	send(&mut stream, &mut buffer);
	
	let mut readed = NioBuffer::new();
	stream
		.set_read_timeout(Option::Some(Duration::from_secs(2)))
		.expect("socket set_read_timeout error");
	match stream.read(readed.to_slice()) {
		Ok(_) => {
			assert!(false);
		}
		Err(e) => {
			if e.kind() == io::ErrorKind::WouldBlock {
				// нет данных для чтения - и это правилно
				assert!(true);
			} else {
				assert!(false);
			}
		}
	};
}


///
/// Проверяем загрузку объекта при подключении второго клиента
///
#[test]
fn should_receive_command_from_server() {
	let addr = "127.0.0.1:5054";
	let server = setup(addr);
	let (room_hash, mut clients) = create_room(&server.rooms);
	let client_a = clients.pop().unwrap();
	let client_b = clients.pop().unwrap();
	
	let mut buffer_for_write_client_b = NioBuffer::new();
	create_client_and_send_hashes(&mut buffer_for_write_client_b, &room_hash, &client_b);
	let mut stream_for_client_b = TcpStream::connect(addr).unwrap();
	send(&mut stream_for_client_b, &mut buffer_for_write_client_b);
	
	let mut buffer_for_write_client_a = NioBuffer::new();
	create_client_and_send_hashes(&mut buffer_for_write_client_a, &room_hash, &client_a);
	create_object(&mut buffer_for_write_client_a, ServerGameObjectId::new(100, ServerOwner::Root));
	let mut stream_for_client_a = TcpStream::connect(addr).unwrap();
	send(&mut stream_for_client_a, &mut buffer_for_write_client_a);
	
	thread::sleep(Duration::from_secs(2));
	let mut readed = NioBuffer::new();
	stream_for_client_b
		.set_read_timeout(Option::Some(Duration::from_secs(2)))
		.expect("set_read_timeout error");
	let size = stream_for_client_b.read(readed.to_slice()).unwrap();
	readed.set_position(0).expect("");
	readed.set_limit(size).expect("");
	
	
	let readed_meta = S2CMetaCommandInformation::decode(&mut readed).unwrap();
	assert_eq!(
		readed_meta.command_code,
		LoadGameObjectCommand::COMMAND_CODE
	);
	let command = LoadGameObjectCommand::decode(&mut readed).unwrap();
	assert_eq!(*command.fields.long_counters.get(&10).unwrap(), 55);
	assert_eq!(*command.fields.float_counters.get(&15).unwrap() as i64, 15);
	assert_eq!(
		command.fields.structures.get(&5).unwrap(),
		&vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]
	);
}

fn create_object(buffer: &mut NioBuffer, object_id: ServerGameObjectId) {
	C2SMetaCommandInformation
	::new(LoadGameObjectCommand::COMMAND_CODE, 0)
		.encode(buffer)
		.ok();
	
	
	let mut fields = GameObjectFields::default();
	fields.long_counters.insert(10, 55);
	fields.float_counters.insert(15, 15.0);
	fields
		.structures
		.insert(5, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
	
	let command = LoadGameObjectCommand {
		object_id: ClientGameObjectId::new(object_id.id, ClientOwner::Root),
		template: 123,
		access_groups: AccessGroups::from(0b110),
		fields,
	};
	
	command.encode(buffer).unwrap();
}

/// создать клиента и  отправить хеш
fn create_client_and_send_hashes(
	buffer: &mut NioBuffer,
	room_hash: &HashValue,
	client_hash: &HashValue,
) {
	send_hash(&room_hash, buffer);
	send_hash(&client_hash, buffer);
}

/// создать комнату
fn create_room(rooms: &Arc<Mutex<Rooms>>) -> (HashValue, Vec<HashValue>) {
	let room_hash = HashValue::from("room_hash");
	let clients = vec![
		HashValue::from("client_hash_a"),
		HashValue::from("client_hash_b"),
	];
	{
		let rooms = &*rooms.clone();
		let mut rooms = rooms.lock().unwrap();
		rooms.create_room(&room_hash);
		clients.iter().for_each(|c| {
			rooms
				.send_room_request(
					&room_hash,
					RoomRequest::AddWaitingClient(c.clone(), AccessGroups::from(0b110)),
				)
				.unwrap_or_default();
		});
	}
	thread::sleep(Duration::from_secs(1));
	(room_hash, clients)
}

/// отправляем хеш порциями для проверки алгоритма чтения на сервере
fn send_hash(hash: &HashValue, buffer: &mut NioBuffer) {
	buffer.write_bytes(&hash.value).ok();
}

fn send(stream: &mut TcpStream, data: &mut NioBuffer) {
	data.flip();
	let mut rng = rand::thread_rng();
	while data.has_remaining() {
		let block_size = min(rng.gen_range(0, 200), data.remaining());
		let size = data.read_to_vec(block_size).ok().unwrap();
		stream.write(&size).ok();
		thread::sleep(Duration::from_millis(100));
	}
	thread::sleep(Duration::from_secs(1));
}

fn setup(addr: &'static str) -> Server {
	init_logger();
	ServerBuilder::new(addr.to_string()).build()
}

fn init_logger() {
	stderrlog::new()
		.verbosity(4)
		.quiet(false)
		.show_level(true)
		.timestamp(Timestamp::Millisecond)
		.init()
		.unwrap_or(());
}

fn get_clients(rooms: Arc<Mutex<Rooms>>, room_hash: &HashValue) -> Vec<ClientInfo> {
	let rooms = &*rooms;
	let mut rooms = rooms.lock().unwrap();
	let (sender, receiver) = mpsc::channel();
	rooms
		.send_room_request(&room_hash, RoomRequest::GetClients(sender))
		.ok()
		.unwrap();
	receiver.recv_timeout(Duration::from_secs(1)).ok().unwrap()
}

fn get_objects(rooms: Arc<Mutex<Rooms>>, room_hash: &HashValue) -> Vec<ServerGameObjectId> {
	let rooms = &*rooms;
	let mut rooms = rooms.lock().unwrap();
	let (sender, receiver) = mpsc::channel();
	rooms
		.send_room_request(&room_hash, RoomRequest::GetObjects(sender))
		.unwrap_or_default();
	receiver.recv_timeout(Duration::from_secs(1)).ok().unwrap()
}
