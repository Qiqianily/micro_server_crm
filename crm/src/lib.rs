mod abi;
pub mod pb;
use crate::pb::{user_service_server::UserService, CreateUserRequest, GetUserRequest, User};
use anyhow::Result;
use tonic::{Request, Response, Status};

#[derive(Default)]
pub struct UserServer {}
#[tonic::async_trait]
impl UserService for UserServer {
    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("Got a request: {:?}", input);
        Ok(Response::new(User::default()))
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let input = request.into_inner();
        println!("create_user: {:?}", input);
        let user = User::new(666, &input.name, &input.email);
        Ok(Response::new(user))
    }
}
