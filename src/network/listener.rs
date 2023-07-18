use super::session;

use {
    super::{service::Service, session::Session},
    std::{io::Error, sync::Arc},
    tokio::net::TcpListener,
};

#[derive(Default)]
pub struct Listener {
    socket: Option<TcpListener>,
    service: Option<Arc<Service>>,
    listen_sessions: Vec<Session>,
}

impl Listener {
    pub async fn start_accept(&mut self, service: Arc<Service>) -> Result<bool, Error> {
        let try_socket = TcpListener::bind(service.clone().net_addr_str()).await;
        let listener = try_socket.expect("Failed to bind addr");

        println!("Listening on {}", service.as_ref().net_addr_str());

        // assign instance into each fields
        self.socket = Some(listener);
        self.service = Some(service);

        while let Ok((stream, addr)) = self.socket.as_mut().unwrap().accept().await {
            tokio::spawn(Session::handle_connection(stream, addr));
        }

        Ok(true)
    }
}
