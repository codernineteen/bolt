use server_core::network::service::ServiceHandle;
use std::io::Error as IoError;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    // TODO : service initialize
    env_logger::init();

    ServiceHandle::instance().start_service().await;
    tokio::signal::ctrl_c().await.unwrap();

    Ok(())
}
