use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use pin_project_lite::pin_project;
use tokio::time::{sleep, Duration, Instant, Sleep};
use tokio_stream::Stream;

pin_project! {
    /// A mock source that emits a unit item at a specified interval.
    pub struct MockSource<T> {
        interval: Duration,
        item: T,
        #[pin]
        sleeper: Sleep,
    }
}

impl<T> MockSource<T> {
    /// Creates a new `MockSource` with the specified interval and item.
    pub fn new(interval: Duration, item: T) -> Self {
        MockSource {
            interval,
            item,
            sleeper: sleep(interval),
        }
    }

    /// Creates a new `MockSource` with the specified interval and a default item.
    pub fn with_interval(interval: Duration) -> Self
    where
        T: Default,
    {
        Self::new(interval, T::default())
    }

    /// Returns the interval of the `MockSource`.
    pub fn interval(&self) -> Duration {
        self.interval
    }

    /// Returns a reference to the item of the `MockSource`.
    pub fn item(&self) -> &T {
        &self.item
    }
}

impl<T> Stream for MockSource<T>
where
    T: Clone,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        match this.sleeper.as_mut().poll(cx) {
            Poll::Ready(_) => {
                this.sleeper.as_mut().reset(Instant::now() + *this.interval);
                Poll::Ready(Some(this.item.clone()))
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}