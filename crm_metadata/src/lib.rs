use crate::config::AppConfig;
use crate::pb::metadata_server::MetadataServer;
use crate::pb::Content;
use futures::Stream;
use std::pin::Pin;
use tonic::{Response, Status};

mod abi;
pub mod config;
pub mod pb;
pub use abi::Tpl;

type ServiceResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;

// The service implementation
#[allow(dead_code)]
pub struct MetadataService {
    config: AppConfig,
}

// The implementation of the service
impl MetadataService {
    pub fn new(config: AppConfig) -> Self {
        MetadataService { config }
    }

    // converts the service into a tonic service
    pub fn into_service(self) -> MetadataServer<MetadataService> {
        MetadataServer::new(self)
    }
}
