use super::rpc_service_client::RpcServiceClient;
use crate::{DeploymentResponse, ProjectResponse};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tonic::codegen::InterceptedService;
use tonic::metadata::MetadataValue;
use tonic::service::Interceptor;
use tonic::transport::Channel;
use tonic::{Request, Status};

/// ClientTokenInterceptor is a interceptor to add jwt token to request
pub struct ClientTokenInterceptor {
    token: String,
}
impl Interceptor for ClientTokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        let token_value = format!("Bearer {}", self.token);
        let token: MetadataValue<_> = token_value.parse().unwrap();
        req.metadata_mut().insert("authorization", token);

        let grpc_method: MetadataValue<_> = "lol-cli".parse().unwrap();
        req.metadata_mut().insert("x-grpc-method", grpc_method);
        Ok(req)
    }
}

pub struct Client {
    client: RpcServiceClient<InterceptedService<Channel, ClientTokenInterceptor>>,
}

impl Client {
    pub async fn new(addr: String, token: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Channel::from_shared(addr)?.connect().await?;
        let client = RpcServiceClient::with_interceptor(channel, ClientTokenInterceptor { token });
        Ok(Client { client })
    }

    pub async fn fetch_project(
        &mut self,
        name: String,
        language: String,
    ) -> Result<Option<ProjectResponse>, Box<dyn std::error::Error>> {
        let req = tonic::Request::new(super::FetchProjectRequest { name, language });
        let resp = self.client.fetch_project(req).await;
        if resp.is_err() {
            let err = resp.err().unwrap();
            if err.code() == tonic::Code::NotFound {
                return Ok(None);
            }
            return Err(err.into());
        }
        Ok(Some(resp.unwrap().into_inner()))
    }

    pub async fn create_project(
        &mut self,
        name: String,
        language: String,
    ) -> Result<Option<ProjectResponse>, Box<dyn std::error::Error>> {
        let req = tonic::Request::new(super::FetchProjectRequest { name, language });
        let resp = self.client.create_empty_project(req).await?;
        Ok(Some(resp.into_inner()))
    }

    pub async fn create_deployment(
        &mut self,
        project_name: String,
        project_uuid: String,
        binary: Vec<u8>,
        content_type: String,
        is_production: bool,
    ) -> Result<Option<DeploymentResponse>, Box<dyn std::error::Error>> {
        let deploy_name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();
        let req = tonic::Request::new(super::CreateDeploymentRequest {
            project_name,
            project_uuid,
            deploy_name: deploy_name.to_lowercase(),
            deploy_chunk: binary,
            deploy_content_type: content_type,
        });
        let resp = self.client.create_deployment(req).await?;
        let mut deploy_resp = resp.into_inner();

        // set production
        if is_production {
            let req = tonic::Request::new(super::PublishDeploymentRequest {
                deploy_id: deploy_resp.id as i64,
                deploy_uuid: deploy_resp.uuid,
            });
            let resp2 = self.client.publish_deployment(req).await?;
            deploy_resp = resp2.into_inner();
        }
        Ok(Some(deploy_resp))
    }
}