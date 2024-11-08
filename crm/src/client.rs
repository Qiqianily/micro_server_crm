use anyhow::Result;
use crm::pb::user_service_client::UserServiceClient;
use crm::pb::CreateUserRequest;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = UserServiceClient::connect("http://[::1]:50051").await?;
    let request = Request::new(CreateUserRequest {
        name: "Tom".to_string(),
        email: "tom@163.com".to_string(),
    });
    let user = client.create_user(request).await?.into_inner();
    println!("Returned user={:?}", user);
    Ok(())
}
