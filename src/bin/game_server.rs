use server_core::network::service::ServiceHandle;
use std::io::Error as IoError;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    // TODO : service initialize
    let service = ServiceHandle::default();
    service.start_service().await.expect("error occurs");

    Ok(())
}
