use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures_core::Stream;
use pin_project_lite::pin_project;

pin_project! {
    pub(crate) struct Combine<T, S, F>
    where
        S: Stream<Item = T>,
        F: Future<Output = ()>,
    {
        #[pin]
        stream: S,
        #[pin]
        task: F,
    }
}

impl<T, S, F> Combine<T, S, F>
where
    S: Stream<Item = T>,
    F: Future<Output = ()>,
{
    pub(crate) fn new(stream: S, task: F) -> Self {
        Self { stream, task }
    }
}

impl<T, S, F> Stream for Combine<T, S, F>
where
    S: Stream<Item = T>,
    F: Future<Output = ()>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.task.poll(cx) {
            Poll::Ready(_) => Poll::Ready(None),
            Poll::Pending => match this.stream.poll_next(cx) {
                Poll::Ready(item) => Poll::Ready(item),
                Poll::Pending => Poll::Pending,
            },
        }
    }
}