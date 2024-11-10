use anyhow::Result;
use crm_metadata::{config::AppConfig, MetadataService};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::load()?;
    let addr = config.server.port;
    let addr = format!("[::1]:{}", addr).parse().expect("Invalid address");

    info!("Starting metadata service on {}", addr);
    let svc = MetadataService::new(config).into_service();
    Server::builder().add_service(svc).serve(addr).await?;

    Ok(())
}
