use bytes::BytesMut;
use tokio::{
  io::{AsyncRead, AsyncReadExt},
  process::Child,
  spawn,
  task::JoinHandle,
};

use crate::app::apps::rust::Error;
pub trait TokioReadExt: AsyncRead + Unpin + Send {
  fn read_to_end(
    &mut self,
  ) -> impl Future<Output = Result<BytesMut, tokio::io::Error>> + Send;
  fn read_to_end_with<F>(
    &mut self, f: F,
  ) -> impl Future<Output = Result<BytesMut, Error>> + Send
  where F: Fn(&[u8], &[u8]) + Send;
}
pub trait TokioReadTaskExt: TokioReadExt {
  fn spawn_read(self) -> JoinHandle<Result<BytesMut, tokio::io::Error>>
  where Self: Sized + Send + 'static;
  fn spawn_read_with<F>(self, f: F) -> JoinHandle<Result<BytesMut, Error>>
  where
    Self: Sized + Send + 'static,
    F: Fn(&[u8], &[u8]) + Send + 'static;
  fn spawn_read_opt<F>(
    self, f: Option<F>,
  ) -> JoinHandle<Result<BytesMut, Error>>
  where
    Self: Sized + Send + 'static,
    F: Fn(&[u8], &[u8]) + Send + 'static;
}
pub trait TokioChildExt {
  async fn read_out<F1, F2>(
    &mut self, on_stdout: Option<F1>, on_stderr: Option<F2>,
  ) -> Result
  where
    F1: Fn(&[u8], &[u8]) + Send + 'static,
    F2: Fn(&[u8], &[u8]) + Send + 'static;
}
impl TokioChildExt for Child {
  async fn read_out<F1, F2>(
    &mut self, on_stdout: Option<F1>, on_stderr: Option<F2>,
  ) where
    F1: Fn(&[u8], &[u8]) + Send + 'static,
    F2: Fn(&[u8], &[u8]) + Send + 'static,
  {
    let stdout_task = self.stdout.take().unwrap().spawn_read_opt(on_stdout);
    let stderr_task = self.stderr.take().unwrap().spawn_read_opt(on_stderr);
    let (stdout_buf, stderr_buf) = tokio::try_join!(stdout_task, stderr_task)?;
  }
}
impl<T: AsyncRead + Send + Unpin> TokioReadExt for T {
  async fn read_to_end(&mut self) -> Result<BytesMut, tokio::io::Error> {
    let mut buf = BytesMut::new();
    while self.read_buf(&mut buf).await? != 0 {}
    Ok(buf)
  }

  async fn read_to_end_with<F>(&mut self, f: F) -> Result<BytesMut, Error>
  where F: Fn(&[u8], &[u8]) + Send {
    let mut buf = BytesMut::new();
    let mut last_cursor = 0;
    while self.read_buf(&mut buf).await? != 0 {
      f(&buf, &buf[last_cursor..]);
      last_cursor = buf.len();
    }
    Ok(buf)
  }
}
impl<T: TokioReadExt> TokioReadTaskExt for T {
  fn spawn_read(mut self) -> JoinHandle<Result<BytesMut, tokio::io::Error>>
  where Self: Sized + Send + 'static {
    spawn(async move { self.read_to_end().await })
  }

  fn spawn_read_with<F>(mut self, f: F) -> JoinHandle<Result<BytesMut, Error>>
  where
    Self: Sized + Send + 'static,
    F: Fn(&[u8], &[u8]) + Send + 'static,
  {
    spawn(async move { self.read_to_end_with(f).await })
  }

  fn spawn_read_opt<F>(
    self, f: Option<F>,
  ) -> JoinHandle<Result<BytesMut, Error>>
  where
    Self: Sized + Send + 'static,
    F: Fn(&[u8], &[u8]) + Send + 'static,
  {
    match f {
      Some(func) => self.spawn_read_with(func),
      None => self.spawn_read(),
    }
  }
}
