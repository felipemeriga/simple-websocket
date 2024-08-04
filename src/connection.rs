use crate::error::{CloseError, StreamError};
use crate::frame::{Frame, OpCode};
use std::time::Duration;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::timeout;

const CLOSE_TIMEOUT: u64 = 5;
pub struct WSConnection {
    pub read: Receiver<Result<Vec<u8>, StreamError>>,
    write: Sender<Frame>,
    close_rx: Receiver<bool>,
}

impl WSConnection {
    pub fn new(
        read: Receiver<Result<Vec<u8>, StreamError>>,
        write: Sender<Frame>,
        close_rx: Receiver<bool>,
    ) -> Self {
        Self {
            read,
            write,
            close_rx,
        }
    }

    pub async fn close_connection(&mut self) -> Result<(), CloseError> {
        self.write
            .send(Frame::new(true, OpCode::Close, Vec::new()))
            .await?;

        match timeout(Duration::from_secs(CLOSE_TIMEOUT), self.close_rx.recv()).await {
            Err(err) => Err(err)?,
            _ => Ok(()),
        }
    }

    pub async fn send_binary_frame(&mut self, data: Vec<u8>) -> Result<(), SendError<Frame>> {
        self.write
            .send(Frame::new(true, OpCode::Binary, data))
            .await
    }

    pub async fn send_data(&mut self, data: Vec<u8>) -> Result<(), SendError<Frame>> {
        self.write.send(Frame::new(true, OpCode::Text, data)).await
    }

    pub async fn send_ping(&mut self) -> Result<(), SendError<Frame>> {
        self.write
            .send(Frame::new(true, OpCode::Ping, Vec::new()))
            .await
    }

    pub async fn send_large_data_fragmented(
        &mut self,
        data: Vec<u8>,
    ) -> Result<(), SendError<Frame>> {
        // We can set the MAX_FRAGMENT_SIZE to 65536 bytes(64KB), which is the maximum
        // size of a TCP packet. As TCP is based in packets, and HTTP and WS works on the top of TCP, any
        // fragment greater than 64KB would still work, since it will be divided into packets
        const MAX_FRAGMENT_SIZE: usize = 64 * 1024;

        let chunks = data.chunks(MAX_FRAGMENT_SIZE);
        let total_chunks = chunks.len();

        for (i, chunk) in chunks.enumerate() {
            let is_final = i == total_chunks - 1;

            let opcode = if i == 0 {
                OpCode::Text
            } else {
                OpCode::Continue
            };

            println!(
                "Sending chunk: {}, as opcode: {:?}, data: {:?}",
                i + 1,
                opcode,
                String::from_utf8(Vec::from(chunk))
            );
            self.write
                .send(Frame::new(is_final, opcode, Vec::from(chunk)))
                .await?
        }

        Ok(())
    }
}
