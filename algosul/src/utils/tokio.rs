use bytes::BytesMut;
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::app::apps::rust::Error;
pub trait AsyncBufReadExt: AsyncRead {
    async fn read_to_end_with<F>(&mut self, f: F) -> Result<BytesMut, Error>
    where
        Self: Unpin,
        F: Fn(&[u8], &[u8]);
}
impl<T: AsyncRead> AsyncBufReadExt for T {
    async fn read_to_end_with<F>(&mut self, f: F) -> Result<BytesMut, Error>
    where
        Self: Unpin,
        F: Fn(&[u8], &[u8]),
    {
        let mut buf = BytesMut::new();
        let mut last_cursor = 0;
        loop {
            if self.read_buf(&mut buf).await? == 0 {
                break;
            }
            f(&buf, &buf[last_cursor..]);
            last_cursor = buf.len();
        }
        Ok(buf)
    }
}
