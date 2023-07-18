use std::collections::HashMap;

use {
    super::{listener::Listener, net_addr::NetAddress, session::Session},
    std::{
        io::Error,
        net::SocketAddr,
        sync::{Arc, Weak},
    },
    tokio::sync::Mutex,
};

type SessionMap = Arc<Mutex<HashMap<SocketAddr, Session>>>;

pub struct Service {
    net_addr: NetAddress,
    max_session_count: u32,
    session_count: u32,
    sessions: SessionMap,
    listener: Arc<Mutex<Listener>>,
    service_ref: Weak<Service>, // shared pointer for this struct itself
}

impl Service {
    // Construct reference counted Service
    pub fn new(net_addr: NetAddress, max_session: u32) -> Arc<Self> {
        Arc::new_cyclic(|myself| Service {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            net_addr,
            max_session_count: max_session,
            session_count: 0,
            listener: Arc::new(Mutex::new(Listener::default())),
            service_ref: myself.clone(),
        })
    }

    pub fn max_session_count(&self) -> u32 {
        self.max_session_count
    }

    pub fn net_addr_str(&self) -> &String {
        &self.net_addr.addr_str
    }

    pub fn get_service_ref(&self) -> Arc<Self> {
        self.service_ref.upgrade().unwrap()
    }

    pub async fn start_service(&self) -> Result<bool, Error> {
        self.listener
            .lock()
            .await
            .start_accept(self.get_service_ref())
            .await
    }
}
