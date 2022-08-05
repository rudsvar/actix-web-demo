//! The echo gRPC service.

use tonic::{Request, Response, Status};

/// Generated types for the echo service.
pub mod generated {
    tonic::include_proto!("string");
}

use generated::{string_service_server::StringService, Message};

/// A service that echoes requests.
#[derive(Clone, Copy, Default, Debug)]
pub struct MyStringService;

#[tonic::async_trait]
impl StringService for MyStringService {
    async fn echo(&self, req: Request<Message>) -> Result<Response<Message>, Status> {
        let req = req.into_inner();
        Ok(Response::new(Message {
            message: req.message,
        }))
    }

    async fn reverse(&self, req: Request<Message>) -> Result<Response<Message>, Status> {
        let req = req.into_inner();
        Ok(Response::new(Message {
            message: req.message.chars().rev().collect(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        generated::{string_service_server::StringService, Message},
        MyStringService,
    };
    use tonic::Request;

    #[tokio::test]
    async fn echo_works() {
        let service = MyStringService;
        let request = Request::new(Message {
            message: "hello!".to_string(),
        });
        let response = service.echo(request).await.unwrap().into_inner();

        assert_eq!(
            Message {
                message: "hello!".to_string()
            },
            response
        );
    }

    #[tokio::test]
    async fn reverse_works() {
        let service = MyStringService;
        let request = Request::new(Message {
            message: "hello!".to_string(),
        });
        let response = service.reverse(request).await.unwrap().into_inner();

        assert_eq!(
            Message {
                message: "!olleh".to_string()
            },
            response
        );
    }

    #[tokio::test]
    async fn reverse_utf8_works() {
        let service = MyStringService;
        let request = Request::new(Message {
            message: "💯".to_string(),
        });
        let response = service.reverse(request).await.unwrap().into_inner();

        assert_eq!(
            Message {
                message: "💯".to_string()
            },
            response
        );
    }
}
