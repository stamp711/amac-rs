use std::collections::VecDeque;

use futures::{pin_mut, Future};

pub struct LocalPool<F> {
    queue: VecDeque<F>,
}

impl<F: Future + Unpin> LocalPool<F> {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
        }
    }

    pub fn from_futures(futures: Vec<F>) -> Self {
        Self {
            queue: futures.into(),
        }
    }

    pub fn add_future(&mut self, future: F) {
        self.queue.push_back(future);
    }

    pub fn run_until_finish(mut self) {
        while let Some(mut future) = self.queue.pop_front() {
            let fut = &mut future;
            pin_mut!(fut);
            match fut.poll(&mut futures::task::Context::from_waker(
                futures::task::noop_waker_ref(),
            )) {
                std::task::Poll::Pending => self.queue.push_back(future),
                std::task::Poll::Ready(_) => {}
            }
        }
    }
}
