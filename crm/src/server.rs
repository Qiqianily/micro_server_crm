use anyhow::Result;
use crm::pb::user_service_server::UserServiceServer;
use crm::UserServer;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:50051".parse()?;
    let svc = UserServer::default();
    println!("UserServer listening on {}", addr);
    Server::builder()
        .add_service(UserServiceServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}
