use super::session::SessionHandle;
use futures_util::stream::{SplitSink, SplitStream};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;

pub type WsMessageSender = mpsc::UnboundedSender<Message>;
pub type WsMessageReceiver = mpsc::UnboundedReceiver<Message>;
pub type WsStream = WebSocketStream<TcpStream>;
pub type WsSendBuffer = SplitSink<WsStream, Message>;
pub type WsRecvBuffer = SplitStream<WsStream>;
pub type SessionMap = Arc<Mutex<HashMap<SocketAddr, SessionHandle>>>;
pub type WsMessage = Message;
