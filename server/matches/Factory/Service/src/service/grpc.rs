use tonic::transport::Channel;
use tonic::{Request, Response, Status};

use crate::proto::matches::factory::internal as factory_internal;
use crate::proto::matches::registry::internal as registry_internal;
use crate::proto::matches::relay::internal::relay_client::RelayClient;
use crate::service::FactoryService;

#[tonic::async_trait]
impl factory_internal::factory_server::Factory for FactoryService {
    async fn create_match(
        &self,
        request: Request<factory_internal::CreateMatchRequest>,
    ) -> Result<Response<factory_internal::CreateMatchResponse>, Status> {
        // ищем свободный relay сервер
        let mut registry_client = registry_internal::registry_client::RegistryClient::connect(
            self.registry_grpc_service_address.to_owned(),
        )
        .await
        .unwrap();
        let free_relay = registry_client
            .find_free_relay(registry_internal::FindFreeRelayRequest {})
            .await
            .unwrap()
            .into_inner();

        // создаем матч на relay сервере
        let mut relay = RelayClient::connect(format!(
            "{}:{}",
            free_relay.relay_grpc_host, free_relay.relay_grpc_port
        ))
        .await
        .unwrap();
        // получаем шаблон
        let template_name = &request.get_ref().template;
        let template = self.get_template(template_name);
        match template {
            None => Result::Err(Status::internal(format!(
                "Template {} not found",
                template_name
            ))),
            Some(template) => {
                // создаем комнату
                let create_room_result = relay.create_room(template).await?.into_inner();
                let room_id = create_room_result.id;
                Result::Ok(Response::new(factory_internal::CreateMatchResponse {
                    relay_grpc_host: free_relay.relay_grpc_host.clone(),
                    relay_grpc_port: free_relay.relay_grpc_port,
                    relay_game_host: free_relay.relay_game_host.clone(),
                    relay_game_port: free_relay.relay_game_port,
                    id: room_id,
                }))
            }
        }
    }
}
