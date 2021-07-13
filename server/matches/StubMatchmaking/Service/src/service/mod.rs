use std::collections::HashMap;

use tokio::sync::RwLock;
use tonic::transport::Uri;
use tonic::{Request, Response, Status};

use crate::proto::matches::factory::internal as factory;
use crate::proto::matches::matchmaking::external as matchmaking;
use crate::proto::matches::relay::internal as relay;

pub struct StubMatchmakingService {
    pub factory_service: Uri,
    pub matches: RwLock<HashMap<String, MatchInfo>>,
}

#[derive(Clone)]
pub struct MatchInfo {
    pub relay_grpc_host: String,
    pub relay_grpc_port: u16,
    pub relay_game_host: String,
    pub relay_game_port: u16,
    pub room_id: u64,
}

impl StubMatchmakingService {
    pub fn new(factory_service: Uri) -> Self {
        StubMatchmakingService {
            factory_service,
            matches: RwLock::new(HashMap::new()),
        }
    }
    async fn matchmaking(&self, ticket: matchmaking::TicketRequest) -> matchmaking::TicketResponse {
        let template = ticket.match_template.clone();
        let match_info = self.find_or_create_match(template).await;
        let user_attach_response = StubMatchmakingService::attach_user(ticket, &match_info).await;
        matchmaking::TicketResponse {
            private_key: user_attach_response.private_key,
            user_id: user_attach_response.user_id,
            room_id: match_info.room_id,
            relay_game_host: match_info.relay_game_host,
            relay_game_port: match_info.relay_game_port as u32,
        }
    }

    async fn attach_user(
        ticket: matchmaking::TicketRequest,
        match_info: &MatchInfo,
    ) -> relay::AttachUserResponse {
        let mut relay = relay::relay_client::RelayClient::connect(format!(
            "http://{}:{}",
            match_info.relay_grpc_host, match_info.relay_grpc_port
        ))
        .await
        .unwrap();

        let user_attach_response = relay
            .attach_user(Request::new(relay::AttachUserRequest {
                room_id: match_info.room_id,
                user: ticket.user,
            }))
            .await
            .unwrap()
            .into_inner();
        user_attach_response
    }

    async fn find_or_create_match(&self, template: String) -> MatchInfo {
        let mut matches = self.matches.write().await;
        match matches.get(&template) {
            None => {
                let mut factory =
                    factory::factory_client::FactoryClient::connect(self.factory_service.clone())
                        .await
                        .unwrap();

                let create_match_response = factory
                    .create_match(Request::new(factory::CreateMatchRequest {
                        template: template.clone(),
                    }))
                    .await
                    .unwrap()
                    .into_inner();
                let match_info = MatchInfo {
                    relay_grpc_host: create_match_response.relay_grpc_host,
                    relay_grpc_port: create_match_response.relay_grpc_port as u16,
                    relay_game_host: create_match_response.relay_game_host,
                    relay_game_port: create_match_response.relay_game_port as u16,
                    room_id: create_match_response.id,
                };
                matches.insert(template.clone(), match_info.clone());
                match_info
            }
            Some(match_info) => match_info.clone(),
        }
    }
}

#[tonic::async_trait]
impl matchmaking::matchmaking_server::Matchmaking for StubMatchmakingService {
    async fn matchmaking(
        &self,
        request: Request<matchmaking::TicketRequest>,
    ) -> Result<tonic::Response<matchmaking::TicketResponse>, tonic::Status> {
        let ticket_request = request.into_inner();
        let response = self.matchmaking(ticket_request).await;
        Result::Ok(Response::new(response))
    }
}
