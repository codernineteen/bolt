use super::{
    global::G_SERVICE_HANDLE,
    service::ServiceHandle,
    types::{WsMessage, WsRecvBuffer, WsSendBuffer, WsStream},
};
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use std::net::SocketAddr;
use tokio::sync::mpsc;

/**
 * Session message definition
 */

#[derive(Debug)]
pub enum ReadSessionMessage {
    OnRecv,
}

#[derive(Debug)]
pub enum WriteSessionMessage {
    OnSend { ws_msg: WsMessage },
}

/**
 * Read Session struct
 */

pub struct ReadSession {
    addr: SocketAddr,
    recv_buffer: WsRecvBuffer,
    msg_receiver: mpsc::UnboundedReceiver<ReadSessionMessage>,
}

impl ReadSession {
    pub fn new(
        addr: SocketAddr,
        recv_buffer: WsRecvBuffer,
        msg_receiver: mpsc::UnboundedReceiver<ReadSessionMessage>,
    ) -> Self {
        Self {
            addr,
            recv_buffer,
            msg_receiver,
        }
    }

    pub async fn handle_recv_message(&mut self) {
        while let Some(msg) = self.recv_buffer.next().await {
            let msg = msg.unwrap();
            G_SERVICE_HANDLE.broadcast(msg, self.addr).await;
        }
    }
}

/**
 * Write Session struct
 */

pub struct WriteSession {
    addr: SocketAddr,
    send_buffer: WsSendBuffer,
    msg_receiver: mpsc::UnboundedReceiver<WriteSessionMessage>,
}

impl WriteSession {
    pub fn new(
        addr: SocketAddr,
        send_buffer: WsSendBuffer,
        msg_receiver: mpsc::UnboundedReceiver<WriteSessionMessage>,
    ) -> Self {
        Self {
            addr,
            send_buffer,
            msg_receiver,
        }
    }

    pub async fn handle_message(&mut self, msg: WriteSessionMessage) {
        match msg {
            WriteSessionMessage::OnSend { ws_msg } => {
                if let Err(e) = self.send_buffer.send(ws_msg).await {
                    error!("reason: {} | while send message to send_buffer", e);
                };
            }
        }
    }
}

/**
 * Session Handle Struct
 */

pub struct SessionHandle {
    pub read_sender: mpsc::UnboundedSender<ReadSessionMessage>,
    pub write_sender: mpsc::UnboundedSender<WriteSessionMessage>,
}

impl SessionHandle {
    pub fn new(addr: SocketAddr, ws_stream: WsStream) -> Self {
        let (read_sender, read_receiver) = mpsc::unbounded_channel::<ReadSessionMessage>();
        let (write_sender, write_receiver) = mpsc::unbounded_channel::<WriteSessionMessage>();

        let (send_buffer, recv_buffer) = ws_stream.split();
        let read_actor = ReadSession::new(addr, recv_buffer, read_receiver);
        let write_actor = WriteSession::new(addr, send_buffer, write_receiver);

        // separate actor into read and write part.
        tokio::spawn(run_read_session_actor(read_actor));
        tokio::spawn(run_write_session_actor(write_actor));

        Self {
            read_sender,
            write_sender,
        }
    }

    pub fn register_send(&self, ws_msg: WsMessage) {
        let msg = WriteSessionMessage::OnSend { ws_msg };
        self.write_sender
            .send(msg)
            .expect("failed to send message to session actor");
    }

    pub fn register_recv(&self) {
        let msg = ReadSessionMessage::OnRecv;
        self.read_sender
            .send(msg)
            .expect("Failed to send message to session actor");
    }
}

async fn run_read_session_actor(mut actor: ReadSession) {
    while let Some(msg) = actor.msg_receiver.recv().await {
        info!("Read Session Actor : {:?}", msg);
        match msg {
            ReadSessionMessage::OnRecv => actor.handle_recv_message().await,
        }
    }
}

async fn run_write_session_actor(mut actor: WriteSession) {
    while let Some(msg) = actor.msg_receiver.recv().await {
        info!("Write Session Actor : {:?}", msg);
        actor.handle_message(msg).await;
    }
}
