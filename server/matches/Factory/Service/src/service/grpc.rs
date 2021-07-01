use tonic::{Request, Response, Status};

use crate::proto::matches::factory::internal as factory_internal;
use crate::service::FactoryService;

#[tonic::async_trait]
impl factory_internal::factory_server::Factory for FactoryService {
    async fn create_match(
        &self,
        request: Request<factory_internal::CreateMatchRequest>,
    ) -> Result<Response<factory_internal::CreateMatchResponse>, Status> {
        todo!()
    }
}
