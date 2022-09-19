use std::collections::HashMap;

use jwt_tonic_user_uuid::JWTUserTokenParser;
use tokio::sync::RwLock;
use tonic::{
	transport::{Channel, Uri},
	Code, Request, Response, Status,
};

use cheetah_libraries_microservice::trace::Trace;
use factory::internal::factory_client::FactoryClient;
use factory::internal::CreateMatchRequest;
use matchmaking::external::matchmaking_server::Matchmaking;
use matchmaking::external::{TicketRequest, TicketResponse};

use crate::proto::matches::factory::internal::CreateMatchResponse;
use crate::proto::matches::matchmaking;
use crate::proto::matches::realtime::internal::realtime_client::RealtimeClient;
use crate::proto::matches::realtime::internal::{CreateMemberRequest, CreateMemberResponse, QueryRoomRequest, QueryRoomResponse};
use crate::proto::matches::{factory, realtime};

pub struct StubMatchmakingService {
	pub jwt_public_key: String,
	pub factory_service_uri: Uri,
	pub matches: RwLock<HashMap<String, MatchInfo>>,
	rules: Rules,
}

struct Rules {
	max_user_count: u32,
}

impl Default for Rules {
	fn default() -> Self {
		Self { max_user_count: 4 }
	}
}

#[derive(Clone)]
struct ServerInfo {
	grpc_host: String,
	grpc_port: u16,
	udp_host: String,
	udp_port: u16,
}

#[derive(Clone)]
struct RoomInfo {
	id: u64,
}

#[derive(Clone)]
pub struct MatchInfo {
	server: ServerInfo,
	room: RoomInfo,
}

impl From<CreateMatchResponse> for MatchInfo {
	fn from(response: CreateMatchResponse) -> Self {
		let addrs = response.addrs.unwrap();
		let grpc_addr = addrs.grpc_internal.unwrap();
		let game_addr = addrs.game.unwrap();
		MatchInfo {
			server: ServerInfo {
				grpc_host: grpc_addr.host,
				grpc_port: grpc_addr.port as u16,
				udp_host: game_addr.host,
				udp_port: game_addr.port as u16,
			},
			room: RoomInfo { id: response.id },
		}
	}
}

impl StubMatchmakingService {
	pub fn new(factory_service: Uri, jwt_public_key: String) -> Self {
		StubMatchmakingService {
			jwt_public_key,
			factory_service_uri: factory_service,
			matches: RwLock::new(HashMap::new()),
			rules: Default::default(),
		}
	}

	async fn matchmake(&self, ticket: TicketRequest) -> Result<TicketResponse, String> {
		let template = ticket.match_template.clone();
		match self.try_matchmake(&ticket, &template).await {
			Ok(ticket) => {
				tracing::info!("Ticket created {:?}", ticket);
				Ok(ticket)
			}
			Err(e) => {
				tracing::error!("Cannot attach_user {}", e);
				// если  такой комнаты нет - то удаляем ее из существующих
				let mut matches = self.matches.write().await;
				matches.remove(&template);
				drop(matches);
				// и создаем снова
				self.try_matchmake(&ticket, &template).await
			}
		}
	}

	async fn try_matchmake(&self, ticket: &TicketRequest, template: &str) -> Result<TicketResponse, String> {
		let match_info = self.find_or_create_match(&template).await?;
		match self.attach_user(ticket, &match_info).await {
			Ok(member_attach_response) => Ok(TicketResponse {
				private_key: member_attach_response.private_key,
				member_id: member_attach_response.user_id,
				room_id: match_info.room.id,
				realtime_server_host: match_info.server.udp_host,
				realtime_server_port: match_info.server.udp_port as u32,
			}),
			Err(e) => Err(e),
		}
	}

	async fn find_or_create_match(&self, template: &str) -> Result<MatchInfo, String> {
		let matches = self.matches.read().await;
		let match_infos: Vec<&MatchInfo> = matches.iter().filter_map(|m| if m.0 == template { Some(m.1) } else { None }).collect();
		let maybe_match = self.select_by_rules(match_infos).await.cloned();
		drop(matches);

		let match_info = match maybe_match {
			Some(match_info) => match_info,
			None => self.create_match(template).await?,
		};

		Ok(match_info)
	}

	async fn select_by_rules<'a, 'b>(&'b self, match_infos: Vec<&'a MatchInfo>) -> Option<&'a MatchInfo> {
		// Фильтруем комнаты в соответствии со значениями в self.rules
		let mut suitable_rooms = Vec::with_capacity(match_infos.len());
		for mi in match_infos {
			let maybe_room = self.query_room(mi).await;
			match maybe_room {
				Ok(room_state) => {
					if room_state.user_count < self.rules.max_user_count {
						suitable_rooms.push((room_state, mi))
					}
				}
				Err(e) => {
					tracing::warn!("Failed to query room: {}", e);
				}
			}
		}

		// Выбираем одну комнату из подходящих
		suitable_rooms
			.into_iter()
			.max_by_key(|(room_state, _)| room_state.user_count)
			.map(|(_, match_info)| match_info)
	}

	async fn create_match(&self, template: &str) -> Result<MatchInfo, String> {
		let mut factory = FactoryClient::connect(self.factory_service_uri.clone()).await.unwrap();

		let create_match_response = factory
			.create_match(Request::new(CreateMatchRequest {
				template: template.to_string(),
			}))
			.await
			.map_err(|e| format!("Failed to create a match: {:?}", e))?
			.into_inner();

		let match_info: MatchInfo = create_match_response.into();

		let mut matches = self.matches.write().await;
		matches.insert(template.to_string(), match_info.clone());

		Ok(match_info)
	}

	async fn attach_user(&self, ticket: &TicketRequest, match_info: &MatchInfo) -> Result<CreateMemberResponse, String> {
		let mut relay = self.new_realtime_connection(&match_info.server).await?;

		relay
			.create_member(Request::new(CreateMemberRequest {
				room_id: match_info.room.id,
				user: Some(realtime::internal::UserTemplate {
					groups: ticket.user_groups,
					objects: Default::default(),
				}),
			}))
			.await
			.map(|response| response.into_inner())
			.map_err(|status| self.transform_bad_realtime_response_status(status))
	}

	async fn query_room(&self, match_info: &MatchInfo) -> Result<QueryRoomResponse, String> {
		let mut relay = self.new_realtime_connection(&match_info.server).await?;
		relay
			.query_room(QueryRoomRequest { id: match_info.room.id })
			.await
			.map(|response| response.into_inner())
			.map_err(|status| self.transform_bad_realtime_response_status(status))
	}

	async fn new_realtime_connection(&self, server: &ServerInfo) -> Result<RealtimeClient<Channel>, String> {
		RealtimeClient::connect(cheetah_libraries_microservice::make_internal_srv_uri(&server.grpc_host, server.grpc_port))
			.await
			.map_err(|e| format!("Failed to connect to realtime error {:?}", e))
	}

	fn transform_bad_realtime_response_status(&self, status: Status) -> String {
		match status.code() {
			Code::NotFound => "Realtime server not found".to_string(),
			e => format!("Realtime server has unknown status: {:?}", e),
		}
	}
}

#[tonic::async_trait]
impl Matchmaking for StubMatchmakingService {
	async fn matchmaking(&self, request: Request<TicketRequest>) -> Result<Response<TicketResponse>, Status> {
		// TODO: заменить на новую схему авторизации
		let _user = JWTUserTokenParser::new(self.jwt_public_key.clone())
			.get_user_uuid_from_grpc(request.metadata())
			.trace_err(format!("Get user uuid {:?}", request.metadata()))
			.map_err(|_| Status::unauthenticated(""))?;

		let ticket_request = request.into_inner();
		self.matchmake(ticket_request)
			.await
			.trace_err("Matchmaking error")
			.map_err(|_| Status::internal(""))
			.map(Response::new)
	}
}

#[cfg(test)]
pub mod tests {
	use tokio::net::TcpListener;
	use tokio::sync::RwLock;
	use tokio_stream::wrappers::{ReceiverStream, UnboundedReceiverStream};
	use tonic::transport::Server;
	use tonic::{Request, Response, Status};

	use factory::internal::factory_server::Factory;
	use factory::internal::{CreateMatchRequest, CreateMatchResponse};
	use matchmaking::external::TicketRequest;
	use realtime::internal::CreateMemberResponse;

	use crate::proto::matches::factory;
	use crate::proto::matches::matchmaking;
	use crate::proto::matches::realtime;
	use crate::proto::matches::realtime::internal::{
		CreateMemberRequest, CreateSuperMemberRequest, EmptyRequest, ProbeRequest, ProbeResponse, QueryRoomRequest, QueryRoomResponse, RoomIdResponse,
	};
	use crate::proto::matches::registry::internal::{Addr, RelayAddrs};
	use crate::service::StubMatchmakingService;

	#[tokio::test]
	async fn should_create_match() {
		let matchmaking = setup(100, false).await;
		let response = matchmaking
			.matchmake(TicketRequest {
				user_groups: 0,
				match_template: Default::default(),
			})
			.await
			.unwrap();
		assert_eq!(response.room_id, StubFactory::ROOM_ID);
		assert_eq!(response.member_id, StubRealtimeService::MEMBER_ID);
	}

	///
	/// Повторный матчинг для одного и того же шаблона
	/// не должен привести к изменению id комнаты
	///
	#[tokio::test]
	async fn should_not_create_match_if_exists() {
		let matchmaking = setup(100, false).await;
		matchmaking
			.matchmake(TicketRequest {
				user_groups: Default::default(),
				match_template: "some-template".to_owned(),
			})
			.await
			.unwrap();
		let response = matchmaking
			.matchmake(TicketRequest {
				user_groups: Default::default(),
				match_template: "some-template".to_owned(),
			})
			.await
			.unwrap();
		assert_eq!(response.room_id, StubFactory::ROOM_ID);
	}
	///
	/// Для каждого шаблона должен быть собственный матч     
	///
	#[tokio::test]
	async fn should_create_different_match_for_different_template() {
		let matchmaking = setup(100, false).await;
		let response_a = matchmaking
			.matchmake(TicketRequest {
				user_groups: Default::default(),
				match_template: "some-template-a".to_owned(),
			})
			.await
			.unwrap();
		let response_b = matchmaking
			.matchmake(TicketRequest {
				user_groups: Default::default(),
				match_template: "some-template-b".to_owned(),
			})
			.await
			.unwrap();
		assert_eq!(response_a.room_id, StubFactory::ROOM_ID);
		assert_eq!(response_b.room_id, StubFactory::ROOM_ID + 1);
	}

	///
	/// Для каждого шаблона должен быть собственный матч     
	///
	#[tokio::test]
	async fn should_recreate_match_if_not_found() {
		let matchmaking = setup(1, false).await;
		let response_a = matchmaking
			.matchmake(TicketRequest {
				user_groups: Default::default(),
				match_template: "some-template".to_owned(),
			})
			.await
			.unwrap();
		let response_b = matchmaking
			.matchmake(TicketRequest {
				user_groups: Default::default(),
				match_template: "some-template".to_owned(),
			})
			.await
			.unwrap();
		assert_eq!(response_a.room_id, StubFactory::ROOM_ID);
		assert_eq!(response_b.room_id, StubFactory::ROOM_ID + 1);
	}

	#[tokio::test]
	async fn should_create_new_match_because_existing_one_full() {
		let matchmaking = setup(100, true).await;
		let request = TicketRequest {
			user_groups: Default::default(),
			match_template: "yes".to_owned(),
		};
		matchmaking.matchmake(request.clone()).await.unwrap();
		let response = matchmaking.matchmake(request).await.unwrap();

		assert_eq!(response.room_id, StubFactory::ROOM_ID + 1);
	}

	async fn setup(fail_create_user: i8, simulate_full_room: bool) -> StubMatchmakingService {
		let stub_grpc_service_tcp = TcpListener::bind("127.0.0.1:0").await.unwrap();
		let stub_grpc_service_addr = stub_grpc_service_tcp.local_addr().unwrap();

		let stub_factory = StubFactory {
			relay_grpc_host: stub_grpc_service_addr.ip().to_string(),
			relay_grpc_port: stub_grpc_service_addr.port(),
			room_sequence: RwLock::new(0),
		};
		let stub_relay = StubRealtimeService {
			fail_when_zero: RwLock::new(fail_create_user),
			expected_user_limit: 4,
			room_full: simulate_full_room,
		};
		tokio::spawn(async move {
			Server::builder()
				.add_service(factory::internal::factory_server::FactoryServer::new(stub_factory))
				.add_service(realtime::internal::realtime_server::RealtimeServer::new(stub_relay))
				.serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(stub_grpc_service_tcp))
				.await
		});

		let matchmaking = StubMatchmakingService::new(
			cheetah_libraries_microservice::make_internal_srv_uri(stub_grpc_service_addr.ip().to_string().as_str(), stub_grpc_service_addr.port()),
			Default::default(),
		);
		matchmaking
	}

	struct StubFactory {
		pub relay_grpc_host: String,
		pub relay_grpc_port: u16,
		pub room_sequence: RwLock<u16>,
	}

	impl StubFactory {
		pub const ROOM_ID: u64 = 555;
	}

	#[tonic::async_trait]
	impl Factory for StubFactory {
		async fn create_match(&self, _request: Request<CreateMatchRequest>) -> Result<Response<CreateMatchResponse>, Status> {
			let mut sequence = self.room_sequence.write().await;
			let current_seq = *sequence;
			*sequence += 1;
			Ok(Response::new(CreateMatchResponse {
				addrs: Some(RelayAddrs {
					// not used
					game: Some(Addr {
						host: "127.0.0.1".to_string(),
						port: 0,
					}),
					grpc_internal: Some(Addr {
						host: self.relay_grpc_host.clone(),
						port: self.relay_grpc_port as u32,
					}),
				}),
				id: StubFactory::ROOM_ID + current_seq as u64,
			}))
		}
	}

	struct StubRealtimeService {
		fail_when_zero: RwLock<i8>,
		expected_user_limit: u32,
		room_full: bool,
	}

	impl StubRealtimeService {
		pub const MEMBER_ID: u32 = 777;
	}

	#[tonic::async_trait]
	impl realtime::internal::realtime_server::Realtime for StubRealtimeService {
		async fn create_room(
			&self,
			_request: Request<realtime::internal::RoomTemplate>,
		) -> Result<Response<realtime::internal::RoomIdResponse>, Status> {
			unimplemented!()
		}

		async fn create_member(&self, _request: Request<CreateMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
			let mut fail = self.fail_when_zero.write().await;
			let current = *fail;
			*fail -= 1;
			if current == 0 {
				Err(Status::not_found(""))
			} else {
				Ok(Response::new(CreateMemberResponse {
					user_id: StubRealtimeService::MEMBER_ID,
					private_key: vec![],
				}))
			}
		}

		async fn create_super_member(&self, _request: Request<CreateSuperMemberRequest>) -> Result<Response<CreateMemberResponse>, Status> {
			unimplemented!()
		}

		async fn query_room(&self, _request: Request<QueryRoomRequest>) -> Result<Response<QueryRoomResponse>, Status> {
			let user_count = if self.room_full {
				self.expected_user_limit
			} else {
				(self.expected_user_limit - 1).clamp(0, self.expected_user_limit)
			};

			Ok(Response::new(QueryRoomResponse { user_count }))
		}

		async fn probe(&self, _request: Request<ProbeRequest>) -> Result<Response<ProbeResponse>, Status> {
			Ok(Response::new(ProbeResponse {}))
		}

		type WatchCreatedRoomEventStream = ReceiverStream<Result<RoomIdResponse, Status>>;

		async fn watch_created_room_event(&self, _request: Request<EmptyRequest>) -> Result<Response<Self::WatchCreatedRoomEventStream>, Status> {
			unimplemented!()
		}
	}
}
