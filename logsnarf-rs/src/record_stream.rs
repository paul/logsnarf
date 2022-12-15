use std::io::Result;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use pin_project_lite::pin_project;
use tokio::io::{AsyncRead, ReadBuf};

pin_project! {
    pub struct RecordStream<I> {
        #[pin]
        inner: I,
        received: Arc<AtomicUsize>,
    }
}

impl<I> RecordStream<I> {
    pub fn new(inner: I, received: Arc<AtomicUsize>) -> Self {
        Self { inner, received }
    }
}

impl<I: AsyncRead> AsyncRead for RecordStream<I> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let this = self.project();
        let poll = this.inner.poll_read(cx, buf);

        this.received
            .fetch_add(buf.filled().len(), Ordering::Release);

        poll
    }
}
