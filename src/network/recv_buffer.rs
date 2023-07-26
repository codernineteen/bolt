use {
    bytes::{Buf, BytesMut},
    std::io::{self, Cursor},
    tokio::{
        io::{AsyncReadExt, AsyncWriteExt, BufWriter},
        net::TcpStream,
    },
};

/**
 * Receive Buffer
 */

// RecvBuffer is an abstraction to handle recv data from TcpStream

pub struct RecvBuffer {
    pub capacity: usize,
    buffer: BytesMut,
}

impl RecvBuffer {
    pub fn new(buffer_size: usize) -> Self {
        Self {
            capacity: buffer_size,
            buffer: BytesMut::with_capacity(buffer_size),
        }
    }

    pub fn on_read() {}
}
