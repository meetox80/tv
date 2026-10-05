#![allow(non_snake_case)]

use std::io;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use std::task::{Context, Poll};

use axum::{extract::{Request, State}, middleware::Next, response::Response};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};

#[derive(Default)]
pub struct Totals {
    Sent: AtomicU64,
    Received: AtomicU64,
    Requests: AtomicU64,
}

impl Totals {
    pub fn Sent(&self) -> u64 {
        self.Sent.load(Relaxed)
    }

    pub fn Received(&self) -> u64 {
        self.Received.load(Relaxed)
    }

    pub fn Requests(&self) -> u64 {
        self.Requests.load(Relaxed)
    }
}

pub struct Counted {
    Inner: TcpStream,
    Totals: Arc<Totals>,
}

impl AsyncRead for Counted {
    fn poll_read(mut self: Pin<&mut Self>, Cx: &mut Context<'_>, Buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let Before = Buf.filled().len();
        let Result = Pin::new(&mut self.Inner).poll_read(Cx, Buf);
        if let Poll::Ready(Ok(())) = Result {
            self.Totals.Received.fetch_add((Buf.filled().len() - Before) as u64, Relaxed);
        }
        Result
    }
}

impl AsyncWrite for Counted {
    fn poll_write(mut self: Pin<&mut Self>, Cx: &mut Context<'_>, Data: &[u8]) -> Poll<io::Result<usize>> {
        let Result = Pin::new(&mut self.Inner).poll_write(Cx, Data);
        if let Poll::Ready(Ok(Written)) = Result {
            self.Totals.Sent.fetch_add(Written as u64, Relaxed);
        }
        Result
    }

    fn poll_write_vectored(mut self: Pin<&mut Self>, Cx: &mut Context<'_>, Bufs: &[io::IoSlice<'_>]) -> Poll<io::Result<usize>> {
        let Result = Pin::new(&mut self.Inner).poll_write_vectored(Cx, Bufs);
        if let Poll::Ready(Ok(Written)) = Result {
            self.Totals.Sent.fetch_add(Written as u64, Relaxed);
        }
        Result
    }

    fn is_write_vectored(&self) -> bool {
        self.Inner.is_write_vectored()
    }

    fn poll_flush(mut self: Pin<&mut Self>, Cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.Inner).poll_flush(Cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, Cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.Inner).poll_shutdown(Cx)
    }
}

pub struct Listener {
    Inner: TcpListener,
    Totals: Arc<Totals>,
}

impl Listener {
    pub fn New(Inner: TcpListener, Totals: Arc<Totals>) -> Self {
        Self { Inner, Totals }
    }
}

impl axum::serve::Listener for Listener {
    type Io = Counted;
    type Addr = SocketAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (Stream, Addr) = axum::serve::Listener::accept(&mut self.Inner).await;
        (Counted { Inner: Stream, Totals: Arc::clone(&self.Totals) }, Addr)
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        self.Inner.local_addr()
    }
}

#[derive(Clone, Copy)]
pub struct Peer(pub SocketAddr);

impl axum::extract::connect_info::Connected<axum::serve::IncomingStream<'_, Listener>> for Peer {
    fn connect_info(Stream: axum::serve::IncomingStream<'_, Listener>) -> Self {
        Peer(*Stream.remote_addr())
    }
}

pub async fn CountRequest(State(Totals): State<Arc<Totals>>, Req: Request, Next: Next) -> Response {
    Totals.Requests.fetch_add(1, Relaxed);
    Next.run(Req).await
}
