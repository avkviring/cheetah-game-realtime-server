use crate::service::FactoryService;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl crate::proto::internal::factory_server::Factory for FactoryService {
    async fn create_match(
        &self,
        request: Request<crate::proto::internal::CreateMatchRequest>,
    ) -> Result<Response<crate::proto::internal::CreateMatchResponse>, Status> {
        todo!()
    }
}
